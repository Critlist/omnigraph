import { resolve } from "path";
import { defineConfig } from "vitest/config";

export default defineConfig({
	test: {
		// Test environment - use jsdom for frontend tests
		environment: "jsdom",
		
		// Test pattern matching - updated for Tauri structure
		include: [
			"../../src/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}",
			"../../tests/**/*.{test,spec}.{js,mjs,cjs,ts,mts,cts,jsx,tsx}"
		],
		exclude: [
			"../../node_modules",
			"../../dist",
			"../../src-tauri",
			"../../target"
		],
		
		// Global test settings
		globals: true,
		
		// Coverage configuration
		coverage: {
			provider: "v8",
			reporter: ["text", "json", "html"],
			reportsDirectory: "../../coverage",
			exclude: [
				"node_modules/",
				"dist/",
				"src-tauri/",
				"target/",
				"src/**/*.d.ts",
				"tests/**",
				"**/*.test.ts",
				"**/*.spec.ts",
				"src/engine/**", // Legacy TS engine
			],
			thresholds: {
				global: {
					branches: 60,
					functions: 60,
					lines: 60,
					statements: 60,
				},
			},
		},
		
		// Timeout settings
		testTimeout: 10000,
		hookTimeout: 10000,
		
		// Setup files (if they exist)
		setupFiles: ["../../tests/setup.ts"],
	},
	
	// Module resolution - match main vite config
	resolve: {
		alias: {
			"@": resolve(__dirname, "../../src"),
			"@visualization": resolve(__dirname, "../../src/visualization"),
			"@engine": resolve(__dirname, "../../src/engine"),
			"@types": resolve(__dirname, "../../src/types"),
		},
	},
	
	// Define globals for better TypeScript support
	define: {
		__TEST__: true,
		"import.meta.vitest": "undefined",
	},
}); 