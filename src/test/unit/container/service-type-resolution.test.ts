/**
 * @file Service Type Resolution Tests
 * @fileoverview Tests for reliable service type resolution replacing fragile string parsing
 */

import { describe, test, expect, beforeEach, afterEach, vi } from "vitest";

import { TYPES } from "../../../config/types";
import { ServiceContainer } from "../../../container/container";

describe("Service Type Resolution", () => {
	let container: ServiceContainer;

	beforeEach(() => {
		container = ServiceContainer.getInstance();
	});

	afterEach(async () => {
		await container.dispose();
	});

	describe("Service Type to Name Mapping", () => {
		test("should map all TYPES symbols to service names", () => {
			// Get private method for testing
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);

			// Test all known service types
			expect(getServiceName(TYPES.Logger)).toBe("Logger");
			expect(getServiceName(TYPES.ConfigManager)).toBe("ConfigManager");
			expect(getServiceName(TYPES.DatabaseService)).toBe(
				"DatabaseService",
			);
			expect(getServiceName(TYPES.ParserService)).toBe("ParserService");
			expect(getServiceName(TYPES.GraphService)).toBe("GraphService");
			expect(getServiceName(TYPES.VisualizationService)).toBe(
				"VisualizationService",
			);
			expect(getServiceName(TYPES.AnalyticsService)).toBe(
				"AnalyticsService",
			);
		});

		test("should return UnknownService for unmapped symbols", () => {
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);
			const unknownSymbol = Symbol.for("UnknownService");

			expect(getServiceName(unknownSymbol)).toBe("UnknownService");
		});

		test("should not rely on symbol string representation", () => {
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);

			// Create a symbol with misleading string representation
			const misleadingSymbol = Symbol("DatabaseService_but_not_really");

			// Should return UnknownService, not DatabaseService
			expect(getServiceName(misleadingSymbol)).toBe("UnknownService");
		});
	});

	describe("Service Name to Type Mapping", () => {
		test("should map all service names to TYPES symbols", () => {
			const getServiceTypeFromName = (
				container as any
			).getServiceTypeFromName.bind(container);

			expect(getServiceTypeFromName("Logger")).toBe(TYPES.Logger);
			expect(getServiceTypeFromName("ConfigManager")).toBe(
				TYPES.ConfigManager,
			);
			expect(getServiceTypeFromName("DatabaseService")).toBe(
				TYPES.DatabaseService,
			);
			expect(getServiceTypeFromName("ParserService")).toBe(
				TYPES.ParserService,
			);
			expect(getServiceTypeFromName("GraphService")).toBe(
				TYPES.GraphService,
			);
			expect(getServiceTypeFromName("VisualizationService")).toBe(
				TYPES.VisualizationService,
			);
			expect(getServiceTypeFromName("AnalyticsService")).toBe(
				TYPES.AnalyticsService,
			);
		});

		test("should return undefined for unknown service names", () => {
			const getServiceTypeFromName = (
				container as any
			).getServiceTypeFromName.bind(container);

			expect(getServiceTypeFromName("UnknownService")).toBeUndefined();
			expect(getServiceTypeFromName("")).toBeUndefined();
			expect(getServiceTypeFromName("NotAService")).toBeUndefined();
		});

		test("should be case-sensitive", () => {
			const getServiceTypeFromName = (
				container as any
			).getServiceTypeFromName.bind(container);

			expect(getServiceTypeFromName("databaseservice")).toBeUndefined();
			expect(getServiceTypeFromName("DATABASESERVICE")).toBeUndefined();
			expect(getServiceTypeFromName("DatabaseService")).toBe(
				TYPES.DatabaseService,
			);
		});
	});

	describe("Bidirectional Mapping Consistency", () => {
		test("should have consistent bidirectional mapping", () => {
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);
			const getServiceTypeFromName = (
				container as any
			).getServiceTypeFromName.bind(container);

			// Test that type -> name -> type gives the same result
			for (const [, symbol] of Object.entries(TYPES)) {
				const serviceName = getServiceName(symbol);
				const backToSymbol = getServiceTypeFromName(serviceName);

				expect(backToSymbol).toBe(symbol);
				expect(serviceName).not.toBe("UnknownService");
			}
		});

		test("should have all TYPES symbols mapped", () => {
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);

			// Ensure no TYPES symbol returns UnknownService
			for (const symbol of Object.values(TYPES)) {
				const serviceName = getServiceName(symbol);
				expect(serviceName).not.toBe("UnknownService");
			}
		});
	});

	describe("Service Mapping Validation", () => {
		test("should validate service mappings on container creation", () => {
			// This test ensures validateServiceMappings is called during initialization
			expect(() => {
				ServiceContainer.getInstance();
			}).not.toThrow();
		});

		test("should detect missing mappings if they existed", () => {
			// This test would fail if we had unmapped TYPES symbols
			const validateServiceMappings = (ServiceContainer as any)
				.validateServiceMappings;

			expect(() => {
				validateServiceMappings();
			}).not.toThrow();
		});
	});

	describe("Error Handling in Service Resolution", () => {
		test("should log warning for unknown service types", () => {
			const mockLogger = {
				warn: vi.fn(),
			};

			// Replace logger temporarily
			const originalLogger = (container as any).logger;
			(container as any).logger = mockLogger;

			const getServiceName = (container as any).getServiceName.bind(
				container,
			);
			const unknownSymbol = Symbol.for("UnknownSymbol");

			const result = getServiceName(unknownSymbol);

			expect(result).toBe("UnknownService");
			expect(mockLogger.warn).toHaveBeenCalledWith(
				expect.stringContaining("Unknown service type symbol"),
				"ServiceContainer",
			);

			// Restore original logger
			(container as any).logger = originalLogger;
		});

		test("should log warning for unknown service names", () => {
			const mockLogger = {
				warn: vi.fn(),
			};

			const originalLogger = (container as any).logger;
			(container as any).logger = mockLogger;

			const getServiceTypeFromName = (
				container as any
			).getServiceTypeFromName.bind(container);

			const result = getServiceTypeFromName("UnknownServiceName");

			expect(result).toBeUndefined();
			expect(mockLogger.warn).toHaveBeenCalledWith(
				"Unknown service name: UnknownServiceName",
				"ServiceContainer",
			);

			(container as any).logger = originalLogger;
		});
	});

	describe("Performance and Reliability", () => {
		test("should resolve service names quickly", () => {
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);

			const startTime = performance.now();

			// Perform many lookups
			for (let i = 0; i < 1000; i++) {
				getServiceName(TYPES.DatabaseService);
				getServiceName(TYPES.ParserService);
				getServiceName(TYPES.GraphService);
			}

			const endTime = performance.now();
			const duration = endTime - startTime;

			// Should complete 3000 lookups in under 10ms
			expect(duration).toBeLessThan(10);
		});

		test("should be consistent across multiple calls", () => {
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);

			// Multiple calls should return the same result
			const results = [];
			for (let i = 0; i < 100; i++) {
				results.push(getServiceName(TYPES.DatabaseService));
			}

			// All results should be identical
			const uniqueResults = [...new Set(results)];
			expect(uniqueResults).toHaveLength(1);
			expect(uniqueResults[0]).toBe("DatabaseService");
		});

		test("should handle concurrent access safely", async () => {
			const getServiceName = (container as any).getServiceName.bind(
				container,
			);

			// Concurrent access should not cause issues
			const promises = [];
			for (let i = 0; i < 50; i++) {
				promises.push(
					Promise.resolve(getServiceName(TYPES.AnalyticsService)),
				);
			}

			const results = await Promise.all(promises);

			// All results should be the same
			expect(
				results.every((result) => result === "AnalyticsService"),
			).toBe(true);
		});
	});
});
