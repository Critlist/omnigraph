/**
 * End-to-end integration tests for graph generation workflow
 * Tests: File Parsing → Graph Generation → Neo4j Storage → Query Validation
 */

import { readFileSync } from "fs";
import { join } from "path";

import { describe, it, expect } from "vitest";

import { TYPES } from "../../config/types";
import { ServiceContainer } from "../../container/container";
import { GraphGenerator } from "../../graph/graph-generator";
import { DatabaseService } from "../../services/DatabaseService";
import { ParserService } from "../../services/ParserService";
import { SupportedLanguage } from "../../types";
import { TestContainerBase } from "../testcontainers/test-base";

class GraphGenerationIntegrationTest extends TestContainerBase {
	private serviceContainer!: ServiceContainer;
	private parserService!: ParserService;
	private databaseService!: DatabaseService;
	private graphGenerator!: GraphGenerator;

	protected async setupTestData(): Promise<void> {
		await super.setupTestData();

		// Initialize service container with test configuration
		this.serviceContainer = ServiceContainer.getInstance();
		await this.serviceContainer.initialize();

		// Configure services to use test Neo4j container
		const neo4jConfig = this.getNeo4jConfig();

		// Mock VS Code context for services that need it
		this.createMockVSCodeContext();

		// Initialize services
		this.parserService = this.serviceContainer.get<ParserService>(
			TYPES.ParserService,
		);
		await this.parserService.init();

		this.databaseService = this.serviceContainer.get<DatabaseService>(
			TYPES.DatabaseService,
		);
		// Configure database service with test container settings
		await this.databaseService.init();
		await this.databaseService.connect(neo4jConfig);

		this.graphGenerator = new GraphGenerator();
	}

	protected async cleanupTestData(): Promise<void> {
		// Cleanup services
		if (this.databaseService?.isReady()) {
			await this.databaseService.dispose();
		}
		if (this.parserService?.isReady()) {
			await this.parserService.dispose();
		}

		await super.cleanupTestData();
	}

	async testCompleteWorkflow() {
		this.checkContainersReady();

		// Step 1: Parse sample files
		const jsFilePath = join(
			__dirname,
			"../test/sample-javascript/simple-javascript.js",
		);
		const tsFilePath = join(
			__dirname,
			"../test/sample-javascript/simple-typescript.ts",
		);

		const files = [
			{
				filePath: jsFilePath,
				content: readFileSync(jsFilePath, "utf-8"),
			},
			{
				filePath: tsFilePath,
				content: readFileSync(tsFilePath, "utf-8"),
			},
		];

		// Step 2: Batch parse files
		const parseResults = await this.parserService.parseBatch(files);
		expect(parseResults).toHaveLength(2);
		expect(parseResults.every((r) => r.errors.length === 0)).toBe(true);

		// Step 3: Convert to ParsedFile format for GraphGenerator
		const parsedFiles = parseResults.map((result) => ({
			filePath: result.filePath,
			language: result.language as SupportedLanguage,
			nodes: this.flattenAST(result.root),
			relationships: [], // Will be extracted by GraphGenerator
			parseTime: new Date(),
			success: result.errors.length === 0,
			errors: result.errors.map((e) => e.message),
		}));

		// Step 4: Generate graph
		const graphResult =
			await this.graphGenerator.generateGraph(parsedFiles);
		expect(graphResult.isOk()).toBe(true);

		if (!graphResult.isOk()) {
			throw new Error(
				`Graph generation failed: ${graphResult.error.message}`,
			);
		}

		const { graph: _graph } = graphResult.value;

		// Validate graph structure
		expect(_graph.nodes.length).toBeGreaterThan(0);
		expect(_graph.relationships.length).toBeGreaterThan(0);
		expect(_graph.metadata.totalFiles).toBe(2);

		// Step 5: Store graph in Neo4j
		await this.storeGraphInNeo4j(_graph);

		// Step 6: Validate data in Neo4j
		await this.validateStoredGraph(_graph);

		return { parseResults, graph: _graph };
	}

	async testJavaScriptProjectWorkflow() {
		this.checkContainersReady();

		// Parse all JavaScript sample files
		const sampleDir = join(__dirname, "../test/sample-javascript");
		const jsFile = join(sampleDir, "simple-javascript.js");
		const tsFile = join(sampleDir, "simple-typescript.ts");
		const reactFile = join(sampleDir, "sample-react-component.tsx");

		const files = [
			{ filePath: jsFile, content: readFileSync(jsFile, "utf-8") },
			{ filePath: tsFile, content: readFileSync(tsFile, "utf-8") },
			{ filePath: reactFile, content: readFileSync(reactFile, "utf-8") },
		];

		// Complete workflow
		const parseResults = await this.parserService.parseBatch(files);
		const parsedFiles = parseResults.map((result) => ({
			filePath: result.filePath,
			language: result.language as SupportedLanguage,
			nodes: this.flattenAST(result.root),
			relationships: [],
			parseTime: new Date(),
			success: result.errors.length === 0,
			errors: result.errors.map((e) => e.message),
		}));

		const graphResult =
			await this.graphGenerator.generateGraph(parsedFiles);
		expect(graphResult.isOk()).toBe(true);

		if (graphResult.isOk()) {
			const { graph: _graph } = graphResult.value;

			// Should have nodes for files, functions, classes, imports
			const fileNodes = _graph.nodes.filter((n: any) =>
				n.labels?.includes("File"),
			);
			const functionNodes = _graph.nodes.filter((n: any) =>
				n.labels?.includes("Function"),
			);
			const importNodes = _graph.nodes.filter((n: any) =>
				n.labels?.includes("Import"),
			);

			expect(fileNodes).toHaveLength(3);
			expect(functionNodes.length).toBeGreaterThan(0);
			expect(importNodes.length).toBeGreaterThan(0);

			return { parseResults, graph: _graph };
		}

		throw new Error("Graph generation failed");
	}

	private async storeGraphInNeo4j(graph: any): Promise<void> {
		const session = this["neo4jContainer"].getSession();

		try {
			// Store nodes
			for (const node of graph.nodes) {
				const labels = node.labels?.join(":") || "Node";
				const properties = { ...node.properties };

				await session.run(`CREATE (n:${labels}) SET n = $properties`, {
					properties,
				});
			}

			// Store relationships
			for (const rel of graph.relationships) {
				await session.run(
					`
                    MATCH (source {id: $sourceId})
                    MATCH (target {id: $targetId})
                    CREATE (source)-[r:${rel.type}]->(target)
                    SET r = $properties
                `,
					{
						sourceId: rel.source,
						targetId: rel.target,
						properties: rel.properties || {},
					},
				);
			}
		} finally {
			await session.close();
		}
	}

	private async validateStoredGraph(expectedGraph: any): Promise<void> {
		const session = this["neo4jContainer"].getSession();

		try {
			// Validate node count
			const nodeCountResult = await session.run(
				"MATCH (n) RETURN count(n) as count",
			);
			const actualNodeCount = nodeCountResult.records[0]
				.get("count")
				.toNumber();
			expect(actualNodeCount).toBe(expectedGraph.nodes.length);

			// Validate relationship count
			const relCountResult = await session.run(
				"MATCH ()-[r]->() RETURN count(r) as count",
			);
			const actualRelCount = relCountResult.records[0]
				.get("count")
				.toNumber();
			expect(actualRelCount).toBe(expectedGraph.relationships.length);

			// Validate specific node types exist
			const fileNodesResult = await session.run(
				"MATCH (n:File) RETURN count(n) as count",
			);
			const fileNodeCount = fileNodesResult.records[0]
				.get("count")
				.toNumber();
			expect(fileNodeCount).toBeGreaterThan(0);

			// Test queries that the application would use
			const functionsResult = await session.run(`
                MATCH (f:Function)
                RETURN f.name as name, f.filePath as filePath
                LIMIT 10
            `);
			expect(functionsResult.records.length).toBeGreaterThan(0);
		} finally {
			await session.close();
		}
	}

	private flattenAST(rootNode: any): any[] {
		const nodes: any[] = [];

		const traverse = (node: any) => {
			nodes.push(node);
			if (node.children) {
				for (const child of node.children) {
					traverse(child);
				}
			}
		};

		traverse(rootNode);
		return nodes;
	}
}

describe("Graph Generation Integration Tests", () => {
	const testInstance = new GraphGenerationIntegrationTest();

	it("should complete full workflow: Parse → Generate → Store → Query", async () => {
		const { parseResults, graph } =
			await testInstance.testCompleteWorkflow();

		// Validate end-to-end results
		expect(parseResults.length).toBe(2);
		expect(graph.nodes.length).toBeGreaterThan(parseResults.length); // Should have more nodes than files
		expect(graph.relationships.length).toBeGreaterThan(0);

		// Metadata should be correct
		expect(graph.metadata.totalFiles).toBe(2);
		expect(graph.metadata.languages).toContain(
			SupportedLanguage.JAVASCRIPT,
		);
		expect(graph.metadata.languages).toContain(
			SupportedLanguage.TYPESCRIPT,
		);
	}, 180_000); // 3 minute timeout for full workflow

	it("should handle JavaScript/TypeScript/React project", async () => {
		const { parseResults, graph } =
			await testInstance.testJavaScriptProjectWorkflow();

		// Should parse all file types successfully
		expect(parseResults.length).toBe(3);
		expect(parseResults.every((r) => r.errors.length === 0)).toBe(true);

		// Graph should represent the project structure
		const fileNodes = graph.nodes.filter((n: any) =>
			n.labels?.includes("File"),
		);
		expect(fileNodes).toHaveLength(3);

		// Should have cross-file relationships
		expect(graph.relationships.length).toBeGreaterThan(fileNodes.length);
	}, 180_000);

	it("should persist and query graph data correctly", async () => {
		const { graph } = await testInstance.testCompleteWorkflow();

		// Verify we can query the stored data
		const session = testInstance["neo4jContainer"].getSession();

		try {
			// Test complex queries
			const complexQueryResult = await session.run(`
                MATCH (file:File)-[:CONTAINS]->(func:Function)
                WHERE func.name IS NOT NULL
                RETURN file.filePath as filePath,
                       func.name as functionName,
                       func.startLine as startLine
                ORDER BY file.filePath, func.startLine
            `);

			expect(complexQueryResult.records.length).toBeGreaterThan(0);

			// Verify data integrity
			for (const record of complexQueryResult.records) {
				expect(record.get("filePath")).toBeTruthy();
				expect(record.get("functionName")).toBeTruthy();
				expect(record.get("startLine")).toBeGreaterThan(0);
			}
		} finally {
			await session.close();
		}
	}, 120_000);
});
