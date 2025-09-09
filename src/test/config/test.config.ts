/**
 * Test configuration for TestContainers integration tests
 */

export const TEST_CONFIG = {
	// TestContainer timeouts
	CONTAINER_STARTUP_TIMEOUT: 120_000, // 2 minutes
	CONTAINER_STOP_TIMEOUT: 30_000, // 30 seconds
	TEST_TIMEOUT: 180_000, // 3 minutes for complex tests

	// Neo4j test configuration
	NEO4J: {
		IMAGE: "neo4j:5.28",
		USERNAME: "neo4j",
		PASSWORD: "testpassword",
		DATABASE: "neo4j",
		PORTS: {
			BOLT: 7687,
			HTTP: 7474,
		},
	},

	// Test data paths
	SAMPLE_FILES: {
		JAVASCRIPT_DIR: "../test/sample-javascript",
		PYTHON_DIR: "../test/sample-python",
		FILES: {
			SIMPLE_JS: "simple-javascript.js",
			SIMPLE_TS: "simple-typescript.ts",
			REACT_COMPONENT: "sample-react-component.tsx",
			PYTHON_CLASS: "sample_class.py",
			PYTHON_MODULE: "simple_module.py",
		},
	},

	// Performance test thresholds
	PERFORMANCE: {
		MAX_PARSE_TIME_MS: 5000, // 5 seconds per file
		MAX_BATCH_PARSE_TIME_MS: 15000, // 15 seconds for batch
		MAX_GRAPH_GENERATION_TIME_MS: 10000, // 10 seconds
		MAX_DATABASE_STORE_TIME_MS: 20000, // 20 seconds

		// Memory usage limits (in MB)
		MAX_MEMORY_USAGE_MB: 512,
		MAX_GRAPH_NODES: 10000,
		MAX_GRAPH_RELATIONSHIPS: 50000,
	},

	// Environment detection
	ENVIRONMENT: {
		// Check if we're in CI environment
		IS_CI: process.env.CI === "true",

		// Check if Docker is available
		HAS_DOCKER:
			process.env.TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE !== undefined ||
			process.env.DOCKER_HOST !== undefined,

		// Skip container tests if explicitly disabled
		SKIP_CONTAINERS: process.env.SKIP_CONTAINER_TESTS === "true",
	},

	// Logging configuration for tests
	LOGGING: {
		LEVEL: process.env.TEST_LOG_LEVEL || "info",
		ENABLE_CONTAINER_LOGS: process.env.ENABLE_CONTAINER_LOGS === "true",
		ENABLE_PERFORMANCE_LOGS: process.env.ENABLE_PERFORMANCE_LOGS === "true",
	},
};

/**
 * Check if TestContainers can run in current environment
 */
export function canRunContainerTests(): boolean {
	if (TEST_CONFIG.ENVIRONMENT.SKIP_CONTAINERS) {
		return false;
	}

	// In CI, assume Docker is available
	if (TEST_CONFIG.ENVIRONMENT.IS_CI) {
		return true;
	}

	// For local development, check for Docker
	try {
		// This is a simple check - TestContainers will do more thorough validation
		return TEST_CONFIG.ENVIRONMENT.HAS_DOCKER;
	} catch {
		return false;
	}
}

/**
 * Get appropriate test timeout based on environment
 */
export function getTestTimeout(
	testType: "unit" | "integration" | "e2e",
): number {
	const baseTimeouts = {
		unit: 10_000, // 10 seconds
		integration: 60_000, // 1 minute
		e2e: 180_000, // 3 minutes
	};

	// Increase timeouts in CI environment
	const multiplier = TEST_CONFIG.ENVIRONMENT.IS_CI ? 2 : 1;

	return baseTimeouts[testType] * multiplier;
}

/**
 * Create mock VS Code extension context for testing
 */
export function createTestExtensionContext() {
	return {
		extensionPath: "/test/extension/path",
		globalState: {
			get: (_key: string, defaultValue?: any) => defaultValue,
			update: async (_key: string, _value: any) => {},
			keys: () => [],
		},
		workspaceState: {
			get: (_key: string, defaultValue?: any) => defaultValue,
			update: async (_key: string, _value: any) => {},
			keys: () => [],
		},
		secrets: {
			get: async (_key: string) => undefined,
			store: async (_key: string, _value: string) => {},
			delete: async (_key: string) => {},
		},
		extensionUri: {
			scheme: "file",
			path: "/test/extension/path",
		},
	};
}
