use crate::analysis::AnalysisReport;
use crate::metrics::{
    centrality::CentralityMetrics,
    community::CommunityDetection,
    quality::QualityMetrics,
    risk::RiskAnalysis,
    Metric, MetricResults, MetricValue,
};
use anyhow::Result;
use dashmap::DashMap;
use og_graph::graph::CodeGraph;
use rayon::prelude::*;
use std::sync::Arc;
use tracing::{debug, info, warn, error};

/// Configuration for analytics engine
#[derive(Debug, Clone)]
pub struct AnalyticsConfig {
    /// Weights for composite metrics
    pub weights: MetricWeights,
    /// Enable parallel computation
    pub parallel: bool,
    /// Cache intermediate results
    pub use_cache: bool,
    /// PageRank iterations
    pub pagerank_iterations: usize,
    /// PageRank damping factor
    pub pagerank_damping: f64,
    /// Community detection resolution
    pub louvain_resolution: f64,
}

impl Default for AnalyticsConfig {
    fn default() -> Self {
        Self {
            weights: MetricWeights::default(),
            parallel: true,
            use_cache: true,
            pagerank_iterations: 30,
            pagerank_damping: 0.85,
            louvain_resolution: 1.0,
        }
    }
}

/// Weights for composite metrics
#[derive(Debug, Clone)]
pub struct MetricWeights {
    /// Importance weights
    pub importance_pagerank: f64,
    pub importance_degree: f64,
    pub importance_betweenness: f64,
    /// Risk weights
    pub risk_complexity: f64,
    pub risk_coupling: f64,
    pub risk_churn: f64,
    /// Chokepoint weights
    pub chokepoint_betweenness: f64,
    pub chokepoint_clustering: f64,
    pub chokepoint_degree: f64,
    /// Payoff weights (improvement potential)
    pub payoff_risk: f64,
    pub payoff_importance: f64,
    pub payoff_coverage: f64,
}

impl Default for MetricWeights {
    fn default() -> Self {
        Self {
            // Importance: weighted combination
            importance_pagerank: 0.5,
            importance_degree: 0.3,
            importance_betweenness: 0.2,
            // Risk: complexity and coupling matter most
            risk_complexity: 0.4,
            risk_coupling: 0.4,
            risk_churn: 0.2,
            // Chokepoint: betweenness is key
            chokepoint_betweenness: 0.6,
            chokepoint_clustering: 0.2,
            chokepoint_degree: 0.2,
            // Payoff: balance of factors
            payoff_risk: 0.4,
            payoff_importance: 0.3,
            payoff_coverage: 0.3,
        }
    }
}

/// Main analytics engine
pub struct AnalyticsEngine {
    config: AnalyticsConfig,
    metrics_cache: Arc<DashMap<String, MetricValue>>,
    metrics: Vec<Box<dyn Metric>>,
}

impl AnalyticsEngine {
    /// Create a new analytics engine
    pub fn new(config: AnalyticsConfig) -> Self {
        let mut engine = Self {
            config,
            metrics_cache: Arc::new(DashMap::new()),
            metrics: Vec::new(),
        };

        // Register default metrics
        engine.register_default_metrics();
        engine
    }

    /// Register all default metrics
    fn register_default_metrics(&mut self) {
        // Centrality metrics
        self.add_metric(Box::new(CentralityMetrics::new()));
        // Quality metrics  
        self.add_metric(Box::new(QualityMetrics::new()));
        // Risk analysis
        self.add_metric(Box::new(RiskAnalysis::new()));
        // Community detection
        self.add_metric(Box::new(CommunityDetection::new(
            self.config.louvain_resolution,
        )));
    }

    /// Add a metric to the engine
    pub fn add_metric(&mut self, metric: Box<dyn Metric>) {
        self.metrics.push(metric);
    }

    /// Validate graph before analysis
    fn validate_graph(&self, graph: &CodeGraph) -> Result<()> {
        let node_count = graph.node_map.len();
        let edge_count = graph.graph.edge_count();
        
        debug!("Validating graph: {} nodes, {} edges", node_count, edge_count);
        
        if node_count == 0 {
            warn!("Graph is empty, returning empty analysis");
            return Ok(());
        }
        
        // Check for disconnected components (informational)
        let connected = petgraph::algo::connected_components(&graph.graph);
        if connected > 1 {
            info!("Graph has {} disconnected components", connected);
        }
        
        // Check for self-loops (informational)
        let self_loops = graph.graph.edge_indices()
            .filter(|&e| {
                if let Some((s, t)) = graph.graph.edge_endpoints(e) {
                    s == t
                } else {
                    false
                }
            })
            .count();
        
        if self_loops > 0 {
            debug!("Graph contains {} self-loops", self_loops);
        }
        
        Ok(())
    }

    /// Analyze a code graph
    pub async fn analyze(&self, graph: &CodeGraph) -> Result<AnalysisReport> {
        println!("[ENGINE-ANALYTICS] Starting analysis with {} nodes", graph.node_map.len());
        info!("Starting graph analysis with {} nodes", graph.node_map.len());

        // Validate graph (82%)
        println!("[ENGINE-ANALYTICS] Validating graph...");
        self.validate_graph(graph)?;
        println!("[ENGINE-ANALYTICS] Graph validation passed");

        // Clear cache if not using it
        if !self.config.use_cache {
            self.metrics_cache.clear();
        }

        // Run all metrics with error recovery (83-88%)
        println!("[ENGINE-ANALYTICS] Running metrics (parallel={})", self.config.parallel);
        let results = if self.config.parallel {
            println!("[ENGINE-ANALYTICS] Running parallel metrics...");
            self.run_metrics_parallel_safe(graph).await?
        } else {
            println!("[ENGINE-ANALYTICS] Running sequential metrics...");
            self.run_metrics_sequential_safe(graph).await?
        };
        println!("[ENGINE-ANALYTICS] Metrics completed, got {} results", results.len());

        // Build analysis report (89%)
        println!("[ENGINE-ANALYTICS] Building analysis report...");
        let report = AnalysisReport::new(
            results,
            &self.config.weights,
            graph,
        );

        println!("[ENGINE-ANALYTICS] Analysis complete!");
        info!("Analysis complete");
        Ok(report)
    }

    /// Run metrics in parallel with error recovery
    async fn run_metrics_parallel_safe(&self, graph: &CodeGraph) -> Result<Vec<MetricResults>> {
        debug!("Running metrics in parallel with error recovery");
        
        // Use rayon for parallel execution with panic catching
        let results: Vec<(String, Result<MetricResults>)> = self.metrics
            .par_iter()
            .map(|metric| {
                let name = metric.name().to_string();
                debug!("Running metric: {}", name);
                
                // Catch panics and convert to errors
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    metric.calculate(graph)
                }));
                
                let metric_result = match result {
                    Ok(calc_result) => calc_result,
                    Err(panic_info) => {
                        let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "Unknown panic".to_string()
                        };
                        Err(anyhow::anyhow!("Metric {} panicked: {}", name, msg))
                    }
                };
                
                (name, metric_result)
            })
            .collect();

        // Collect successful results and log failures
        let mut all_results = Vec::new();
        for (name, result) in results {
            match result {
                Ok(metric_result) => {
                    debug!("Metric {} completed successfully", name);
                    all_results.push(metric_result);
                },
                Err(e) => {
                    error!("Metric {} failed: {}", name, e);
                    // Create empty result for failed metric
                    all_results.push(MetricResults::new(name));
                }
            }
        }

        Ok(all_results)
    }

    /// Run metrics sequentially with error recovery
    async fn run_metrics_sequential_safe(&self, graph: &CodeGraph) -> Result<Vec<MetricResults>> {
        debug!("Running metrics sequentially with error recovery");
        println!("[ENGINE-ANALYTICS] Starting sequential metrics execution");
        
        let mut results = Vec::new();
        for (idx, metric) in self.metrics.iter().enumerate() {
            let name = metric.name();
            println!("[ENGINE-ANALYTICS] Running metric {} of {}: {}", idx + 1, self.metrics.len(), name);
            debug!("Running metric: {}", name);
            
            // Catch panics and convert to errors
            println!("[ENGINE-ANALYTICS] About to calculate metric: {}", name);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                println!("[ENGINE-ANALYTICS] Inside panic catch for metric: {}", name);
                metric.calculate(graph)
            }));
            
            println!("[ENGINE-ANALYTICS] Metric {} calculation returned", name);
            match result {
                Ok(Ok(metric_result)) => {
                    println!("[ENGINE-ANALYTICS] Metric {} completed successfully", name);
                    debug!("Metric {} completed successfully", name);
                    results.push(metric_result);
                },
                Ok(Err(e)) => {
                    println!("[ENGINE-ANALYTICS] Metric {} failed with error: {}", name, e);
                    error!("Metric {} failed: {}", name, e);
                    results.push(MetricResults::new(name.to_string()));
                },
                Err(panic_info) => {
                    let msg = if let Some(s) = panic_info.downcast_ref::<String>() {
                        s.clone()
                    } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                        s.to_string()
                    } else {
                        "Unknown panic".to_string()
                    };
                    println!("[ENGINE-ANALYTICS] Metric {} panicked: {}", name, msg);
                    error!("Metric {} panicked: {}", name, msg);
                    results.push(MetricResults::new(name.to_string()));
                }
            }
        }

        println!("[ENGINE-ANALYTICS] All metrics completed");
        Ok(results)
    }

    /// Run metrics in parallel (legacy, kept for compatibility)
    async fn run_metrics_parallel(&self, graph: &CodeGraph) -> Result<Vec<MetricResults>> {
        self.run_metrics_parallel_safe(graph).await
    }

    /// Run metrics sequentially (legacy, kept for compatibility)
    async fn run_metrics_sequential(&self, graph: &CodeGraph) -> Result<Vec<MetricResults>> {
        self.run_metrics_sequential_safe(graph).await
    }

    /// Get cached value
    pub fn get_cached(&self, key: &str) -> Option<MetricValue> {
        self.metrics_cache.get(key).map(|v| v.clone())
    }

    /// Set cached value
    pub fn set_cached(&self, key: String, value: MetricValue) {
        if self.config.use_cache {
            self.metrics_cache.insert(key, value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnalyticsConfig::default();
        assert!(config.parallel);
        assert!(config.use_cache);
        assert_eq!(config.pagerank_iterations, 30);
    }

    #[test]
    fn test_weights_sum_to_one() {
        let weights = MetricWeights::default();
        
        let importance_sum = weights.importance_pagerank 
            + weights.importance_degree 
            + weights.importance_betweenness;
        assert!((importance_sum - 1.0).abs() < 0.001);

        let risk_sum = weights.risk_complexity 
            + weights.risk_coupling 
            + weights.risk_churn;
        assert!((risk_sum - 1.0).abs() < 0.001);
    }
}