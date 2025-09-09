/**
 * Base class for TestContainers integration tests
 * Provides common setup and teardown for Neo4j container tests
 */

import { beforeAll, afterAll, beforeEach, afterEach } from "vitest";

import { Logger } from "../../utils/logger";

import { Neo4jTestContainer } from "./neo4j-container";

export abstract class TestContainerBase {
	protected neo4jContainer: Neo4jTestContainer;
	protected logger = Logger.getInstance();

	constructor() {
		this.neo4jContainer = new Neo4jTestContainer();
		this.setupHooks();
	}

	/**
	 * Setup test hooks for container lifecycle
	 */
	private setupHooks(): void {
		beforeAll(async () => {
			await this.startContainers();
		}, 120_000); // 2 minute timeout for container startup

		afterAll(async () => {
			await this.stopContainers();
		}, 30_000); // 30 second timeout for cleanup

		beforeEach(async () => {
			await this.setupTestData();
		});

		afterEach(async () => {
			await this.cleanupTestData();
		});
	}

	/**
	 * Start all required containers
	 */
	protected async startContainers(): Promise<void> {
		this.logger.info("Starting test containers", "TestContainerBase");

		try {
			await this.neo4jContainer.start();
			await this.neo4jContainer.setupSchema();
			this.logger.info(
				"All test containers started successfully",
				"TestContainerBase",
			);
		} catch (error) {
			this.logger.error(
				"Failed to start test containers",
				error as Error,
				"TestContainerBase",
			);

			// If Docker is not available, log helpful message
			if (error instanceof Error && error.message.includes("Docker")) {
				this.logger.warn(
					"Docker not available. To run TestContainer tests, install Docker: https://docs.docker.com/get-docker/",
					"TestContainerBase",
				);
			}

			throw error;
		}
	}

	/**
	 * Stop all containers
	 */
	protected async stopContainers(): Promise<void> {
		this.logger.info("Stopping test containers", "TestContainerBase");

		try {
			await this.neo4jContainer.stop();
			this.logger.info(
				"All test containers stopped",
				"TestContainerBase",
			);
		} catch (error) {
			this.logger.error(
				"Error stopping test containers",
				error as Error,
				"TestContainerBase",
			);
			// Don't throw here - we want cleanup to continue
		}
	}

	/**
	 * Setup test data before each test
	 * Override in subclasses for specific test data needs
	 */
	protected async setupTestData(): Promise<void> {
		// Default: ensure clean database
		await this.neo4jContainer.clearDatabase();
	}

	/**
	 * Cleanup test data after each test
	 * Override in subclasses for specific cleanup needs
	 */
	protected async cleanupTestData(): Promise<void> {
		// Default: clear database
		await this.neo4jContainer.clearDatabase();
	}

	/**
	 * Get Neo4j connection configuration for services
	 */
	protected getNeo4jConfig() {
		return this.neo4jContainer.getConnectionConfig();
	}

	/**
	 * Helper to check if containers are available
	 */
	protected checkContainersReady(): void {
		if (!this.neo4jContainer.isRunning()) {
			throw new Error(
				"Test containers not ready. Ensure Docker is running and containers started successfully.",
			);
		}
	}

	/**
	 * Helper to create a mock VS Code context for testing
	 */
	protected createMockVSCodeContext() {
		return {
			extensionPath: "/mock/extension/path",
			globalState: {
				get: (_key: string) => undefined,
				update: async (_key: string, _value: any) => {},
			},
			secrets: {
				get: async (_key: string) => undefined,
				store: async (_key: string, _value: string) => {},
			},
		};
	}
}
