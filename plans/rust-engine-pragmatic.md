# Pragmatic Rust Engine Implementation

## Workspace Structure

```toml
# /Cargo.toml
[workspace]
members = [
    "crates/og-types",
    "crates/og-parser", 
    "crates/og-graph",
    "crates/og-db",
    "crates/og-analytics",
    "crates/og-services",
    "crates/og-utils",
    "src-tauri"
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
thiserror = "1"
anyhow = "1"
tracing = "0.1"
neo4rs = "0.7"
rayon = "1.8"
```

## Phase 1: Core Types & DTOs

### `crates/og-types/src/lib.rs`

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// UI-facing stable contract
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UINodeMetricsV1 {
    pub path: String,
    pub name: String,
    pub node_type: String,
    pub community: i64,
    pub importance: f32,
    pub risk: f32,
    pub chokepoint: f32,
    pub payoff: f32,
    pub raw: RawMetrics,
    pub normalized: NormalizedMetrics,
    #[serde(default = "default_version")]
    pub version: u8,
}

fn default_version() -> u8 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawMetrics {
    pub pagerank_imports: f64,
    pub pagerank_calls: Option<f64>,
    pub indegree: i64,
    pub outdegree: i64,
    pub k_core: i64,
    pub clustering: f64,
    pub betweenness: f64,
    pub churn: i64,
    pub complexity: i64,
    pub owners: i64,
    pub coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedMetrics {
    pub pagerank_imports: f64,
    pub pagerank_calls: Option<f64>,
    pub indegree: f64,
    pub k_core: f64,
    pub clustering: f64,
    pub betweenness: f64,
    pub churn: f64,
    pub complexity: f64,
    pub owners: f64,
    pub coverage: f64,
}

// Internal AST types
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: String,
    pub language: Language,
    pub nodes: Vec<AstNode>,
    pub relationships: Vec<Relationship>,
    pub parse_time_ms: u64,
    pub errors: Vec<ParseError>,
}

#[derive(Debug, Clone)]
pub struct AstNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
pub enum NodeType {
    File,
    Module,
    Class,
    Interface,
    Function,
    Method,
    Variable,
    Property,
    Import,
    Export,
}

#[derive(Debug, Clone)]
pub struct Relationship {
    pub id: String,
    pub source: String,
    pub target: String,
    pub rel_type: RelationshipType,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy)]
pub enum RelationshipType {
    Contains,
    Calls,
    Imports,
    Exports,
    Extends,
    Implements,
    References,
}

#[derive(Debug, Clone, Copy)]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
    Rust,
}
```

### `crates/og-types/src/error.rs`

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Parse error in {file}: {message}")]
    ParseError { file: String, message: String },
    
    #[error("Database error: {0}")]
    DatabaseError(#[from] neo4rs::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Graph processing error: {0}")]
    GraphError(String),
    
    #[error("Analytics error: {0}")]
    AnalyticsError(String),
}

pub type EngineResult<T> = Result<T, EngineError>;
```

## Phase 2: Parser Module

### `crates/og-parser/Cargo.toml`

```toml
[package]
name = "og-parser"
version = "0.1.0"

[dependencies]
og-types = { path = "../og-types" }
tree-sitter = "0.20"
tree-sitter-javascript = { version = "0.20", optional = true }
tree-sitter-typescript = { version = "0.20", optional = true }
tree-sitter-python = { version = "0.20", optional = true }
tree-sitter-rust = { version = "0.20", optional = true }
rayon = "1.8"
tracing = "0.1"
dashmap = "5"

[features]
default = ["js", "ts", "python"]
js = ["tree-sitter-javascript"]
ts = ["tree-sitter-typescript"]
python = ["tree-sitter-python"]
rust = ["tree-sitter-rust"]
```

### `crates/og-parser/src/lib.rs`

```rust
use og_types::{ParsedFile, Language, EngineResult};
use std::path::Path;
use rayon::prelude::*;
use tracing::{instrument, info};

pub trait Parser: Send + Sync {
    fn supported_extensions(&self) -> &[&str];
    fn can_parse(&self, path: &Path) -> bool;
    fn parse(&self, path: &Path, content: &str) -> EngineResult<ParsedFile>;
}

pub struct ParserEngine {
    parsers: Vec<Box<dyn Parser>>,
}

impl ParserEngine {
    pub fn new() -> Self {
        let mut parsers: Vec<Box<dyn Parser>> = vec![];
        
        #[cfg(feature = "js")]
        parsers.push(Box::new(JavaScriptParser::new()));
        
        #[cfg(feature = "ts")]
        parsers.push(Box::new(TypeScriptParser::new()));
        
        #[cfg(feature = "python")]
        parsers.push(Box::new(PythonParser::new()));
        
        Self { parsers }
    }
    
    #[instrument(skip(self, files))]
    pub fn parse_batch<'a>(
        &self,
        files: impl ParallelIterator<Item = (&'a Path, &'a str)>,
    ) -> Vec<EngineResult<ParsedFile>> {
        files
            .map(|(path, content)| self.parse_file(path, content))
            .collect()
    }
    
    fn parse_file(&self, path: &Path, content: &str) -> EngineResult<ParsedFile> {
        for parser in &self.parsers {
            if parser.can_parse(path) {
                return parser.parse(path, content);
            }
        }
        Err(og_types::EngineError::ParseError {
            file: path.display().to_string(),
            message: "No parser found".to_string(),
        })
    }
}
```

## Phase 3: Database Module

### `crates/og-db/src/lib.rs`

```rust
use neo4rs::{Graph, query, Node, Relation};
use og_types::{EngineResult, AstNode, Relationship};
use std::sync::Arc;
use tracing::{instrument, info};

pub struct Neo4jClient {
    graph: Arc<Graph>,
}

impl Neo4jClient {
    pub async fn connect(uri: &str, username: &str, password: &str) -> EngineResult<Self> {
        let graph = Arc::new(
            Graph::new(uri, username, password).await?
        );
        Ok(Self { graph })
    }
    
    #[instrument(skip(self, nodes, relationships))]
    pub async fn ingest_batch(
        &self,
        nodes: Vec<AstNode>,
        relationships: Vec<Relationship>,
    ) -> EngineResult<()> {
        // Clear existing data
        self.graph
            .run(query("MATCH (n) DETACH DELETE n"))
            .await?;
        
        // Batch create nodes
        let create_nodes = include_str!("cypher/batch_create_nodes.cypher");
        self.graph
            .run(query(create_nodes).param("nodes", nodes))
            .await?;
        
        // Batch create relationships
        let create_rels = include_str!("cypher/batch_create_relationships.cypher");
        self.graph
            .run(query(create_rels).param("relationships", relationships))
            .await?;
        
        info!("Ingested {} nodes, {} relationships", nodes.len(), relationships.len());
        Ok(())
    }
    
    pub async fn project_graph(&self, name: &str) -> EngineResult<()> {
        let cypher = include_str!("cypher/project_graph.cypher");
        self.graph
            .run(query(cypher).param("graph_name", name))
            .await?;
        Ok(())
    }
}
```

### `crates/og-db/src/cypher/batch_create_nodes.cypher`

```cypher
UNWIND $nodes AS node
CREATE (n:Node {
    id: node.id,
    name: node.name,
    type: node.node_type,
    file_path: node.file_path,
    start_line: node.start_line,
    end_line: node.end_line
})
SET n += node.metadata
```

## Phase 4: Analytics Module

### `crates/og-analytics/src/lib.rs`

```rust
use og_types::{RawMetrics, NormalizedMetrics, UINodeMetricsV1, EngineResult};
use neo4rs::Graph;
use std::collections::HashMap;
use tracing::{instrument, info};

pub struct AnalyticsEngine {
    client: Arc<Neo4jClient>,
}

#[derive(Debug, Clone)]
pub struct CompositeInputs {
    pub pr_imports: f64,
    pub pr_calls: Option<f64>,
    pub k_core: f64,
    pub indegree: f64,
    pub clustering: f64,
    pub betweenness: f64,
    pub churn: f64,
    pub complexity: f64,
    pub owners: f64,
    pub coverage: f64,
}

#[derive(Debug, Clone)]
pub struct CompositeOutputs {
    pub importance: f64,
    pub chokepoint: f64,
    pub risk: f64,
    pub payoff: f64,
}

#[derive(Debug, Clone)]
pub struct NormalizationRanges {
    pub pr_imports: (f64, f64),
    pub pr_calls: (f64, f64),
    pub k_core: (f64, f64),
    pub indegree: (f64, f64),
    pub clustering: (f64, f64),
    pub betweenness: (f64, f64),
    pub churn: (f64, f64),
    pub complexity: (f64, f64),
    pub owners: (f64, f64),
    pub coverage: (f64, f64),
}

impl AnalyticsEngine {
    #[instrument(skip(self))]
    pub async fn compute_all_metrics(&self) -> EngineResult<Vec<UINodeMetricsV1>> {
        // Run GDS algorithms
        let pagerank = self.run_pagerank("og_imports").await?;
        let indegree = self.run_degree_centrality().await?;
        let k_core = self.run_k_core().await?;
        let clustering = self.run_clustering().await?;
        let betweenness = self.run_betweenness().await?;
        let communities = self.run_louvain().await?;
        
        // Get code metrics (churn, complexity, etc)
        let code_metrics = self.get_code_metrics().await?;
        
        // Calculate normalization ranges
        let ranges = self.calculate_ranges(&pagerank, &indegree, &k_core, 
                                          &clustering, &betweenness, &code_metrics)?;
        
        // Build UI metrics for each node
        let mut results = Vec::new();
        for (node_id, node_data) in self.get_all_nodes().await? {
            let raw = RawMetrics {
                pagerank_imports: pagerank.get(&node_id).copied().unwrap_or(0.0),
                pagerank_calls: None, // TODO: calls graph
                indegree: indegree.get(&node_id).copied().unwrap_or(0) as i64,
                k_core: k_core.get(&node_id).copied().unwrap_or(0) as i64,
                clustering: clustering.get(&node_id).copied().unwrap_or(0.0),
                betweenness: betweenness.get(&node_id).copied().unwrap_or(0.0),
                churn: code_metrics.get(&node_id).map(|m| m.churn).unwrap_or(0),
                complexity: code_metrics.get(&node_id).map(|m| m.complexity).unwrap_or(0),
                owners: code_metrics.get(&node_id).map(|m| m.owners).unwrap_or(1),
                coverage: code_metrics.get(&node_id).map(|m| m.coverage).unwrap_or(0.0),
            };
            
            let normalized = self.normalize(&raw, &ranges);
            let composites = self.compute_composites(&normalized);
            
            results.push(UINodeMetricsV1 {
                path: node_data.path,
                name: node_data.name,
                node_type: node_data.node_type,
                community: communities.get(&node_id).copied().unwrap_or(0),
                importance: composites.importance as f32,
                risk: composites.risk as f32,
                chokepoint: composites.chokepoint as f32,
                payoff: composites.payoff as f32,
                raw,
                normalized,
                version: 1,
            });
        }
        
        Ok(results)
    }
    
    fn normalize(&self, raw: &RawMetrics, ranges: &NormalizationRanges) -> NormalizedMetrics {
        NormalizedMetrics {
            pagerank_imports: normalize_value(raw.pagerank_imports, ranges.pr_imports),
            pagerank_calls: raw.pagerank_calls.map(|v| normalize_value(v, ranges.pr_calls)),
            indegree: normalize_value(raw.indegree as f64, ranges.indegree),
            k_core: normalize_value(raw.k_core as f64, ranges.k_core),
            clustering: normalize_value(raw.clustering, ranges.clustering),
            betweenness: normalize_value(raw.betweenness, ranges.betweenness),
            churn: normalize_value(raw.churn as f64, ranges.churn),
            complexity: normalize_value(raw.complexity as f64, ranges.complexity),
            owners: normalize_value(raw.owners as f64, ranges.owners),
            coverage: normalize_value(raw.coverage, ranges.coverage),
        }
    }
    
    fn compute_composites(&self, norm: &NormalizedMetrics) -> CompositeOutputs {
        let importance = 
            0.40 * norm.pagerank_imports +
            0.20 * norm.indegree +
            0.20 * norm.k_core +
            0.10 * norm.clustering +
            0.10 * norm.betweenness;
        
        let chokepoint =
            0.50 * norm.betweenness +
            0.30 * norm.k_core +
            0.20 * (1.0 - norm.clustering);
        
        let risk =
            0.30 * norm.churn +
            0.30 * norm.complexity +
            0.20 * (1.0 / norm.owners.max(0.1)) +
            0.20 * (1.0 - norm.coverage);
        
        let payoff = importance * (1.0 - risk);
        
        CompositeOutputs {
            importance,
            chokepoint,
            risk,
            payoff,
        }
    }
    
    async fn run_pagerank(&self, graph: &str) -> EngineResult<HashMap<String, f64>> {
        let cypher = include_str!("cypher/pagerank.cypher");
        // Execute and collect results
        Ok(HashMap::new()) // TODO: implement
    }
}

fn normalize_value(value: f64, range: (f64, f64)) -> f64 {
    if range.1 <= range.0 {
        return 0.0;
    }
    ((value - range.0) / (range.1 - range.0)).clamp(0.0, 1.0)
}
```

## Phase 5: Service Orchestration

### `crates/og-services/src/lib.rs`

```rust
use og_types::{UINodeMetricsV1, EngineResult};
use og_parser::ParserEngine;
use og_db::Neo4jClient;
use og_analytics::AnalyticsEngine;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{instrument, info};

pub struct Engine {
    parser: Arc<ParserEngine>,
    db: Arc<Neo4jClient>,
    analytics: Arc<AnalyticsEngine>,
    cached_metrics: Arc<RwLock<Option<Vec<UINodeMetricsV1>>>>,
}

impl Engine {
    pub async fn new(db_config: DatabaseConfig) -> EngineResult<Self> {
        let parser = Arc::new(ParserEngine::new());
        let db = Arc::new(
            Neo4jClient::connect(&db_config.uri, &db_config.username, &db_config.password).await?
        );
        let analytics = Arc::new(AnalyticsEngine::new(db.clone()));
        
        Ok(Self {
            parser,
            db,
            analytics,
            cached_metrics: Arc::new(RwLock::new(None)),
        })
    }
    
    #[instrument(skip(self))]
    pub async fn parse_and_analyze(&self, root_path: &Path) -> EngineResult<Vec<UINodeMetricsV1>> {
        info!("Starting parse and analyze for: {}", root_path.display());
        
        // 1. Parse files
        let files = self.discover_files(root_path)?;
        let parsed = self.parser.parse_batch(files.par_iter().map(|(p, c)| (p.as_path(), c.as_str())));
        
        // 2. Build graph
        let (nodes, relationships) = self.build_graph(parsed)?;
        
        // 3. Ingest to Neo4j
        self.db.ingest_batch(nodes, relationships).await?;
        
        // 4. Project graph
        self.db.project_graph("og_imports").await?;
        
        // 5. Run analytics
        let metrics = self.analytics.compute_all_metrics().await?;
        
        // 6. Cache results
        *self.cached_metrics.write().await = Some(metrics.clone());
        
        Ok(metrics)
    }
    
    pub async fn get_cached_metrics(&self) -> Option<Vec<UINodeMetricsV1>> {
        self.cached_metrics.read().await.clone()
    }
    
    pub async fn get_node_metrics(&self, path: &str) -> EngineResult<UINodeMetricsV1> {
        let metrics = self.cached_metrics.read().await;
        metrics
            .as_ref()
            .and_then(|m| m.iter().find(|n| n.path == path).cloned())
            .ok_or_else(|| og_types::EngineError::GraphError(format!("Node not found: {}", path)))
    }
}

pub struct DatabaseConfig {
    pub uri: String,
    pub username: String,
    pub password: String,
}
```

## Phase 6: Tauri Integration

### `src-tauri/src/commands.rs`

```rust
use og_services::{Engine, DatabaseConfig};
use og_types::UINodeMetricsV1;
use std::sync::Arc;
use tauri::State;
use tracing::info;

#[tauri::command]
pub async fn parse_codebase(
    path: String,
    state: State<'_, Arc<Engine>>,
) -> Result<Vec<UINodeMetricsV1>, String> {
    info!("Parsing codebase: {}", path);
    state
        .parse_and_analyze(&Path::new(&path))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_node_metrics(
    path: String,
    state: State<'_, Arc<Engine>>,
) -> Result<UINodeMetricsV1, String> {
    state
        .get_node_metrics(&path)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_top_nodes(
    metric: String,
    limit: usize,
    state: State<'_, Arc<Engine>>,
) -> Result<Vec<UINodeMetricsV1>, String> {
    let mut metrics = state
        .get_cached_metrics()
        .await
        .ok_or("No metrics available")?;
    
    // Sort by requested metric
    match metric.as_str() {
        "importance" => metrics.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap()),
        "risk" => metrics.sort_by(|a, b| b.risk.partial_cmp(&a.risk).unwrap()),
        "chokepoint" => metrics.sort_by(|a, b| b.chokepoint.partial_cmp(&a.chokepoint).unwrap()),
        "payoff" => metrics.sort_by(|a, b| b.payoff.partial_cmp(&a.payoff).unwrap()),
        _ => return Err(format!("Unknown metric: {}", metric)),
    }
    
    Ok(metrics.into_iter().take(limit).collect())
}

#[tauri::command]
pub async fn recompute_metrics(
    state: State<'_, Arc<Engine>>,
) -> Result<(), String> {
    info!("Recomputing metrics");
    // Just rerun analytics, don't reparse
    state
        .analytics
        .compute_all_metrics()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
```

## Testing Strategy

### `crates/og-analytics/tests/composites.rs`

```rust
use og_analytics::*;
use insta::assert_json_snapshot;

#[test]
fn test_composite_calculations() {
    let norm = NormalizedMetrics {
        pagerank_imports: 0.8,
        pagerank_calls: None,
        indegree: 0.7,
        k_core: 0.6,
        clustering: 0.3,
        betweenness: 0.5,
        churn: 0.2,
        complexity: 0.4,
        owners: 0.8,
        coverage: 0.9,
    };
    
    let engine = AnalyticsEngine::new_test();
    let composites = engine.compute_composites(&norm);
    
    assert_json_snapshot!(composites);
}
```

## Key Improvements from Original Plan

1. **No IoC container** - Simple builder pattern with explicit dependencies
2. **Workspace structure** - Better compile times and modularity  
3. **Stable DTOs** - `UINodeMetricsV1` with versioning
4. **Pragmatic concurrency** - Rayon for CPU, Tokio for I/O
5. **Cypher in files** - Embedded with `include_str!`
6. **Single math home** - All normalization/composites in analytics module
7. **Thin Tauri layer** - Commands just validate and delegate
8. **Iterator-based parsing** - Stream results for large repos
9. **Proper error types** - `thiserror` for domains, `anyhow` at edges
10. **Snapshot testing** - Golden fixtures with `insta`

## Next Steps

1. Set up workspace structure
2. Implement types crate with DTOs
3. Parser with single language (JS)
4. Neo4j connection and ingestion
5. Basic analytics (PageRank + composites)
6. Wire up Tauri commands
7. Add remaining parsers
8. Full GDS algorithm suite
9. Performance optimization
10. Integration tests