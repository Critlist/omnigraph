pub mod algorithms;
pub mod analysis;
pub mod engine;
pub mod metrics;

pub use analysis::{AnalysisReport, ImpactAnalysis};
pub use engine::{AnalyticsConfig, AnalyticsEngine};
pub use metrics::{Metric, MetricResults, MetricValue};

use anyhow::Result;
use og_graph::graph::CodeGraph;
use og_types::metrics::UINodeMetricsV1;

/// Main entry point for analytics with comprehensive error handling
pub async fn analyze_graph(
    graph: &CodeGraph,
    config: Option<AnalyticsConfig>,
) -> Result<AnalysisReport> {
    // Log entry
    println!("[ANALYTICS] Starting graph analysis");
    tracing::info!("[ANALYTICS] Starting graph analysis");
    println!("[ANALYTICS] Graph has {} nodes and {} edges", 
             graph.graph.node_count(), 
             graph.graph.edge_count());
    
    let config = config.unwrap_or_default();
    println!("[ANALYTICS] Creating engine with config: parallel={}, use_cache={}", 
             config.parallel, config.use_cache);
    
    let engine = AnalyticsEngine::new(config);
    
    println!("[ANALYTICS] Calling engine.analyze...");
    // Wrap the analysis in error handling
    match engine.analyze(graph).await {
        Ok(report) => {
            println!("[ANALYTICS] Graph analysis completed successfully");
            tracing::info!("[ANALYTICS] Graph analysis completed successfully");
            Ok(report)
        },
        Err(e) => {
            println!("[ANALYTICS] Graph analysis failed: {}", e);
            tracing::error!("[ANALYTICS] Graph analysis failed: {}", e);
            Err(e)
        }
    }
}

/// Convert analysis report to UI metrics format
pub fn to_ui_metrics(report: &AnalysisReport, graph: &CodeGraph) -> Vec<UINodeMetricsV1> {
    report.to_ui_metrics(graph)
}
