/**
 * Neo4j TestContainer setup for integration testing
 * Provides a real Neo4j instance for testing graph operations
 */

import { Driver, auth, Session } from "neo4j-driver";
import * as neo4j from "neo4j-driver";
import { GenericContainer, StartedTestContainer, Wait } from "testcontainers";

import { Logger } from "../../utils/logger";

export interface Neo4jTestConfig {
	username: string;
	password: string;
	database: string;
	port: number;
}

export class Neo4jTestContainer {
	private container?: StartedTestContainer;
	private driver?: Driver;
	private logger = Logger.getInstance();

	private readonly config: Neo4jTestConfig = {
		username: "neo4j",
		password: "testpassword",
		database: "neo4j",
		port: 7687,
	};

	/**
	 * Start a Neo4j container for testing
	 */
	async start(): Promise<void> {
		this.logger.info("Starting Neo4j TestContainer", "Neo4jTestContainer");

		try {
			this.container = await new GenericContainer("neo4j:5.28")
				.withEnvironment({
					NEO4J_AUTH: `${this.config.username}/${this.config.password}`,
					NEO4J_PLUGINS: '["apoc"]', // Include APOC for advanced operations
					NEO4J_dbms_security_procedures_unrestricted: "apoc.*",
					NEO4J_apoc_export_file_enabled: "true",
					NEO4J_apoc_import_file_enabled: "true",
				})
				.withExposedPorts(7687, 7474) // Bolt and HTTP ports
				.withWaitStrategy(
					Wait.forLogMessage("Started.", 1).withStartupTimeout(
						60_000,
					), // 60 seconds timeout
				)
				.withReuse() // Ensure fresh container each time
				.withAutoRemove(true) // Automatically remove when stopped
				.start();

			this.logger.info(
				"Neo4j container started successfully",
				"Neo4jTestContainer",
				{
					containerId: this.container.getId(),
					mappedPort: this.container.getMappedPort(7687),
				},
			);

			// Initialize driver
			await this.initializeDriver();
		} catch (error) {
			this.logger.error(
				"Failed to start Neo4j container",
				error as Error,
				"Neo4jTestContainer",
			);
			throw error;
		}
	}

	/**
	 * Initialize Neo4j driver connection
	 */
	private async initializeDriver(): Promise<void> {
		if (!this.container) {
			throw new Error("Container not started");
		}

		const host = this.container.getHost();
		const port = this.container.getMappedPort(7687);
		const uri = `bolt://${host}:${port}`;

		this.driver = neo4j.driver(
			uri,
			auth.basic(this.config.username, this.config.password),
			{
				connectionTimeout: 30000,
				maxConnectionLifetime: 30000,
				maxConnectionPoolSize: 10,
			},
		);

		// Verify connection
		const session = this.driver.session({ database: this.config.database });
		try {
			await session.run("RETURN 1 as test");
			this.logger.info(
				"Neo4j driver connection verified",
				"Neo4jTestContainer",
			);
		} finally {
			await session.close();
		}
	}

	/**
	 * Get a Neo4j session for testing
	 */
	getSession(): Session {
		if (!this.driver) {
			throw new Error("Driver not initialized. Call start() first.");
		}
		return this.driver.session({ database: this.config.database });
	}

	/**
	 * Get the Neo4j driver
	 */
	getDriver(): Driver {
		if (!this.driver) {
			throw new Error("Driver not initialized. Call start() first.");
		}
		return this.driver;
	}

	/**
	 * Get connection configuration for services
	 */
	getConnectionConfig() {
		if (!this.container) {
			throw new Error("Container not started");
		}

		return {
			uri: `bolt://${this.container.getHost()}:${this.container.getMappedPort(7687)}`,
			username: this.config.username,
			password: this.config.password,
			database: this.config.database,
		};
	}

	/**
	 * Clear all data from the database
	 */
	async clearDatabase(): Promise<void> {
		const session = this.getSession();
		try {
			// Delete all nodes and relationships
			await session.run("MATCH (n) DETACH DELETE n");
			this.logger.debug("Database cleared", "Neo4jTestContainer");
		} finally {
			await session.close();
		}
	}

	/**
	 * Create basic schema and indexes
	 */
	async setupSchema(): Promise<void> {
		const session = this.getSession();
		try {
			// Create indexes for common node types
			const indexQueries = [
				"CREATE INDEX IF NOT EXISTS FOR (n:File) ON (n.filePath)",
				"CREATE INDEX IF NOT EXISTS FOR (n:Class) ON (n.name)",
				"CREATE INDEX IF NOT EXISTS FOR (n:Function) ON (n.name)",
				"CREATE INDEX IF NOT EXISTS FOR (n:Variable) ON (n.name)",
				"CREATE INDEX IF NOT EXISTS FOR (n:Import) ON (n.source)",
				"CREATE CONSTRAINT IF NOT EXISTS FOR (n:File) REQUIRE n.filePath IS UNIQUE",
			];

			for (const query of indexQueries) {
				await session.run(query);
			}

			this.logger.info(
				"Database schema setup completed",
				"Neo4jTestContainer",
			);
		} finally {
			await session.close();
		}
	}

	/**
	 * Stop and cleanup the container
	 */
	async stop(): Promise<void> {
		this.logger.info("Stopping Neo4j TestContainer", "Neo4jTestContainer");

		if (this.driver) {
			await this.driver.close();
			this.driver = undefined;
		}

		if (this.container) {
			await this.container.stop();
			this.container = undefined;
		}

		this.logger.info("Neo4j TestContainer stopped", "Neo4jTestContainer");
	}

	/**
	 * Check if container is running
	 */
	isRunning(): boolean {
		return this.container !== undefined && this.driver !== undefined;
	}

	/**
	 * Get container logs for debugging
	 */
	async getLogs(): Promise<string> {
		if (!this.container) {
			throw new Error("Container not started");
		}

		const logs = await this.container.logs();
		return logs.toString();
	}
}
