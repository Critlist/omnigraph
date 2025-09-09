/**
 * Unit tests for TestContainers configuration (no Docker required)
 */

import { describe, it, expect } from "vitest";

import {
	TEST_CONFIG,
	canRunContainerTests,
	getTestTimeout,
	createTestExtensionContext,
} from "../config/test.config";

describe("TestContainers Configuration", () => {
	it("should have valid Neo4j configuration", () => {
		expect(TEST_CONFIG.NEO4J.IMAGE).toBe("neo4j:5.28");
		expect(TEST_CONFIG.NEO4J.USERNAME).toBe("neo4j");
		expect(TEST_CONFIG.NEO4J.PASSWORD).toBe("testpassword");
		expect(TEST_CONFIG.NEO4J.DATABASE).toBe("neo4j");
		expect(TEST_CONFIG.NEO4J.PORTS.BOLT).toBe(7687);
		expect(TEST_CONFIG.NEO4J.PORTS.HTTP).toBe(7474);
	});

	it("should have valid performance thresholds", () => {
		expect(TEST_CONFIG.PERFORMANCE.MAX_PARSE_TIME_MS).toBeGreaterThan(0);
		expect(TEST_CONFIG.PERFORMANCE.MAX_BATCH_PARSE_TIME_MS).toBeGreaterThan(
			TEST_CONFIG.PERFORMANCE.MAX_PARSE_TIME_MS,
		);
		expect(TEST_CONFIG.PERFORMANCE.MAX_MEMORY_USAGE_MB).toBeGreaterThan(0);
		expect(TEST_CONFIG.PERFORMANCE.MAX_GRAPH_NODES).toBeGreaterThan(0);
	});

	it("should detect environment correctly", () => {
		const canRun = canRunContainerTests();
		expect(typeof canRun).toBe("boolean");

		// Test environment variables
		expect(typeof TEST_CONFIG.ENVIRONMENT.IS_CI).toBe("boolean");
		expect(typeof TEST_CONFIG.ENVIRONMENT.SKIP_CONTAINERS).toBe("boolean");
	});

	it("should provide appropriate timeouts", () => {
		const unitTimeout = getTestTimeout("unit");
		const integrationTimeout = getTestTimeout("integration");
		const e2eTimeout = getTestTimeout("e2e");

		expect(unitTimeout).toBe(10_000);
		expect(integrationTimeout).toBe(60_000);
		expect(e2eTimeout).toBe(180_000);

		// Note: CI detection is static at module load time
		// This test validates the normal timeout behavior
		expect(e2eTimeout).toBe(180_000);
	});

	it("should create valid test extension context", () => {
		const context = createTestExtensionContext();

		expect(context.extensionPath).toBeTruthy();
		expect(context.globalState).toBeDefined();
		expect(context.workspaceState).toBeDefined();
		expect(context.secrets).toBeDefined();
		expect(context.extensionUri).toBeDefined();

		expect(typeof context.globalState.get).toBe("function");
		expect(typeof context.globalState.update).toBe("function");
		expect(typeof context.secrets.get).toBe("function");
		expect(typeof context.secrets.store).toBe("function");
	});
});

describe("TestContainers Module Imports", () => {
	it("should import TestContainers without errors", async () => {
		const { GenericContainer, Wait } = await import("testcontainers");

		expect(GenericContainer).toBeDefined();
		expect(Wait).toBeDefined();
		expect(typeof GenericContainer).toBe("function");
		expect(typeof Wait.forLogMessage).toBe("function");
	});

	it("should import Neo4j driver without errors", async () => {
		const neo4j = await import("neo4j-driver");

		expect(neo4j.driver).toBeDefined();
		expect(neo4j.auth).toBeDefined();
		expect(typeof neo4j.driver).toBe("function");
		expect(typeof neo4j.auth.basic).toBe("function");
	});

	it("should import test infrastructure classes", async () => {
		// Test that our classes can be imported
		try {
			const { Neo4jTestContainer } = await import(
				"../testcontainers/neo4j-container"
			);
			const { TestContainerBase } = await import(
				"../testcontainers/test-base"
			);

			expect(Neo4jTestContainer).toBeDefined();
			expect(TestContainerBase).toBeDefined();
			expect(typeof Neo4jTestContainer).toBe("function");
			expect(typeof TestContainerBase).toBe("function");
		} catch (error) {
			console.error("Failed to import test infrastructure:", error);
			throw error;
		}
	});
});

describe("Sample Files Availability", () => {
	it("should have sample JavaScript files", async () => {
		const fs = await import("fs");
		const path = await import("path");

		const sampleDir = path.join(__dirname, "../test/sample-javascript");
		const files = [
			"simple-javascript.js",
			"simple-typescript.ts",
			"sample-react-component.tsx",
		];

		expect(fs.existsSync(sampleDir)).toBe(true);

		for (const file of files) {
			const filePath = path.join(sampleDir, file);
			expect(fs.existsSync(filePath)).toBe(true);

			const content = fs.readFileSync(filePath, "utf-8");
			expect(content.length).toBeGreaterThan(0);
		}
	});

	it("should have sample Python files", async () => {
		const fs = await import("fs");
		const path = await import("path");

		const sampleDir = path.join(__dirname, "../test/sample-python");
		const files = ["sample_class.py", "simple_module.py"];

		expect(fs.existsSync(sampleDir)).toBe(true);

		for (const file of files) {
			const filePath = path.join(sampleDir, file);
			expect(fs.existsSync(filePath)).toBe(true);

			const content = fs.readFileSync(filePath, "utf-8");
			expect(content.length).toBeGreaterThan(0);
		}
	});
});
