/**
 * TestContainers setup verification test
 * This test validates the TestContainers infrastructure without requiring Docker
 */

import { describe, it, expect } from "vitest";

import { Logger } from "../../utils/logger";
import {
	TEST_CONFIG,
	canRunContainerTests,
	getTestTimeout,
} from "../config/test.config";

const logger = Logger.getInstance();

describe("TestContainers Setup Verification", () => {
	it("should have correct test configuration", () => {
		expect(TEST_CONFIG.NEO4J.IMAGE).toBe("neo4j:5.28");
		expect(TEST_CONFIG.NEO4J.USERNAME).toBe("neo4j");
		expect(TEST_CONFIG.NEO4J.PASSWORD).toBe("testpassword");
		expect(TEST_CONFIG.NEO4J.PORTS.BOLT).toBe(7687);
		expect(TEST_CONFIG.NEO4J.PORTS.HTTP).toBe(7474);
	});

	it("should detect container environment correctly", () => {
		const canRun = canRunContainerTests();
		const isCI = TEST_CONFIG.ENVIRONMENT.IS_CI;
		const skipContainers = TEST_CONFIG.ENVIRONMENT.SKIP_CONTAINERS;

		logger.info("Container tests environment:", "TestContainersSetup", {
			canRun,
			isCI,
			skipContainers,
			dockerHost: process.env.DOCKER_HOST,
			ci: process.env.CI,
		});

		expect(typeof canRun).toBe("boolean");
	});

	it("should provide appropriate timeouts", () => {
		const unitTimeout = getTestTimeout("unit");
		const integrationTimeout = getTestTimeout("integration");
		const e2eTimeout = getTestTimeout("e2e");

		expect(unitTimeout).toBeGreaterThan(0);
		expect(integrationTimeout).toBeGreaterThan(unitTimeout);
		expect(e2eTimeout).toBeGreaterThan(integrationTimeout);

		logger.info("Test timeouts:", "TestContainersSetup", {
			unit: unitTimeout,
			integration: integrationTimeout,
			e2e: e2eTimeout,
		});
	});

	it("should have sample test files available", async () => {
		const fs = await import("fs");
		const path = await import("path");

		const sampleJsDir = path.join(__dirname, "../test/sample-javascript");
		const samplePyDir = path.join(__dirname, "../test/sample-python");

		// Check if sample directories exist
		const jsExists = fs.existsSync(sampleJsDir);
		const pyExists = fs.existsSync(samplePyDir);

		logger.info("Sample files availability:", "TestContainersSetup", {
			javascriptDir: jsExists,
			pythonDir: pyExists,
			jsPath: sampleJsDir,
			pyPath: samplePyDir,
		});

		expect(jsExists || pyExists).toBe(true); // At least one should exist
	});

	it("should be able to import TestContainers modules", async () => {
		// Test that TestContainers can be imported
		const { GenericContainer } = await import("testcontainers");

		expect(GenericContainer).toBeDefined();
		expect(typeof GenericContainer).toBe("function");

		logger.info(
			"TestContainers modules imported successfully",
			"TestContainersSetup",
		);
	});

	it("should be able to import Neo4j driver", async () => {
		// Test that Neo4j driver can be imported
		const neo4j = await import("neo4j-driver");

		expect(neo4j.driver).toBeDefined();
		expect(neo4j.auth).toBeDefined();
		expect(typeof neo4j.driver).toBe("function");

		logger.info(
			"Neo4j driver imported successfully",
			"TestContainersSetup",
		);
	});
});

describe("TestContainers Infrastructure Classes", () => {
	it("should be able to import test infrastructure", async () => {
		// Test imports without instantiating (which requires Docker)
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

		logger.info(
			"TestContainer infrastructure classes imported successfully",
			"TestContainersSetup",
		);
	});

	it("should validate Neo4jTestContainer interface", async () => {
		const { Neo4jTestContainer } = await import(
			"../testcontainers/neo4j-container"
		);

		// Create instance without starting (won't require Docker)
		const container = new Neo4jTestContainer();

		// Check that methods exist
		expect(typeof container.start).toBe("function");
		expect(typeof container.stop).toBe("function");
		expect(typeof container.isRunning).toBe("function");
		expect(typeof container.getConnectionConfig).toBe("function");
		expect(typeof container.clearDatabase).toBe("function");
		expect(typeof container.setupSchema).toBe("function");

		// Should not be running initially
		expect(container.isRunning()).toBe(false);

		logger.info(
			"Neo4jTestContainer interface validated",
			"TestContainersSetup",
		);
	});
});
