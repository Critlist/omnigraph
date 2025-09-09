# TestContainers Integration Testing

This directory contains integration tests using TestContainers for real-world testing with actual Neo4j databases and file parsing.

## Architecture

### 🐳 TestContainers Infrastructure

- **`testcontainers/neo4j-container.ts`** - Neo4j container management
- **`testcontainers/test-base.ts`** - Base class for container tests
- **`config/test.config.ts`** - Test configuration and environment detection

### 🧪 Integration Tests

- **`integration/parser-service.integration.test.ts`** - Parser tests with real files
- **`integration/graph-generation.integration.test.ts`** - End-to-end workflow tests
- **`integration/testcontainers-setup.test.ts`** - Setup verification tests

### 📁 Test Assets

- **`test/sample-javascript/`** - Real JavaScript/TypeScript/React files
- **`test/sample-python/`** - Real Python modules and classes

## Prerequisites

### Docker Installation

```bash
# Install Docker (if not already installed)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Start Docker daemon
sudo systemctl start docker
sudo systemctl enable docker

# Add user to docker group (optional, avoids sudo)
sudo usermod -aG docker $USER
```

### Dependencies

TestContainers dependencies are already installed:

```bash
pnpm install  # Includes testcontainers package
```

## Running Tests

### Quick Setup Verification

```bash
# Test that TestContainers infrastructure is working
pnpm test:integration testcontainers-setup.test.ts
```

### Integration Tests

```bash
# Run all integration tests (requires Docker)
pnpm test:integration

# Run with verbose output
pnpm test:containers

# Run with UI
pnpm test:containers:ui
```

### Manual Container Management

```bash
# Start Neo4j container manually for debugging
pnpm docker:test:up

# View container logs
pnpm docker:test:logs

# Stop containers (removes volumes too)
pnpm docker:test:down

# Clean up ALL TestContainers and related resources
pnpm docker:cleanup

# Nuclear option: remove ALL Docker resources (use with caution!)
pnpm docker:nuke
```

## Test Structure

### Basic Integration Test Pattern

```typescript
import { TestContainerBase } from "../testcontainers/test-base";

class MyIntegrationTest extends TestContainerBase {
	async testSomething() {
		this.checkContainersReady();

		// Use this.neo4jContainer for database operations
		const session = this.neo4jContainer.getSession();

		// Test your functionality
		// ...

		await session.close();
	}
}

describe("My Integration Tests", () => {
	const testInstance = new MyIntegrationTest();

	it("should do something", async () => {
		await testInstance.testSomething();
	});
});
```

### End-to-End Test Pattern

```typescript
// Complete workflow: Parse → Generate → Store → Query
async testCompleteWorkflow() {
    // 1. Parse real sample files
    const parseResults = await this.parserService.parseBatch(files);

    // 2. Generate graph
    const graphResult = await this.graphGenerator.generateGraph(parsedFiles);

    // 3. Store in Neo4j
    await this.storeGraphInNeo4j(graph);

    // 4. Validate with queries
    await this.validateStoredGraph(graph);
}
```

## Environment Configuration

### Environment Variables

```bash
# Skip container tests (useful for CI without Docker)
export SKIP_CONTAINER_TESTS=true

# Enable verbose container logging
export ENABLE_CONTAINER_LOGS=true

# Set test log level
export TEST_LOG_LEVEL=debug

# CI environment detection
export CI=true
```

### Docker Configuration

```bash
# Use custom Docker socket (if needed)
export DOCKER_HOST=unix:///var/run/docker.sock

# TestContainers configuration
export TESTCONTAINERS_DOCKER_SOCKET_OVERRIDE=/var/run/docker.sock
```

## Performance Expectations

### Test Timeouts

- **Unit tests**: 10 seconds
- **Integration tests**: 60 seconds (2x in CI)
- **End-to-end tests**: 180 seconds (2x in CI)

### Container Startup

- **Neo4j container**: ~30-60 seconds
- **Schema setup**: ~5 seconds
- **Test cleanup**: ~10 seconds

### Performance Thresholds

- **File parsing**: < 5 seconds per file
- **Batch parsing**: < 15 seconds for multiple files
- **Graph generation**: < 10 seconds
- **Database operations**: < 20 seconds

## Troubleshooting

### Docker Issues

```bash
# Check Docker daemon status
docker info

# Test Docker connectivity
docker run hello-world

# Check available images
docker images

# View running containers
docker ps
```

### Container Startup Issues

```bash
# View container logs
pnpm docker:test:logs

# Connect to container manually
docker exec -it omnigraph-neo4j-test cypher-shell -u neo4j -p testpassword

# Check container health
docker inspect omnigraph-neo4j-test | grep Health -A 10
```

### Test Failures

```bash
# Run with verbose logging
ENABLE_CONTAINER_LOGS=true pnpm test:containers

# Run single test file
pnpm test:integration parser-service.integration.test.ts

# Debug with UI
pnpm test:containers:ui
```

## Benefits of TestContainers

### ✅ Real Integration Testing

- Tests actual Neo4j database interactions
- Validates real file parsing with sample code
- Catches integration issues early

### ✅ Isolated Test Environment

- Fresh database for each test
- No test pollution between runs
- Reproducible test conditions

### ✅ Production-Like Testing

- Same Neo4j version as production
- Real APOC procedures and plugins
- Actual Cypher query validation

### ✅ CI/CD Ready

- Automatic container lifecycle management
- Configurable timeouts and environments
- Graceful fallback when Docker unavailable

## Migration from Mocks

### Before (Mock-based)

```typescript
// ❌ Fake, unreliable
const mockParser = {
	parseFile: vi.fn().mockResolvedValue(fakeResult),
};
```

### After (TestContainers)

```typescript
// ✅ Real, reliable
const parseResult = await this.parserService.parseFile(realFilePath);
const session = this.neo4jContainer.getSession();
await session.run("CREATE (n:File {path: $path})", { path: realFilePath });
```

This approach provides **real confidence** in the system's functionality!
