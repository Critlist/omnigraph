/**
 * @file Service Error Handler Tests
 * @fileoverview Tests for standardized error handling patterns
 */

import { describe, test, expect, beforeEach, vi } from "vitest";

import {
	ServiceErrorHandler,
	ServiceErrorType,
	ServiceError,
	createServiceErrorHandler,
	getErrorMessage,
	propagateError,
} from "../../../utils/service-error-handler";
import { ok, err } from "../../../utils/result";

describe("ServiceErrorHandler", () => {
	let errorHandler: ServiceErrorHandler;
	const serviceName = "TestService";

	beforeEach(() => {
		errorHandler = createServiceErrorHandler(serviceName);
	});

	describe("Error Creation", () => {
		test("should create structured service error", () => {
			const error = errorHandler.createError(
				ServiceErrorType.VALIDATION,
				"Invalid input",
				{ field: "username" },
				new Error("Original error"),
			);

			expect(error).toBeInstanceOf(ServiceError);
			expect(error.type).toBe(ServiceErrorType.VALIDATION);
			expect(error.message).toBe("Invalid input");
			expect(error.serviceName).toBe(serviceName);
			expect(error.context).toEqual({ field: "username" });
			expect(error.cause?.message).toBe("Original error");
		});

		test("should create formatted error message", () => {
			const error = new ServiceError(
				ServiceErrorType.OPERATION,
				"Operation failed",
				"TestService",
				{ operation: "getData" },
				new Error("Network error"),
			);

			const formatted = error.getFormattedMessage();
			expect(formatted).toContain("[TestService]");
			expect(formatted).toContain("OPERATION_ERROR");
			expect(formatted).toContain("Operation failed");
			expect(formatted).toContain('{"operation":"getData"}');
			expect(formatted).toContain("Caused by: Network error");
		});
	});

	describe("Async Operation Wrapping", () => {
		test("should wrap successful async operation", async () => {
			const operation = async () => "success";

			const result = await errorHandler.wrapAsync(operation);

			expect(result.isOk()).toBe(true);
			if (result.isOk()) {
				expect(result.value).toBe("success");
			}
		});

		test("should wrap failing async operation", async () => {
			const operation = async () => {
				throw new Error("Operation failed");
			};

			const result = await errorHandler.wrapAsync(
				operation,
				ServiceErrorType.OPERATION,
				{ operationId: 123 },
			);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.type).toBe(ServiceErrorType.OPERATION);
				expect(result.error.message).toBe("Operation failed");
				expect(result.error.serviceName).toBe(serviceName);
				expect(result.error.context).toEqual({ operationId: 123 });
			}
		});

		test("should handle non-Error exceptions", async () => {
			const operation = async () => {
				throw new Error("String error");
			};

			const result = await errorHandler.wrapAsync(operation);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.message).toBe("String error");
				expect(result.error.cause).toBeUndefined();
			}
		});
	});

	describe("Sync Operation Wrapping", () => {
		test("should wrap successful sync operation", () => {
			const operation = () => 42;

			const result = errorHandler.wrap(operation);

			expect(result.isOk()).toBe(true);
			if (result.isOk()) {
				expect(result.value).toBe(42);
			}
		});

		test("should wrap failing sync operation", () => {
			const operation = () => {
				throw new Error("Sync error");
			};

			const result = errorHandler.wrap(
				operation,
				ServiceErrorType.VALIDATION,
				{ field: "email" },
			);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.type).toBe(ServiceErrorType.VALIDATION);
				expect(result.error.context).toEqual({ field: "email" });
			}
		});
	});

	describe("Input Validation", () => {
		test("should validate valid input", () => {
			const result = errorHandler.validateInput(
				"valid@email.com",
				(email) => email.includes("@"),
				"Invalid email format",
			);

			expect(result.isOk()).toBe(true);
			if (result.isOk()) {
				expect(result.value).toBe("valid@email.com");
			}
		});

		test("should reject invalid input", () => {
			const result = errorHandler.validateInput(
				"invalid-email",
				(email) => email.includes("@"),
				"Invalid email format",
				{ inputLength: 13 },
			);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.type).toBe(ServiceErrorType.INVALID_INPUT);
				expect(result.error.message).toBe("Invalid email format");
				expect(result.error.context).toEqual({ inputLength: 13 });
			}
		});
	});

	describe("Readiness Checking", () => {
		test("should pass readiness check when ready", () => {
			const result = errorHandler.checkReadiness(true);

			expect(result.isOk()).toBe(true);
		});

		test("should fail readiness check when not ready", () => {
			const result = errorHandler.checkReadiness(
				false,
				"Service is initializing",
			);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.type).toBe(ServiceErrorType.NOT_READY);
				expect(result.error.message).toBe("Service is initializing");
			}
		});

		test("should use default message when not provided", () => {
			const result = errorHandler.checkReadiness(false);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.message).toBe("Service not ready");
			}
		});
	});

	describe("Timeout Operations", () => {
		test("should complete operation within timeout", async () => {
			const operation = async () => {
				await new Promise((resolve) => setTimeout(resolve, 10));
				return "completed";
			};

			const result = await errorHandler.withTimeout(operation, 100);

			expect(result.isOk()).toBe(true);
			if (result.isOk()) {
				expect(result.value).toBe("completed");
			}
		});

		test("should timeout long-running operation", async () => {
			const operation = async () => {
				await new Promise((resolve) => setTimeout(resolve, 100));
				return "completed";
			};

			const result = await errorHandler.withTimeout(
				operation,
				10,
				"Custom timeout message",
			);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.type).toBe(ServiceErrorType.TIMEOUT);
				expect(result.error.message).toBe("Custom timeout message");
				expect(result.error.context).toEqual({ timeoutMs: 10 });
			}
		});

		test("should handle operation errors within timeout", async () => {
			const operation = async () => {
				throw new Error("Operation error");
			};

			const result = await errorHandler.withTimeout(operation, 100);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.type).toBe(ServiceErrorType.OPERATION);
				expect(result.error.message).toBe("Operation error");
			}
		});
	});

	describe("Error Result Creation", () => {
		test("should create error result with all parameters", () => {
			const result = errorHandler.error(
				ServiceErrorType.DEPENDENCY,
				"Database connection failed",
				{ database: "neo4j" },
				new Error("Connection refused"),
			);

			expect(result.isErr()).toBe(true);
			if (result.isErr()) {
				expect(result.error.type).toBe(ServiceErrorType.DEPENDENCY);
				expect(result.error.message).toBe("Database connection failed");
				expect(result.error.context).toEqual({ database: "neo4j" });
				expect(result.error.cause?.message).toBe("Connection refused");
			}
		});
	});
});

describe("ServiceError Utility Functions", () => {
	describe("getErrorMessage", () => {
		test("should extract formatted message from error result", () => {
			const errorResult = err(
				new ServiceError(
					ServiceErrorType.VALIDATION,
					"Test error",
					"TestService",
					{ field: "test" },
				),
			);

			const message = getErrorMessage(errorResult);
			expect(message).toContain("[TestService]");
			expect(message).toContain("VALIDATION_ERROR");
			expect(message).toContain("Test error");
		});

		test("should return no error message for ok result", () => {
			const okResult = ok("success");
			const message = getErrorMessage(okResult);
			expect(message).toBe("No error");
		});
	});

	describe("propagateError", () => {
		test("should propagate error with additional context", () => {
			const originalResult = err(
				new ServiceError(
					ServiceErrorType.OPERATION,
					"Original error",
					"OriginalService",
					{ originalField: "value" },
				),
			);

			const propagated = propagateError(originalResult, "NewService", {
				newField: "newValue",
			});

			expect(propagated.isErr()).toBe(true);
			if (propagated.isErr()) {
				expect(propagated.error.serviceName).toBe("NewService");
				expect(propagated.error.message).toBe("Original error");
				expect(propagated.error.type).toBe(ServiceErrorType.OPERATION);
				expect(propagated.error.context).toEqual({
					originalField: "value",
					newField: "newValue",
				});
			}
		});

		test("should handle propagation from ok result gracefully", () => {
			const okResult = ok("success");

			const propagated = propagateError(okResult, "NewService");

			expect(propagated.isErr()).toBe(true);
			if (propagated.isErr()) {
				expect(propagated.error.type).toBe(ServiceErrorType.INTERNAL);
				expect(propagated.error.message).toBe(
					"Attempted to propagate error from Ok result",
				);
			}
		});
	});

	describe("createServiceErrorHandler", () => {
		test("should create error handler with service name", () => {
			const handler = createServiceErrorHandler("MyService");

			const error = handler.createError(
				ServiceErrorType.VALIDATION,
				"Test",
			);
			expect(error.serviceName).toBe("MyService");
		});
	});
});

describe("ServiceErrorType Enum", () => {
	test("should have all expected error types", () => {
		expect(ServiceErrorType.INITIALIZATION).toBe("INITIALIZATION_ERROR");
		expect(ServiceErrorType.VALIDATION).toBe("VALIDATION_ERROR");
		expect(ServiceErrorType.OPERATION).toBe("OPERATION_ERROR");
		expect(ServiceErrorType.DEPENDENCY).toBe("DEPENDENCY_ERROR");
		expect(ServiceErrorType.TIMEOUT).toBe("TIMEOUT_ERROR");
		expect(ServiceErrorType.NOT_READY).toBe("SERVICE_NOT_READY");
		expect(ServiceErrorType.INVALID_INPUT).toBe("INVALID_INPUT");
		expect(ServiceErrorType.INTERNAL).toBe("INTERNAL_ERROR");
	});

	test("should have stable string values", () => {
		// These values should not change as they may be used for monitoring/alerting
		const expectedTypes = [
			"INITIALIZATION_ERROR",
			"VALIDATION_ERROR",
			"OPERATION_ERROR",
			"DEPENDENCY_ERROR",
			"TIMEOUT_ERROR",
			"SERVICE_NOT_READY",
			"INVALID_INPUT",
			"INTERNAL_ERROR",
		];

		const actualTypes = Object.values(ServiceErrorType);
		expect(actualTypes.sort()).toEqual(expectedTypes.sort());
	});
});
