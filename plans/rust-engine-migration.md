# Rust Engine Migration Plan

## Overview

This document outlines the comprehensive plan to migrate all TypeScript engine features to Rust, maintaining similar separation of concerns (SOC) and architectural patterns.

## Current TypeScript Engine Architecture

### Core Modules Identified

1. **Types & Interfaces** (`types/`)
   - Core AST node types and relationships
   - Graph structures (Neo4j compatible)
   - Visualization data structures
   - Parser plugin interfaces

2. **Parser System** (`parser/`)
   - Base parser abstract class
   - Language-specific parsers (JavaScript, TypeScript, Python)
   - Parser registry and plugin system
   - Grammar loader for tree-sitter

3. **Database Layer** (`database/`)
   - Neo4j connection management
   - Cypher query builder
   - Data ingestion pipeline
   - Database types and interfaces

4. **Services Layer** (`services/`)
   - Parser Service
   - Database Service
   - Graph Service
   - Analytics Service
   - Visualization Service
   - Service lifecycle management

5. **Graph Processing** (`graph/`)
   - Graph generator
   - Graph optimizer
   - Graph schema validation
   - Processing pipeline with steps

6. **Analytics** (`analytics/`)
   - Graph analytics algorithms
   - GDS (Graph Data Science) algorithms
   - Result processor
   - Analytics queries

7. **Dependency Injection** (`container/`)
   - Module system for IoC
   - Lazy service proxy
   - Performance metrics
   - Service registration

8. **Utilities** (`utils/`)
   - Result type (Ok/Err pattern)
   - Logger
   - Resource manager
   - Initialization mutex

## Rust Implementation Plan

### Phase 1: Core Types & Abstractions

**Location**: `src-tauri/src/engine/types/`

```rust
// mod.rs - Core type definitions
pub mod ast;        // AST node types
pub mod graph;      // Graph structures
pub mod viz;        // Visualization types
pub mod error;      // Error handling

// ast.rs
pub enum NodeType {
    File, Module, Class, Interface,
    Function, Method, Variable, Property,
    Import, Export, Decorator, Comment
}

pub struct AstNode {
    id: String,
    node_type: NodeType,
    name: String,
    file_path: PathBuf,
    location: SourceLocation,
    metadata: HashMap<String, Value>,
}

pub enum RelationshipType {
    Contains, Calls, Imports, Exports,
    Extends, Implements, References,
    Decorates, TypedBy, Returns, Parameter
}

// graph.rs
pub struct CodeGraph {
    nodes: Vec<GraphNode>,
    relationships: Vec<GraphRelationship>,
    metadata: GraphMetadata,
}

// error.rs - Result type pattern
pub type EngineResult<T> = Result<T, EngineError>;
```

### Phase 2: Parser Plugin System

**Location**: `src-tauri/src/engine/parser/`

```rust
// traits.rs - Parser plugin traits
pub trait ParserPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn supported_extensions(&self) -> &[&str];
    fn can_parse(&self, path: &Path) -> bool;
    
    async fn parse_file(
        &self,
        path: &Path,
        content: &str,
    ) -> EngineResult<ParsedFile>;
    
    async fn parse_batch(
        &self,
        files: Vec<(PathBuf, String)>,
    ) -> EngineResult<Vec<ParsedFile>>;
}

// base.rs - Base parser implementation
pub struct BaseParser {
    parser: Parser,
    language: Language,
    node_id_gen: AtomicUsize,
}

impl BaseParser {
    pub fn extract_nodes(&self, tree: &Tree) -> EngineResult<Vec<AstNode>>;
    pub fn extract_relationships(&self, nodes: &[AstNode]) -> EngineResult<Vec<Relationship>>;
}

// registry.rs - Parser registry
pub struct ParserRegistry {
    parsers: HashMap<String, Arc<dyn ParserPlugin>>,
}

impl ParserRegistry {
    pub fn register(&mut self, parser: Arc<dyn ParserPlugin>);
    pub fn get_parser_for(&self, path: &Path) -> Option<Arc<dyn ParserPlugin>>;
}
```

### Phase 3: Service Layer Architecture

**Location**: `src-tauri/src/engine/services/`

```rust
// traits.rs - Service interfaces
pub trait ServiceLifecycle: Send + Sync {
    async fn initialize(&mut self) -> EngineResult<()>;
    async fn shutdown(&mut self) -> EngineResult<()>;
    fn is_initialized(&self) -> bool;
}

pub trait ParserService: ServiceLifecycle {
    async fn parse_codebase(
        &self,
        root: &Path,
        options: ParseOptions,
        progress: Arc<dyn ProgressReporter>,
    ) -> EngineResult<ParseResult>;
}

pub trait DatabaseService: ServiceLifecycle {
    async fn connect(&mut self, config: DatabaseConfig) -> EngineResult<()>;
    async fn import_graph(&self, graph: &CodeGraph) -> EngineResult<()>;
    async fn query<T>(&self, cypher: &str, params: HashMap<String, Value>) -> EngineResult<Vec<T>>;
}

pub trait GraphService: ServiceLifecycle {
    async fn build_graph(&self, parsed_files: Vec<ParsedFile>) -> EngineResult<CodeGraph>;
    async fn optimize_graph(&self, graph: CodeGraph) -> EngineResult<CodeGraph>;
}

pub trait AnalyticsService: ServiceLifecycle {
    async fn calculate_pagerank(&self, graph: &CodeGraph) -> EngineResult<HashMap<String, f64>>;
    async fn detect_communities(&self, graph: &CodeGraph) -> EngineResult<Vec<Community>>;
    async fn analyze_complexity(&self, graph: &CodeGraph) -> EngineResult<ComplexityMetrics>;
}
```

### Phase 4: Dependency Injection Container

**Location**: `src-tauri/src/engine/container/`

```rust
// container.rs - Service container
pub struct ServiceContainer {
    services: HashMap<TypeId, Arc<Mutex<dyn Any + Send + Sync>>>,
    initializers: Vec<Box<dyn Fn(&ServiceContainer) -> Pin<Box<dyn Future<Output = ()> + Send>>>>,
}

impl ServiceContainer {
    pub fn register<T: 'static + Send + Sync>(&mut self, service: T);
    pub fn resolve<T: 'static>(&self) -> Option<Arc<Mutex<T>>>;
    pub async fn initialize_all(&self) -> EngineResult<()>;
}

// modules.rs - Service modules
pub struct CoreModule;
impl Module for CoreModule {
    fn configure(&self, container: &mut ServiceContainer) {
        container.register(ParserServiceImpl::new());
        container.register(GraphServiceImpl::new());
    }
}
```

### Phase 5: Graph Processing Pipeline

**Location**: `src-tauri/src/engine/graph/pipeline/`

```rust
// pipeline.rs
pub struct GraphPipeline {
    steps: Vec<Box<dyn PipelineStep>>,
}

pub trait PipelineStep: Send + Sync {
    async fn execute(&self, context: &mut PipelineContext) -> EngineResult<()>;
    fn name(&self) -> &str;
}

// steps/
pub struct InputValidationStep;
pub struct NodeTransformationStep;
pub struct RelationshipExtractionStep;
pub struct GraphOptimizationStep;
pub struct ValidationStep;

impl GraphPipeline {
    pub fn builder() -> PipelineBuilder {
        PipelineBuilder::new()
            .add_step(InputValidationStep)
            .add_step(NodeTransformationStep)
            .add_step(RelationshipExtractionStep)
            .add_step(GraphOptimizationStep)
            .add_step(ValidationStep)
    }
}
```

### Phase 6: Database Integration

**Location**: `src-tauri/src/engine/database/`

```rust
// neo4j.rs
pub struct Neo4jConnection {
    driver: Arc<Driver>,
    config: DatabaseConfig,
}

impl Neo4jConnection {
    pub async fn connect(config: DatabaseConfig) -> EngineResult<Self>;
    pub async fn import_batch(&self, nodes: Vec<GraphNode>, relationships: Vec<GraphRelationship>);
}

// cypher_builder.rs
pub struct CypherQueryBuilder {
    query: String,
    params: HashMap<String, Value>,
}

impl CypherQueryBuilder {
    pub fn create_node(labels: Vec<String>, props: HashMap<String, Value>) -> Self;
    pub fn create_relationship(rel_type: String, source: String, target: String) -> Self;
    pub fn match_pattern(pattern: &str) -> Self;
}
```

### Phase 7: Analytics Engine

**Location**: `src-tauri/src/engine/analytics/`

```rust
// algorithms.rs
pub struct GraphAnalytics {
    graph: Arc<CodeGraph>,
}

impl GraphAnalytics {
    pub async fn pagerank(&self, options: PageRankOptions) -> HashMap<String, f64>;
    pub async fn betweenness_centrality(&self) -> HashMap<String, f64>;
    pub async fn community_detection(&self, algorithm: CommunityAlgorithm) -> Vec<Community>;
    pub async fn cyclomatic_complexity(&self) -> HashMap<String, u32>;
    pub async fn coupling_analysis(&self) -> CouplingMetrics;
}

// metrics.rs
pub struct CodeMetrics {
    pub lines_of_code: usize,
    pub cyclomatic_complexity: u32,
    pub cognitive_complexity: u32,
    pub maintainability_index: f64,
}
```

### Phase 8: Progress Reporting & Monitoring

**Location**: `src-tauri/src/engine/monitoring/`

```rust
// progress.rs
pub trait ProgressReporter: Send + Sync {
    fn report(&self, message: String, percentage: f32);
    fn complete(&self, message: Option<String>);
    fn error(&self, message: String, error: Option<Box<dyn Error>>);
}

pub struct TauriProgressReporter {
    window: Window,
}

impl ProgressReporter for TauriProgressReporter {
    fn report(&self, message: String, percentage: f32) {
        self.window.emit("parse-progress", ProgressEvent {
            message,
            percentage,
            status: ProgressStatus::InProgress,
        });
    }
}

// metrics.rs
pub struct PerformanceMetrics {
    parse_time: Duration,
    graph_build_time: Duration,
    analysis_time: Duration,
    memory_usage: usize,
}
```

## Implementation Timeline

### Week 1-2: Foundation

- [x] Basic type system (partially done)
- [ ] Error handling framework
- [ ] Service trait definitions
- [ ] Progress reporting system

### Week 3-4: Parser System

- [x] Tree-sitter integration (basic version done)
- [ ] Parser plugin trait
- [ ] Parser registry
- [ ] Batch parsing optimization

### Week 5-6: Service Layer

- [ ] Parser service implementation
- [ ] Graph service implementation
- [ ] Service lifecycle management
- [ ] Dependency injection container

### Week 7-8: Graph Processing

- [x] Basic graph building (done)
- [ ] Graph optimization
- [ ] Pipeline architecture
- [ ] Validation steps

### Week 9-10: Database Integration

- [ ] Neo4j driver setup
- [ ] Cypher query builder
- [ ] Batch import optimization
- [ ] Transaction management

### Week 11-12: Analytics

- [x] Basic PageRank (done)
- [ ] Community detection
- [ ] Complexity metrics
- [ ] Coupling analysis

### Week 13-14: Testing & Optimization

- [ ] Unit tests for all modules
- [ ] Integration tests
- [ ] Performance benchmarks
- [ ] Memory optimization

## Migration Strategy

### Incremental Migration

1. Keep TypeScript engine running alongside Rust
2. Migrate one service at a time
3. Use feature flags to switch between implementations
4. Maintain API compatibility

### Testing Strategy

- Unit tests for each module
- Integration tests for service interactions
- Performance benchmarks comparing TS vs Rust
- Regression tests for parser accuracy

### Rollback Plan

- Feature flags for gradual rollout
- Version tagging for quick rollback
- Maintain TypeScript engine until Rust is stable

## Performance Goals

### Targets

- **Parsing Speed**: 10x faster than TypeScript implementation
- **Memory Usage**: 50% reduction
- **Graph Building**: 5x faster for large codebases
- **Analytics**: Real-time for graphs < 10k nodes

### Optimization Techniques

- Parallel parsing with Rayon
- Memory-mapped file reading
- Incremental parsing for changes
- Graph structure caching
- SIMD optimizations where applicable

## Monitoring & Observability

### Metrics to Track

- Parse time per file
- Memory usage over time
- Graph build performance
- Query execution time
- Error rates and types

### Logging Strategy

- Structured logging with `tracing`
- Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- Performance spans for timing
- Error context preservation

## Documentation Requirements

### Code Documentation

- Rust doc comments for all public APIs
- Examples in documentation
- Architecture decision records (ADRs)
- Performance notes

### User Documentation

- Migration guide from TS to Rust
- API reference
- Performance tuning guide
- Troubleshooting guide

## Success Criteria

### Functional

- ✅ All TypeScript features replicated
- ✅ Backward compatibility maintained
- ✅ All tests passing
- ✅ No regression in accuracy

### Performance

- ✅ 10x parsing speed improvement
- ✅ 50% memory reduction
- ✅ Sub-second response for typical operations
- ✅ Scalable to 100k+ file codebases

### Quality

- ✅ 80% code coverage
- ✅ Zero unsafe code (or justified)
- ✅ All clippy warnings resolved
- ✅ Documentation complete

## Risk Mitigation

### Technical Risks

- **Tree-sitter compatibility**: Maintain grammar versions
- **Neo4j driver stability**: Use stable, well-tested driver
- **Memory management**: Profile and optimize early
- **Concurrency issues**: Use safe abstractions

### Process Risks

- **Scope creep**: Strict feature parity first
- **Timeline delays**: Buffer time for unknowns
- **Integration issues**: Early integration testing

## Next Steps

1. Set up Rust module structure
2. Implement core type system
3. Create parser trait and base implementation
4. Build service layer foundation
5. Integrate with existing Tauri commands
6. Begin incremental migration
