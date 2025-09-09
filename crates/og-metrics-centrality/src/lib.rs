use anyhow::Result;
// Removed unused nalgebra imports - can add back if needed for eigenvector
use og_graph::graph::CodeGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Centrality metrics with robust error handling
pub struct CentralityMetrics {
    pub max_iterations: usize,
    pub convergence_threshold: f64,
    pub use_sampling: bool,
    pub sample_size: usize,
}

impl Default for CentralityMetrics {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 1e-6,
            use_sampling: true,
            sample_size: 1000,
        }
    }
}

impl CentralityMetrics {
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculate all centrality metrics with error recovery
    pub fn calculate_all(&self, graph: &CodeGraph) -> Result<CentralityResults> {
        let mut results = CentralityResults::default();
        
        // Validate input
        if graph.graph.node_count() == 0 {
            debug!("Empty graph, returning default centrality results");
            return Ok(results);
        }

        // Calculate each metric with individual error handling
        match self.calculate_degree_centrality(graph) {
            Ok(degree) => results.degree = degree,
            Err(e) => {
                warn!("Degree centrality failed: {}", e);
                results.errors.push(format!("Degree centrality: {}", e));
            }
        }

        match self.calculate_pagerank(graph) {
            Ok(pagerank) => results.pagerank = pagerank,
            Err(e) => {
                warn!("PageRank failed: {}", e);
                results.errors.push(format!("PageRank: {}", e));
            }
        }

        match self.calculate_betweenness_safe(graph) {
            Ok(betweenness) => results.betweenness = betweenness,
            Err(e) => {
                warn!("Betweenness centrality failed: {}", e);
                results.errors.push(format!("Betweenness: {}", e));
            }
        }

        match self.calculate_closeness(graph) {
            Ok(closeness) => results.closeness = closeness,
            Err(e) => {
                warn!("Closeness centrality failed: {}", e);
                results.errors.push(format!("Closeness: {}", e));
            }
        }

        Ok(results)
    }

    /// Calculate degree centrality with validation
    pub fn calculate_degree_centrality(&self, graph: &CodeGraph) -> Result<HashMap<String, DegreeMetrics>> {
        let mut degree_map = HashMap::new();
        let node_count = graph.graph.node_count() as f64;

        if node_count <= 1.0 {
            return Ok(degree_map);
        }

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                let in_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count() as f64;
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count() as f64;

                let normalized_in = (in_degree / (node_count - 1.0)).clamp(0.0, 1.0);
                let normalized_out = (out_degree / (node_count - 1.0)).clamp(0.0, 1.0);
                let total = ((in_degree + out_degree) / (2.0 * (node_count - 1.0))).clamp(0.0, 1.0);

                degree_map.insert(
                    node.id.clone(),
                    DegreeMetrics {
                        in_degree: normalized_in,
                        out_degree: normalized_out,
                        total_degree: total,
                    },
                );
            }
        }

        Ok(degree_map)
    }

    /// Calculate PageRank with proper convergence checks
    pub fn calculate_pagerank(&self, graph: &CodeGraph) -> Result<HashMap<String, f64>> {
        let node_count = graph.graph.node_count();
        if node_count == 0 {
            return Ok(HashMap::new());
        }

        let damping = 0.85;
        let mut ranks = HashMap::new();
        let initial_rank = 1.0 / node_count as f64;

        // Initialize ranks
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                ranks.insert(node.id.clone(), initial_rank);
            }
        }

        // Power iteration with convergence check
        for iteration in 0..self.max_iterations {
            let mut new_ranks = HashMap::new();
            let mut max_diff: f64 = 0.0;

            for node_idx in graph.graph.node_indices() {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    let mut rank_sum = 0.0;

                    // Sum contributions from incoming edges
                    for edge in graph.graph.edges_directed(node_idx, Direction::Incoming) {
                        if let Some(source_node) = graph.graph.node_weight(edge.source()) {
                            let out_degree = graph
                                .graph
                                .edges_directed(edge.source(), Direction::Outgoing)
                                .count() as f64;
                            
                            if out_degree > 0.0 {
                                let source_rank = ranks.get(&source_node.id).unwrap_or(&initial_rank);
                                if source_rank.is_finite() {
                                    rank_sum += source_rank / out_degree;
                                }
                            }
                        }
                    }

                    let new_rank = ((1.0 - damping) / node_count as f64 + damping * rank_sum)
                        .clamp(0.0, 1.0);
                    
                    let old_rank = ranks.get(&node.id).unwrap_or(&initial_rank);
                    max_diff = max_diff.max((new_rank - old_rank).abs());
                    
                    new_ranks.insert(node.id.clone(), new_rank);
                }
            }

            ranks = new_ranks;

            // Check convergence
            if max_diff < self.convergence_threshold {
                debug!("PageRank converged after {} iterations", iteration + 1);
                break;
            }
        }

        Ok(ranks)
    }

    /// Calculate betweenness centrality with sampling for large graphs
    pub fn calculate_betweenness_safe(&self, graph: &CodeGraph) -> Result<HashMap<String, f64>> {
        let node_count = graph.graph.node_count();
        
        if node_count <= 2 {
            let mut result = HashMap::new();
            for node_idx in graph.graph.node_indices() {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    result.insert(node.id.clone(), 0.0);
                }
            }
            return Ok(result);
        }

        // Use sampling for large graphs
        let should_sample = self.use_sampling && node_count > self.sample_size;
        let sample_size = if should_sample {
            self.sample_size.min(node_count)
        } else {
            node_count
        };

        debug!("Calculating betweenness with {} samples", sample_size);

        let mut betweenness = HashMap::new();
        
        // Initialize all nodes with 0
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                betweenness.insert(node.id.clone(), 0.0);
            }
        }

        // Sample nodes for BFS
        let node_indices: Vec<NodeIndex> = graph.graph.node_indices().collect();
        let step = if should_sample {
            node_count / sample_size
        } else {
            1
        };

        for (i, &source) in node_indices.iter().step_by(step.max(1)).enumerate() {
            if i >= sample_size {
                break;
            }

            // Run single-source shortest path
            let paths: HashMap<petgraph::graph::NodeIndex, f64> = petgraph::algo::dijkstra(&graph.graph, source, None, |_| 1.0);
            
            // Count paths through intermediate nodes (simplified)
            for (&target, &dist) in &paths {
                if target != source && dist > 0.0 && dist.is_finite() {
                    // Simplified betweenness increment
                    for (&intermediate, &int_dist) in &paths {
                        if intermediate != source && intermediate != target {
                            if int_dist < dist && int_dist.is_finite() {
                                if let Some(node) = graph.graph.node_weight(intermediate) {
                                    *betweenness.entry(node.id.clone()).or_insert(0.0) += 1.0;
                                }
                            }
                        }
                    }
                }
            }
        }

        // Normalize by sample size if sampling was used
        if should_sample {
            let scale = node_count as f64 / sample_size as f64;
            for value in betweenness.values_mut() {
                *value *= scale;
            }
        }

        // Normalize to [0, 1]
        let max_betweenness = betweenness.values().cloned().fold(0.0, f64::max);
        if max_betweenness > 0.0 {
            for value in betweenness.values_mut() {
                *value /= max_betweenness;
            }
        }

        Ok(betweenness)
    }

    /// Calculate closeness centrality
    pub fn calculate_closeness(&self, graph: &CodeGraph) -> Result<HashMap<String, f64>> {
        let mut closeness = HashMap::new();
        let node_count = graph.graph.node_count();
        
        if node_count <= 1 {
            for node_idx in graph.graph.node_indices() {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    closeness.insert(node.id.clone(), 1.0);
                }
            }
            return Ok(closeness);
        }

        for source in graph.graph.node_indices() {
            let paths: HashMap<petgraph::graph::NodeIndex, f64> = petgraph::algo::dijkstra(&graph.graph, source, None, |_| 1.0);
            
            let mut total_distance = 0.0;
            let mut reachable_nodes = 0;
            
            for (&target, &dist) in &paths {
                if target != source && dist.is_finite() && dist > 0.0 {
                    total_distance += dist;
                    reachable_nodes += 1;
                }
            }
            
            if let Some(node) = graph.graph.node_weight(source) {
                let closeness_value = if reachable_nodes > 0 && total_distance > 0.0 {
                    (reachable_nodes as f64) / total_distance
                } else {
                    0.0
                };
                
                closeness.insert(node.id.clone(), closeness_value.clamp(0.0, 1.0));
            }
        }

        // Normalize
        let max_closeness = closeness.values().cloned().fold(0.0, f64::max);
        if max_closeness > 0.0 {
            for value in closeness.values_mut() {
                *value /= max_closeness;
            }
        }

        Ok(closeness)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CentralityResults {
    pub degree: HashMap<String, DegreeMetrics>,
    pub pagerank: HashMap<String, f64>,
    pub betweenness: HashMap<String, f64>,
    pub closeness: HashMap<String, f64>,
    pub errors: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct DegreeMetrics {
    pub in_degree: f64,
    pub out_degree: f64,
    pub total_degree: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_graph::graph::GraphNode;

    #[test]
    fn test_empty_graph() {
        let graph = CodeGraph::new();
        let metrics = CentralityMetrics::new();
        let results = metrics.calculate_all(&graph).unwrap();
        assert!(results.degree.is_empty());
        assert!(results.pagerank.is_empty());
    }

    #[test]
    fn test_single_node() {
        let mut graph = CodeGraph::new();
        graph.add_node(GraphNode {
            id: "node1".to_string(),
            name: "Node 1".to_string(),
            node_type: "file".to_string(),
            file_path: Some("/test.js".to_string()),
            size: 100,
            color: None,
        });
        
        let metrics = CentralityMetrics::new();
        let results = metrics.calculate_all(&graph).unwrap();
        assert_eq!(results.degree.len(), 1);
        assert_eq!(results.pagerank.len(), 1);
    }
}