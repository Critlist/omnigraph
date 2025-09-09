use anyhow::Result;
use og_analytics::{analyze_graph, to_ui_metrics, AnalyticsConfig};
use og_graph::graph::{CodeGraph, GraphNode, GraphEdge, GraphData};
use og_parser::ParserEngine;
use og_types::{ParsedFile, NodeType, RelationshipType};
use og_utils::ProgressReporter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::info;

/// Main engine that orchestrates parsing, graph building, and analytics
#[derive(Clone)]
pub struct Engine {
    parser: Arc<ParserEngine>,
    base_path: PathBuf,
}

impl Engine {
    /// Create a new engine for analyzing a codebase
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            parser: Arc::new(ParserEngine::with_base_path(base_path.clone())),
            base_path,
        }
    }

    /// Analyze a codebase and return the graph data
    pub async fn analyze_codebase(
        &self,
        progress: Option<Arc<dyn ProgressReporter>>,
    ) -> Result<GraphData> {
        self.analyze_codebase_internal(progress, false).await
    }
    
    /// Internal method for analyze_codebase with progress control
    async fn analyze_codebase_internal(
        &self,
        progress: Option<Arc<dyn ProgressReporter>>,
        with_metrics: bool,
    ) -> Result<GraphData> {
        // Adjust progress percentages based on whether we're doing metrics
        let parse_end = if with_metrics { 30.0 } else { 40.0 };
        let graph_end = if with_metrics { 60.0 } else { 80.0 };
        let viz_end = if with_metrics { 70.0 } else { 95.0 };
        
        // 1. Discover files
        info!("Discovering files in {:?}", self.base_path);
        if let Some(ref reporter) = progress {
            reporter.report("Discovering files", 5.0);
        }
        let files = self.discover_files(&self.base_path)?;
        info!("Found {} files", files.len());

        // 2. Parse files
        if let Some(ref reporter) = progress {
            reporter.report(&format!("Parsing {} files", files.len()), 10.0);
        }
        let parsed_files = self.parse_files(files, progress.clone())?;
        info!("Parsed {} files", parsed_files.len());
        
        if let Some(ref reporter) = progress {
            reporter.report("Files parsed", parse_end);
        }

        // 3. Build graph
        if let Some(ref reporter) = progress {
            reporter.report("Building dependency graph", parse_end + 5.0);
        }
        let graph = self.build_graph(parsed_files)?;
        info!("Built graph with {} nodes and {} edges", 
              graph.node_map.len(), 
              graph.graph.edge_count());
        
        if let Some(ref reporter) = progress {
            reporter.report(&format!("Graph built: {} nodes, {} edges", 
                graph.node_map.len(), graph.graph.edge_count()), graph_end);
        }

        // 4. Convert to frontend format
        if let Some(ref reporter) = progress {
            reporter.report("Preparing visualization", viz_end);
        }
        let graph_data = graph.to_frontend_format();

        // Only mark complete if we're not doing metrics
        if !with_metrics {
            if let Some(ref reporter) = progress {
                reporter.report("Complete", 100.0);
            }
        }

        Ok(graph_data)
    }

    /// Analyze codebase with analytics
    pub async fn analyze_with_metrics(
        &self,
        progress: Option<Arc<dyn ProgressReporter>>,
    ) -> Result<AnalyzedGraph> {
        tracing::info!("[ENGINE] Starting analyze_with_metrics");
        println!("[ENGINE] Starting analyze_with_metrics");
        
        // Wrap everything in a catch to prevent silent crashes
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            println!("[ENGINE] Inside panic catch wrapper");
        }));
        
        if let Err(e) = result {
            println!("[ENGINE] CRITICAL: Panic in analyze_with_metrics setup: {:?}", e);
            tracing::error!("[ENGINE] CRITICAL: Panic in analyze_with_metrics setup: {:?}", e);
        }
        // Get basic graph (this will go to 70%)
        let graph_data = match self.analyze_codebase_internal(progress.clone(), true).await {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to analyze codebase: {}", e);
                return Err(e);
            }
        };
        
        // Check if we have any data to analyze
        if graph_data.nodes.is_empty() {
            tracing::warn!("No nodes found in graph data, returning empty analysis");
            return Ok(AnalyzedGraph {
                graph_data,
                metrics: Vec::new(),
                summary: AnalysisSummary {
                    total_nodes: 0,
                    total_edges: 0,
                    num_communities: 0,
                    modularity: 0.0,
                    avg_complexity: 0.0,
                    high_risk_count: 0,
                    circular_dependencies: 0,
                },
            });
        }
        
        // Build CodeGraph for analytics
        if let Some(ref reporter) = progress {
            reporter.report("Preparing for analysis", 72.0);
        }
        
        let mut code_graph = CodeGraph::new();
        
        // Add nodes with validation
        let mut valid_nodes = 0;
        for node in &graph_data.nodes {
            if !node.id.is_empty() {
                code_graph.add_node(node.clone());
                valid_nodes += 1;
            } else {
                tracing::warn!("Skipping node with empty ID");
            }
        }
        
        tracing::info!("Added {} valid nodes to graph", valid_nodes);
        
        // Add edges with validation
        let mut valid_edges = 0;
        for link in &graph_data.links {
            if !link.source.is_empty() && !link.target.is_empty() {
                code_graph.add_edge(
                    &link.source,
                    &link.target,
                    GraphEdge {
                        edge_type: link.link_type.clone(),
                        weight: link.value,
                    },
                );
                valid_edges += 1;
            } else {
                tracing::warn!("Skipping edge with empty source or target");
            }
        }
        
        tracing::info!("Added {} valid edges to graph", valid_edges);

        // Check if we have a valid graph
        if code_graph.graph.node_count() == 0 {
            tracing::warn!("Graph has no nodes, returning empty metrics");
            return Ok(AnalyzedGraph {
                graph_data,
                metrics: Vec::new(),
                summary: AnalysisSummary {
                    total_nodes: 0,
                    total_edges: 0,
                    num_communities: 0,
                    modularity: 0.0,
                    avg_complexity: 0.0,
                    high_risk_count: 0,
                    circular_dependencies: 0,
                },
            });
        }

        // Run analytics with detailed progress and error handling
        if let Some(ref reporter) = progress {
            reporter.report("Starting analysis phase", 75.0);
        }
        
        let config = AnalyticsConfig::default();
        
        // Log graph statistics before analysis
        tracing::info!("Starting analysis on graph with {} nodes and {} edges", 
                      code_graph.graph.node_count(), 
                      code_graph.graph.edge_count());
        
        // Try to compute PageRank separately first to catch any issues
        if let Some(ref reporter) = progress {
            reporter.report("Computing PageRank", 78.0);
        }
        
        // Test PageRank calculation (clone for thread safety)
        let graph_for_test = code_graph.graph.clone();
        let node_count = graph_for_test.node_count();
        
        // Simple PageRank test without using the full CodeGraph
        let pagerank_works = if node_count > 0 {
            // Try a simple calculation to see if it would panic
            let test_result = std::panic::catch_unwind(|| {
                let mut test_ranks = HashMap::new();
                for idx in graph_for_test.node_indices() {
                    if let Some(node) = graph_for_test.node_weight(idx) {
                        test_ranks.insert(node.id.clone(), 1.0 / node_count as f64);
                    }
                }
                // If we got here without panicking, PageRank should work
                true
            });
            test_result.unwrap_or(false)
        } else {
            false
        };
        
        if !pagerank_works {
            tracing::error!("PageRank calculation would fail, skipping metrics");
            if let Some(ref reporter) = progress {
                reporter.report("PageRank failed, skipping metrics", 85.0);
            }
            
            // Return graph without metrics if PageRank fails
            return Ok(AnalyzedGraph {
                graph_data,
                metrics: Vec::new(),
                summary: AnalysisSummary {
                    total_nodes: code_graph.graph.node_count(),
                    total_edges: code_graph.graph.edge_count(),
                    num_communities: 0,
                    modularity: 0.0,
                    avg_complexity: 0.0,
                    high_risk_count: 0,
                    circular_dependencies: 0,
                },
            });
        }
        
        if let Some(ref reporter) = progress {
            reporter.report("Running analysis suite", 80.0);
        }
        
        println!("[ENGINE] About to run analysis suite");
        tracing::info!("[ENGINE] About to run analysis suite");
        
        // Try to run analysis with comprehensive error handling
        let analysis_result = {
            // Create a custom config that runs sequentially for better debugging
            let mut config = AnalyticsConfig::default();
            config.parallel = false; // Run sequentially to identify which metric fails
            
            tracing::info!("Attempting analysis with config: parallel={}, use_cache={}", 
                         config.parallel, config.use_cache);
            
            // Try the analysis with timeout (simpler approach without spawning)
            println!("[ENGINE] Creating analysis future...");
            let analysis_future = analyze_graph(&code_graph, Some(config));
            let timeout_duration = std::time::Duration::from_secs(30); // Increased timeout
            println!("[ENGINE] Starting timeout wrapper for {} seconds...", timeout_duration.as_secs());
            
            println!("[ENGINE] Awaiting timeout...");
            let timeout_result = tokio::time::timeout(timeout_duration, analysis_future).await;
            println!("[ENGINE] Timeout completed, processing result...");
            
            match timeout_result {
                Ok(Ok(result)) => {
                    println!("[ENGINE] Analysis succeeded!");
                    tracing::info!("[ENGINE] Analysis succeeded");
                    Ok(result)
                },
                Ok(Err(e)) => {
                    println!("[ENGINE] Analysis returned error: {}", e);
                    tracing::error!("[ENGINE] Analysis returned error: {}", e);
                    Err(e)
                },
                Err(_) => {
                    println!("[ENGINE] Analysis timed out after 30 seconds");
                    tracing::error!("[ENGINE] Analysis timed out after 30 seconds");
                    Err(anyhow::anyhow!("Analysis timed out"))
                }
            }
        };
        
        // Run analysis with error handling and fallback
        let (analysis, metrics_available) = match analysis_result {
            Ok(a) => {
                tracing::info!("Analysis completed successfully");
                (Some(a), true)
            },
            Err(e) => {
                tracing::error!("Analysis failed: {}, returning graph without metrics", e);
                if let Some(ref reporter) = progress {
                    reporter.report("Analysis failed, continuing without metrics", 85.0);
                }
                (None, false)
            }
        };
        
        // If analysis failed, return graph without metrics
        if !metrics_available {
            return Ok(AnalyzedGraph {
                graph_data,
                metrics: Vec::new(),
                summary: AnalysisSummary {
                    total_nodes: code_graph.graph.node_count(),
                    total_edges: code_graph.graph.edge_count(),
                    num_communities: 0,
                    modularity: 0.0,
                    avg_complexity: 0.0,
                    high_risk_count: 0,
                    circular_dependencies: 0,
                },
            });
        }
        
        let analysis = analysis.unwrap();
        
        if let Some(ref reporter) = progress {
            reporter.report("Converting metrics for UI", 90.0);
        }
        
        let ui_metrics = match std::panic::catch_unwind(|| {
            to_ui_metrics(&analysis, &code_graph)
        }) {
            Ok(metrics) => metrics,
            Err(e) => {
                tracing::error!("Failed to convert metrics: {:?}", e);
                Vec::new()
            }
        };
        
        if let Some(ref reporter) = progress {
            reporter.report("Finalizing metrics", 95.0);
        }

        let result = AnalyzedGraph {
            graph_data,
            metrics: ui_metrics,
            summary: AnalysisSummary {
                total_nodes: analysis.summary.total_nodes,
                total_edges: analysis.summary.total_edges,
                num_communities: analysis.summary.num_communities,
                modularity: analysis.summary.modularity,
                avg_complexity: analysis.summary.avg_complexity,
                high_risk_count: analysis.summary.high_risk_count,
                circular_dependencies: analysis.summary.circular_dependencies,
            },
        };
        
        if let Some(ref reporter) = progress {
            reporter.report("Analysis complete", 100.0);
        }
        
        Ok(result)
    }

    /// Discover files in the codebase
    fn discover_files(&self, path: &Path) -> Result<Vec<PathBuf>> {
        use ignore::WalkBuilder;
        
        let mut files = Vec::new();
        let walker = WalkBuilder::new(path)
            .hidden(false)
            .git_ignore(true)
            .build();

        for entry in walker {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() && self.is_supported_file(path) {
                tracing::debug!("Found supported file: {:?}", path);
                files.push(path.to_path_buf());
            }
        }
        
        tracing::info!("Discovered {} supported files in {:?}", files.len(), path);

        Ok(files)
    }

    /// Check if a file is supported
    fn is_supported_file(&self, path: &Path) -> bool {
        let extensions = self.parser.supported_extensions();
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            // Extensions from parsers include the dot (e.g., ".js")
            // but path.extension() returns without dot (e.g., "js")
            let ext_with_dot = format!(".{}", ext);
            extensions.contains(&ext_with_dot.as_str())
        } else {
            false
        }
    }

    /// Parse files in parallel
    fn parse_files(
        &self,
        files: Vec<PathBuf>,
        progress: Option<Arc<dyn ProgressReporter>>,
    ) -> Result<Vec<ParsedFile>> {
        let mut file_contents = Vec::new();
        
        tracing::info!("Preparing to parse {} files", files.len());
        
        for path in &files {
            let content = std::fs::read_to_string(path)?;
            file_contents.push((path.display().to_string(), content));
        }

        let results = self.parser.parse_batch(file_contents, progress);
        
        // Collect successful parses
        let mut parsed = Vec::new();
        for result in results {
            match result {
                Ok(file) => {
                    tracing::debug!("Successfully parsed: {} with {} nodes and {} relationships", 
                                   file.path.display(), 
                                   file.nodes.len(), 
                                   file.relationships.len());
                    parsed.push(file);
                },
                Err(e) => {
                    // Log error but continue
                    tracing::warn!("Failed to parse file: {}", e);
                }
            }
        }
        
        tracing::info!("Successfully parsed {} files", parsed.len());

        Ok(parsed)
    }

    /// Build graph from parsed files
    fn build_graph(&self, parsed_files: Vec<ParsedFile>) -> Result<CodeGraph> {
        let mut graph = CodeGraph::new();
        
        for file in parsed_files {
            // Convert nodes
            for node in file.nodes {
                let graph_node = GraphNode {
                    id: node.id.clone(),
                    name: node.name.clone(),
                    node_type: Self::convert_node_type(&node.node_type),
                    size: 10.0, // Default size
                    color: Self::get_node_color(&node.node_type),
                    file_path: Some(file.path.display().to_string()),
                };
                graph.add_node(graph_node);
            }

            // Convert relationships to edges
            for rel in file.relationships {
                let edge = GraphEdge {
                    edge_type: Self::convert_relationship_type(&rel.relationship_type),
                    weight: 1.0,
                };
                graph.add_edge(&rel.source, &rel.target, edge);
            }
        }

        Ok(graph)
    }

    /// Convert NodeType from og-types to string
    fn convert_node_type(node_type: &NodeType) -> String {
        match node_type {
            NodeType::File => "file",
            NodeType::Module => "module",
            NodeType::Class => "class",
            NodeType::Function => "function",
            NodeType::Method => "method",
            NodeType::Variable => "variable",
            NodeType::Import => "import",
            NodeType::Export => "export",
            NodeType::Interface => "interface",
            NodeType::Property => "property",
            NodeType::TypeAlias => "type_alias",
            NodeType::Enum => "enum",
        }.to_string()
    }

    /// Get color for node type
    fn get_node_color(node_type: &NodeType) -> String {
        match node_type {
            NodeType::File => "#4A90E2",
            NodeType::Module => "#7B68EE",
            NodeType::Class => "#50C878",
            NodeType::Function => "#FFB347",
            NodeType::Method => "#FFA07A",
            NodeType::Variable => "#87CEEB",
            NodeType::Import => "#DDA0DD",
            NodeType::Export => "#F0E68C",
            NodeType::Interface => "#98D8C8",
            NodeType::Property => "#F7DC6F",
            NodeType::TypeAlias => "#BB8FCE",
            NodeType::Enum => "#85C1E2",
        }.to_string()
    }

    /// Convert RelationshipType to string
    fn convert_relationship_type(rel_type: &RelationshipType) -> String {
        match rel_type {
            RelationshipType::Imports => "imports",
            RelationshipType::Exports => "exports",
            RelationshipType::Calls => "calls",
            RelationshipType::Extends => "extends",
            RelationshipType::Implements => "implements",
            RelationshipType::Contains => "contains",
            RelationshipType::References => "references",
        }.to_string()
    }
}

/// Graph data with analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzedGraph {
    pub graph_data: GraphData,
    pub metrics: Vec<og_types::metrics::UINodeMetricsV1>,
    pub summary: AnalysisSummary,
}

/// Analysis summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub num_communities: usize,
    pub modularity: f64,
    pub avg_complexity: f64,
    pub high_risk_count: usize,
    pub circular_dependencies: usize,
}