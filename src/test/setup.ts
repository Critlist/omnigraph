/**
 * Test setup for TestContainers integration tests
 * Minimal setup - most configuration is handled by test classes
 */

import { beforeAll } from "vitest";

// Global test setup
beforeAll(() => {
	// Set test environment
	process.env.NODE_ENV = "test";

	// Enable test logging if needed
	if (process.env.TEST_LOG_LEVEL) {
		console.log("Test logging enabled:", process.env.TEST_LOG_LEVEL);
	}

	// Check for Docker availability
	if (process.env.SKIP_CONTAINER_TESTS === "true") {
		console.log("Container tests disabled via SKIP_CONTAINER_TESTS");
	}
});
