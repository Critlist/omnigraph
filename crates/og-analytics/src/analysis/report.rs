use super::impact::ImpactAnalysis;
use crate::engine::MetricWeights;
use crate::metrics::{MetricResults, MetricValue};
use og_graph::graph::CodeGraph;
use og_types::metrics::{
    CompositeOutputs, NormalizedMetrics, NormalizationRanges, RawMetrics, UINodeMetricsV1,
};
use std::collections::HashMap;
use tracing::debug;

/// Complete analysis report
#[derive(Debug, Clone)]
pub struct AnalysisReport {
    pub metrics: Vec<MetricResults>,
    pub impact_analysis: ImpactAnalysis,
    pub composite_scores: HashMap<String, CompositeOutputs>,
    pub normalization_ranges: NormalizationRanges,
    pub summary: AnalysisSummary,
}

#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub num_communities: usize,
    pub modularity: f64,
    pub avg_complexity: f64,
    pub high_risk_count: usize,
    pub circular_dependencies: usize,
}

impl AnalysisReport {
    /// Create a new analysis report
    pub fn new(metrics: Vec<MetricResults>, weights: &MetricWeights, graph: &CodeGraph) -> Self {
        debug!("Creating analysis report");

        // Perform impact analysis
        let impact_analysis = ImpactAnalysis::analyze(graph);

        // Calculate normalization ranges
        let normalization_ranges = Self::calculate_normalization_ranges(&metrics, graph);

        // Calculate composite scores
        let composite_scores =
            Self::calculate_composite_scores(&metrics, weights, &normalization_ranges, graph);

        // Generate summary
        let summary = Self::generate_summary(&metrics, &composite_scores, graph);

        Self {
            metrics,
            impact_analysis,
            composite_scores,
            normalization_ranges,
            summary,
        }
    }

    /// Calculate normalization ranges for metrics
    fn calculate_normalization_ranges(
        metrics: &[MetricResults],
        graph: &CodeGraph,
    ) -> NormalizationRanges {
        let mut ranges = NormalizationRanges {
            pagerank_imports: (0.0, 1.0),
            pagerank_calls: (0.0, 1.0),
            k_core: (0.0, 10.0),
            indegree: (0.0, 10.0),
            clustering: (0.0, 1.0),
            betweenness: (0.0, 1.0),
            churn: (0.0, 100.0),
            complexity: (0.0, 50.0),
            owners: (0.0, 10.0),
            coverage: (0.0, 1.0),
        };

        // Find actual ranges from metrics
        for result in metrics {
            match result.name.as_str() {
                "centrality" => {
                    // Extract betweenness range
                    if let Some(MetricValue::Map(betweenness_map)) =
                        result.values.get("betweenness_map")
                    {
                        let values: Vec<f64> = betweenness_map.values().copied().collect();
                        if !values.is_empty() {
                            ranges.betweenness = (
                                values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                                values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                            );
                        }
                    }

                    // Extract clustering range
                    if let Some(MetricValue::Map(clustering_map)) =
                        result.values.get("clustering_map")
                    {
                        let values: Vec<f64> = clustering_map.values().copied().collect();
                        if !values.is_empty() {
                            ranges.clustering = (
                                values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                                values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                            );
                        }
                    }
                }
                "quality" => {
                    // Extract complexity ranges
                    let mut complexity_values = Vec::new();
                    for (key, value) in &result.values {
                        if key.ends_with("_cyclomatic_complexity") {
                            if let Some(v) = value.as_float() {
                                complexity_values.push(v);
                            }
                        }
                    }
                    if !complexity_values.is_empty() {
                        ranges.complexity = (
                            complexity_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                            complexity_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
                        );
                    }
                }
                _ => {}
            }
        }

        // Calculate degree ranges from graph
        let mut indegrees = Vec::new();
        for node_idx in graph.graph.node_indices() {
            let indegree = graph
                .graph
                .edges_directed(node_idx, petgraph::Direction::Incoming)
                .count() as f64;
            indegrees.push(indegree);
        }
        if !indegrees.is_empty() {
            ranges.indegree = (
                indegrees.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
                indegrees.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)),
            );
        }

        ranges
    }

    /// Calculate composite scores for all nodes
    fn calculate_composite_scores(
        metrics: &[MetricResults],
        weights: &MetricWeights,
        ranges: &NormalizationRanges,
        graph: &CodeGraph,
    ) -> HashMap<String, CompositeOutputs> {
        let mut scores = HashMap::new();

        // Collect all metric values by node
        let mut node_metrics: HashMap<String, NodeMetricValues> = HashMap::new();

        for node in graph.nodes() {
            node_metrics.insert(node.id.clone(), NodeMetricValues::default());
        }

        // Extract metrics from results
        for result in metrics {
            for (key, value) in &result.values {
                // Parse node_id and metric name from key
                if let Some(underscore_pos) = key.rfind('_') {
                    let node_id = &key[..underscore_pos];
                    let metric_name = &key[underscore_pos + 1..];

                    if let Some(node_values) = node_metrics.get_mut(node_id) {
                        if let Some(v) = value.as_float() {
                            match metric_name {
                                "betweenness" => node_values.betweenness = Some(v),
                                "degree" => node_values.degree = Some(v),
                                "clustering" => node_values.clustering = Some(v),
                                "cyclomatic_complexity" => node_values.complexity = Some(v),
                                "afferent_coupling" => node_values.coupling_in = Some(v),
                                "efferent_coupling" => node_values.coupling_out = Some(v),
                                "risk" => node_values.risk = Some(v),
                                "chokepoint" => node_values.chokepoint = Some(v),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Add PageRank if available
        let pagerank_scores = graph.calculate_pagerank(30, 0.85);
        for (node_id, pagerank) in pagerank_scores {
            if let Some(node_values) = node_metrics.get_mut(&node_id) {
                node_values.pagerank = Some(pagerank);
            }
        }

        // Calculate composite scores
        for (node_id, values) in node_metrics {
            let importance = Self::calculate_importance(&values, weights, ranges);
            let risk = values.risk.unwrap_or(0.0);
            let chokepoint = values.chokepoint.unwrap_or(0.0);
            let payoff = Self::calculate_payoff(importance, risk, weights);

            scores.insert(
                node_id,
                CompositeOutputs {
                    importance,
                    chokepoint,
                    risk,
                    payoff,
                },
            );
        }

        scores
    }

    /// Calculate importance score
    fn calculate_importance(
        values: &NodeMetricValues,
        weights: &MetricWeights,
        ranges: &NormalizationRanges,
    ) -> f64 {
        let pagerank = Self::normalize(
            values.pagerank.unwrap_or(0.0),
            ranges.pagerank_imports.0,
            ranges.pagerank_imports.1,
        );
        let degree = Self::normalize(
            values.degree.unwrap_or(0.0),
            0.0,
            1.0, // Already normalized
        );
        let betweenness = Self::normalize(
            values.betweenness.unwrap_or(0.0),
            ranges.betweenness.0,
            ranges.betweenness.1,
        );

        weights.importance_pagerank * pagerank
            + weights.importance_degree * degree
            + weights.importance_betweenness * betweenness
    }

    /// Calculate payoff score (improvement potential)
    fn calculate_payoff(importance: f64, risk: f64, weights: &MetricWeights) -> f64 {
        // High payoff = high risk + high importance + low coverage
        let coverage_inverse = 1.0 - 0.5; // Placeholder for coverage
        weights.payoff_risk * risk
            + weights.payoff_importance * importance
            + weights.payoff_coverage * coverage_inverse
    }

    /// Normalize value to 0-1 range
    fn normalize(value: f64, min: f64, max: f64) -> f64 {
        if max <= min {
            return 0.5;
        }
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }

    /// Generate analysis summary
    fn generate_summary(
        metrics: &[MetricResults],
        composite_scores: &HashMap<String, CompositeOutputs>,
        graph: &CodeGraph,
    ) -> AnalysisSummary {
        let mut summary = AnalysisSummary {
            total_nodes: graph.graph.node_count(),
            total_edges: graph.graph.edge_count(),
            num_communities: 0,
            modularity: 0.0,
            avg_complexity: 0.0,
            high_risk_count: 0,
            circular_dependencies: 0,
        };

        // Extract summary data from metrics
        for result in metrics {
            match result.name.as_str() {
                "community" => {
                    if let Some(MetricValue::Integer(num)) = result.values.get("num_communities") {
                        summary.num_communities = *num as usize;
                    }
                    if let Some(MetricValue::Float(mod_score)) = result.values.get("modularity") {
                        summary.modularity = *mod_score;
                    }
                }
                "risk" => {
                    if let Some(MetricValue::Integer(circs)) =
                        result.values.get("circular_dependencies")
                    {
                        summary.circular_dependencies = *circs as usize;
                    }
                }
                "quality" => {
                    let mut complexity_sum = 0.0;
                    let mut complexity_count = 0;
                    for (key, value) in &result.values {
                        if key.ends_with("_cyclomatic_complexity") {
                            if let Some(v) = value.as_float() {
                                complexity_sum += v;
                                complexity_count += 1;
                            }
                        }
                    }
                    if complexity_count > 0 {
                        summary.avg_complexity = complexity_sum / complexity_count as f64;
                    }
                }
                _ => {}
            }
        }

        // Count high-risk nodes
        summary.high_risk_count = composite_scores
            .values()
            .filter(|scores| scores.risk > 0.7)
            .count();

        summary
    }

    /// Convert to UI metrics format
    pub fn to_ui_metrics(&self, graph: &CodeGraph) -> Vec<UINodeMetricsV1> {
        let mut ui_metrics = Vec::new();

        for node in graph.nodes() {
            // Get composite scores
            let composites = self
                .composite_scores
                .get(&node.id)
                .cloned()
                .unwrap_or(CompositeOutputs {
                    importance: 0.0,
                    chokepoint: 0.0,
                    risk: 0.0,
                    payoff: 0.0,
                });

            // Build raw metrics
            let raw = self.build_raw_metrics(&node.id);
            let normalized = self.build_normalized_metrics(&node.id, &raw);

            // Get community
            let community = self.get_node_community(&node.id);

            ui_metrics.push(UINodeMetricsV1 {
                path: node.file_path.clone().unwrap_or_else(|| node.id.clone()),
                name: node.name.clone(),
                node_type: node.node_type.clone(),
                community,
                importance: composites.importance as f32,
                risk: composites.risk as f32,
                chokepoint: composites.chokepoint as f32,
                payoff: composites.payoff as f32,
                raw,
                normalized,
                version: 1,
            });
        }

        ui_metrics
    }

    /// Build raw metrics for a node
    fn build_raw_metrics(&self, node_id: &str) -> RawMetrics {
        let mut raw = RawMetrics {
            pagerank_imports: 0.0,
            pagerank_calls: None,
            indegree: 0,
            outdegree: 0,
            k_core: 0,
            clustering: 0.0,
            betweenness: 0.0,
            churn: 0,
            complexity: 0,
            owners: 0,
            coverage: 0.0,
        };

        // Extract from metric results
        for result in &self.metrics {
            // Get betweenness
            if let Some(value) = result.get_node_value(node_id, "betweenness") {
                raw.betweenness = value;
            }
            // Get clustering
            if let Some(value) = result.get_node_value(node_id, "clustering") {
                raw.clustering = value;
            }
            // Get k_core
            if let Some(MetricValue::Integer(k)) =
                result.values.get(&format!("{}_k_core", node_id))
            {
                raw.k_core = *k;
            }
            // Get complexity
            if let Some(value) = result.get_node_value(node_id, "cyclomatic_complexity") {
                raw.complexity = value as i64;
            }
        }

        raw
    }

    /// Build normalized metrics for a node
    fn build_normalized_metrics(&self, _node_id: &str, raw: &RawMetrics) -> NormalizedMetrics {
        NormalizedMetrics {
            pagerank_imports: Self::normalize(
                raw.pagerank_imports,
                self.normalization_ranges.pagerank_imports.0,
                self.normalization_ranges.pagerank_imports.1,
            ),
            pagerank_calls: raw.pagerank_calls.map(|v| {
                Self::normalize(
                    v,
                    self.normalization_ranges.pagerank_calls.0,
                    self.normalization_ranges.pagerank_calls.1,
                )
            }),
            indegree: Self::normalize(
                raw.indegree as f64,
                self.normalization_ranges.indegree.0,
                self.normalization_ranges.indegree.1,
            ),
            k_core: Self::normalize(
                raw.k_core as f64,
                self.normalization_ranges.k_core.0,
                self.normalization_ranges.k_core.1,
            ),
            clustering: Self::normalize(
                raw.clustering,
                self.normalization_ranges.clustering.0,
                self.normalization_ranges.clustering.1,
            ),
            betweenness: Self::normalize(
                raw.betweenness,
                self.normalization_ranges.betweenness.0,
                self.normalization_ranges.betweenness.1,
            ),
            churn: Self::normalize(
                raw.churn as f64,
                self.normalization_ranges.churn.0,
                self.normalization_ranges.churn.1,
            ),
            complexity: Self::normalize(
                raw.complexity as f64,
                self.normalization_ranges.complexity.0,
                self.normalization_ranges.complexity.1,
            ),
            owners: Self::normalize(
                raw.owners as f64,
                self.normalization_ranges.owners.0,
                self.normalization_ranges.owners.1,
            ),
            coverage: Self::normalize(
                raw.coverage,
                self.normalization_ranges.coverage.0,
                self.normalization_ranges.coverage.1,
            ),
        }
    }

    /// Get community for a node
    fn get_node_community(&self, node_id: &str) -> i64 {
        for result in &self.metrics {
            if result.name == "community" {
                if let Some(MetricValue::Integer(comm)) =
                    result.values.get(&format!("{}_community", node_id))
                {
                    return *comm;
                }
            }
        }
        0
    }
}

#[derive(Default)]
struct NodeMetricValues {
    pagerank: Option<f64>,
    degree: Option<f64>,
    betweenness: Option<f64>,
    clustering: Option<f64>,
    complexity: Option<f64>,
    coupling_in: Option<f64>,
    coupling_out: Option<f64>,
    risk: Option<f64>,
    chokepoint: Option<f64>,
}