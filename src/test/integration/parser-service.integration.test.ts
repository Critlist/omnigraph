/**
 * Integration tests for ParserService with real files
 * Uses TestContainers for complete workflow testing
 */

import { readFileSync } from "fs";
import { join } from "path";

import { describe, it, expect } from "vitest";

import { ParserService } from "../../services/ParserService";
import { SupportedLanguage } from "../../types";
import { TestContainerBase } from "../testcontainers/test-base";

class ParserServiceIntegrationTest extends TestContainerBase {
	private parserService!: ParserService;

	protected async setupTestData(): Promise<void> {
		await super.setupTestData();
		this.parserService = new ParserService();
		// Initialize the parser service
		await this.parserService.init();
	}

	async testJavaScriptParsing() {
		this.checkContainersReady();

		// Read real sample file
		const filePath = join(
			__dirname,
			"../test/sample-javascript/simple-javascript.js",
		);
		const content = readFileSync(filePath, "utf-8");

		// Parse the file
		const result = await this.parserService.parseCode(
			content,
			SupportedLanguage.JAVASCRIPT,
			{},
			filePath,
		);

		// Validate parsing results
		expect(result.errors).toHaveLength(0);
		expect(result.root).toBeDefined();
		expect(result.root.type).toBe("file");
		expect(result.language).toBe(SupportedLanguage.JAVASCRIPT);
		expect(result.filePath).toBe(filePath);

		// Check for expected AST nodes
		expect(result.root.children.length).toBeGreaterThan(0);

		// Look for function declarations
		const functions = this.findNodesByType(result.root, "function");
		expect(functions.length).toBeGreaterThan(0);

		return { result, functions };
	}

	async testTypeScriptParsing() {
		this.checkContainersReady();

		// Read TypeScript sample file
		const filePath = join(
			__dirname,
			"../test/sample-javascript/simple-typescript.ts",
		);
		const content = readFileSync(filePath, "utf-8");

		// Parse the file
		const result = await this.parserService.parseCode(
			content,
			SupportedLanguage.TYPESCRIPT,
			{},
			filePath,
		);

		// Validate parsing results
		expect(result.errors).toHaveLength(0);
		expect(result.root).toBeDefined();
		expect(result.language).toBe(SupportedLanguage.TYPESCRIPT);

		// Check for TypeScript-specific constructs
		const interfaces = this.findNodesByType(result.root, "interface");
		const classes = this.findNodesByType(result.root, "class");

		expect(interfaces.length + classes.length).toBeGreaterThan(0);

		return { result, interfaces, classes };
	}

	async testReactComponentParsing() {
		this.checkContainersReady();

		// Read React component file
		const filePath = join(
			__dirname,
			"../test/sample-javascript/sample-react-component.tsx",
		);
		const content = readFileSync(filePath, "utf-8");

		// Parse the React component
		const result = await this.parserService.parseCode(
			content,
			SupportedLanguage.TYPESCRIPT,
			{},
			filePath,
		);

		// Validate parsing results
		expect(result.errors).toHaveLength(0);
		expect(result.root).toBeDefined();

		// Check for React-specific patterns
		const functions = this.findNodesByType(result.root, "function");
		const imports = this.findNodesByType(result.root, "import");

		expect(imports.length).toBeGreaterThan(0); // Should have React imports
		expect(functions.length).toBeGreaterThan(0); // Should have component function

		return { result, functions, imports };
	}

	async testBatchParsing() {
		this.checkContainersReady();

		// Prepare multiple files for batch parsing
		const files = [
			{
				filePath: join(
					__dirname,
					"../test/sample-javascript/simple-javascript.js",
				),
				content: readFileSync(
					join(
						__dirname,
						"../test/sample-javascript/simple-javascript.js",
					),
					"utf-8",
				),
			},
			{
				filePath: join(
					__dirname,
					"../test/sample-javascript/simple-typescript.ts",
				),
				content: readFileSync(
					join(
						__dirname,
						"../test/sample-javascript/simple-typescript.ts",
					),
					"utf-8",
				),
			},
		];

		// Test batch parsing
		const results = await this.parserService.parseBatch(files);

		// Validate batch results
		expect(results).toHaveLength(2);

		for (const result of results) {
			expect(result.errors).toHaveLength(0);
			expect(result.root).toBeDefined();
			expect(result.root.children.length).toBeGreaterThan(0);
		}

		return results;
	}

	private findNodesByType(node: any, type: string): any[] {
		const results: any[] = [];

		if (node.type === type) {
			results.push(node);
		}

		if (node.children) {
			for (const child of node.children) {
				results.push(...this.findNodesByType(child, type));
			}
		}

		return results;
	}
}

describe("ParserService Integration Tests", () => {
	const testInstance = new ParserServiceIntegrationTest();

	it("should parse JavaScript files correctly", async () => {
		const { result, functions } =
			await testInstance.testJavaScriptParsing();

		expect(result.sourceCode).toContain("function");
		expect(functions[0].name).toBeTruthy();
	});

	it("should parse TypeScript files correctly", async () => {
		const { result, interfaces, classes } =
			await testInstance.testTypeScriptParsing();

		expect(result.sourceCode).toMatch(/interface|class/);
		expect(interfaces.length + classes.length).toBeGreaterThan(0);
	});

	it("should parse React components correctly", async () => {
		const { functions, imports } =
			await testInstance.testReactComponentParsing();

		expect(imports.some((imp) => imp.source?.includes("react"))).toBe(true);
		expect(functions.length).toBeGreaterThan(0);
	});

	it("should handle batch parsing efficiently", async () => {
		const results = await testInstance.testBatchParsing();

		// All files should parse successfully
		expect(results.every((r) => r.errors.length === 0)).toBe(true);

		// Should have different file paths
		const filePaths = results.map((r) => r.filePath);
		expect(new Set(filePaths).size).toBe(results.length);
	});

	it("should validate syntax correctly", async () => {
		testInstance["checkContainersReady"]();

		const validCode = "const x = 1; function test() { return x; }";
		const invalidCode = "const x = ; function test( { return x; }";

		const validResult = await testInstance["parserService"].validateSyntax(
			validCode,
			SupportedLanguage.JAVASCRIPT,
		);

		const invalidResult = await testInstance[
			"parserService"
		].validateSyntax(invalidCode, SupportedLanguage.JAVASCRIPT);

		expect(validResult).toBe(true);
		expect(invalidResult).toBe(false);
	});

	it("should extract dependencies correctly", async () => {
		testInstance["checkContainersReady"]();

		const codeWithImports = `
            import React from 'react';
            import { useState } from 'react';
            import utils from './utils';
            
            export function Component() {
                return <div>Hello</div>;
            }
        `;

		const parseResult = await testInstance["parserService"].parseCode(
			codeWithImports,
			SupportedLanguage.TYPESCRIPT,
		);

		const dependencies = await testInstance[
			"parserService"
		].extractDependencies(parseResult.root);

		expect(dependencies.imports.length).toBeGreaterThan(0);
		expect(dependencies.dependencies).toContain("react");
		expect(dependencies.dependencies).toContain("./utils");
		expect(dependencies.exports.length).toBeGreaterThan(0);
	});
});
