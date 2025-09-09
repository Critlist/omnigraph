use super::{Metric, MetricResults, MetricValue};
use anyhow::Result;
use nalgebra::{DMatrix, DVector};
use og_graph::graph::CodeGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use petgraph::Direction;
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Centrality metrics calculator
pub struct CentralityMetrics {
    calculate_eigenvector: bool,
    max_eigenvector_iterations: usize,
}

impl CentralityMetrics {
    pub fn new() -> Self {
        Self {
            calculate_eigenvector: true,
            max_eigenvector_iterations: 100,
        }
    }

    /// Calculate degree centrality (in and out)
    fn calculate_degree(&self, graph: &CodeGraph) -> HashMap<String, (f64, f64)> {
        let mut degree_map = HashMap::new();
        let node_count = graph.graph.node_count() as f64;

        if node_count <= 1.0 {
            return degree_map;
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

                // Normalize by node count - 1
                let normalized_in = in_degree / (node_count - 1.0);
                let normalized_out = out_degree / (node_count - 1.0);

                degree_map.insert(node.id.clone(), (normalized_in, normalized_out));
            }
        }

        degree_map
    }

    /// Calculate betweenness centrality
    fn calculate_betweenness(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Calculating betweenness centrality");
        
        let mut betweenness = HashMap::new();
        let node_count = graph.graph.node_count();
        
        // Handle small graphs
        if node_count <= 2 {
            for node_idx in graph.graph.node_indices() {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    betweenness.insert(node.id.clone(), 0.0);
                }
            }
            return betweenness;
        }
        
        // Initialize all nodes with 0 betweenness
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                betweenness.insert(node.id.clone(), 0.0);
            }
        }
        
        // Simple betweenness centrality calculation
        // For each pair of nodes, find shortest paths and count
        for source in graph.graph.node_indices() {
            let paths = petgraph::algo::dijkstra(&graph.graph, source, None, |_| 1.0);
            
            for target in graph.graph.node_indices() {
                if source != target {
                    // Count paths through intermediate nodes
                    for intermediate in graph.graph.node_indices() {
                        if intermediate != source && intermediate != target {
                            // Simplified: increment if on a path
                            if paths.contains_key(&intermediate) && paths.contains_key(&target) {
                                if let Some(node) = graph.graph.node_weight(intermediate) {
                                    *betweenness.entry(node.id.clone()).or_insert(0.0) += 1.0;
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Normalize (with safety check)
        let normalization = ((node_count - 1) * (node_count - 2)) as f64;
        if normalization > 0.0 {
            for value in betweenness.values_mut() {
                *value /= normalization;
                // Ensure finite values
                if !value.is_finite() {
                    *value = 0.0;
                }
            }
        }
        
        betweenness
    }

    /// Calculate closeness centrality
    fn calculate_closeness(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Calculating closeness centrality");
        
        let mut result = HashMap::new();
        let node_count = graph.graph.node_count();
        
        if node_count <= 1 {
            // For single node, closeness is 1.0
            for node_idx in graph.graph.node_indices() {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    result.insert(node.id.clone(), 1.0);
                }
            }
            return result;
        }

        // For each node, calculate shortest paths to all other nodes
        for node_idx in graph.graph.node_indices() {
            let distances = petgraph::algo::dijkstra(
                &graph.graph,
                node_idx,
                None,
                |_| 1.0,
            );
            
            if distances.len() > 1 {
                // Only sum distances to reachable nodes (excluding self)
                let reachable_distances: Vec<f64> = distances
                    .iter()
                    .filter(|(idx, _)| **idx != node_idx)
                    .map(|(_, &dist)| dist)
                    .collect();
                
                if !reachable_distances.is_empty() {
                    let total_distance: f64 = reachable_distances.iter().sum();
                    let closeness = if total_distance > 0.0 {
                        reachable_distances.len() as f64 / total_distance
                    } else {
                        0.0
                    };
                    
                    if let Some(node) = graph.graph.node_weight(node_idx) {
                        // Ensure finite value
                        if closeness.is_finite() {
                            result.insert(node.id.clone(), closeness);
                        } else {
                            result.insert(node.id.clone(), 0.0);
                        }
                    }
                } else {
                    // Node is isolated
                    if let Some(node) = graph.graph.node_weight(node_idx) {
                        result.insert(node.id.clone(), 0.0);
                    }
                }
            } else {
                // Node is isolated
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    result.insert(node.id.clone(), 0.0);
                }
            }
        }
        
        result
    }

    /// Calculate eigenvector centrality
    fn calculate_eigenvector(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Calculating eigenvector centrality");
        
        let node_count = graph.graph.node_count();
        if node_count <= 1 {
            // Return default values for small graphs
            let mut result = HashMap::new();
            for node_idx in graph.graph.node_indices() {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    result.insert(node.id.clone(), 1.0);
                }
            }
            return result;
        }

        // Create adjacency matrix
        let mut adj_matrix = DMatrix::<f64>::zeros(node_count, node_count);
        let node_indices: Vec<NodeIndex> = graph.graph.node_indices().collect();
        let idx_map: HashMap<NodeIndex, usize> = node_indices
            .iter()
            .enumerate()
            .map(|(i, &idx)| (idx, i))
            .collect();

        // Fill adjacency matrix
        let mut has_edges = false;
        for edge in graph.graph.edge_references() {
            if let (Some(&i), Some(&j)) = (
                idx_map.get(&edge.source()),
                idx_map.get(&edge.target()),
            ) {
                // Avoid self-loops in eigenvector calculation
                if i != j {
                    adj_matrix[(i, j)] = edge.weight().weight;
                    has_edges = true;
                }
            }
        }

        // If graph has no edges, return uniform centrality
        if !has_edges {
            let mut result = HashMap::new();
            for node_idx in graph.graph.node_indices() {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    result.insert(node.id.clone(), 1.0 / node_count as f64);
                }
            }
            return result;
        }

        // Power iteration method with safety checks
        let mut eigenvector = DVector::<f64>::from_element(node_count, 1.0 / node_count as f64);
        let mut prev_eigenvalue = 0.0;
        let mut iterations = 0;

        while iterations < self.max_eigenvector_iterations {
            let new_vec = &adj_matrix * &eigenvector;
            let eigenvalue = new_vec.norm();
            
            if eigenvalue > 1e-10 {  // Use small epsilon instead of 0
                eigenvector = new_vec / eigenvalue;
                
                // Check convergence
                if (eigenvalue - prev_eigenvalue).abs() < 1e-6 {
                    break;
                }
                prev_eigenvalue = eigenvalue;
            } else {
                // Matrix has no dominant eigenvalue, use degree centrality fallback
                debug!("Eigenvector calculation failed, using degree centrality fallback");
                let degree_map = self.calculate_degree(graph);
                let mut result = HashMap::new();
                for (node_id, (in_deg, out_deg)) in degree_map {
                    result.insert(node_id, (in_deg + out_deg) / 2.0);
                }
                return result;
            }
            iterations += 1;
        }

        // Convert to HashMap with validation
        let mut result = HashMap::new();
        for (i, &node_idx) in node_indices.iter().enumerate() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                let value = eigenvector[i].abs();
                // Check for NaN or Infinity
                if value.is_finite() {
                    result.insert(node.id.clone(), value);
                } else {
                    result.insert(node.id.clone(), 0.0);
                }
            }
        }

        result
    }

    /// Calculate k-core decomposition
    fn calculate_k_core(&self, graph: &CodeGraph) -> HashMap<String, i64> {
        debug!("Calculating k-core decomposition");
        
        let mut result = HashMap::new();
        
        // Handle empty graph
        if graph.graph.node_count() == 0 {
            return result;
        }
        
        let mut working_graph = graph.graph.clone();
        let mut k = 0;
        let max_iterations = graph.graph.node_count().min(100); // Bounded by node count
        let mut iterations = 0;
        
        loop {
            iterations += 1;
            if iterations > max_iterations {
                debug!("K-core decomposition reached iteration limit");
                break;
            }
            
            let mut removed_any = false;
            let mut to_remove = Vec::new();
            
            // Find nodes with degree <= k
            for node_idx in working_graph.node_indices() {
                let degree = working_graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count()
                    + working_graph
                        .edges_directed(node_idx, Direction::Incoming)
                        .count();
                
                if degree <= k {
                    to_remove.push(node_idx);
                    removed_any = true;
                }
            }
            
            // Remove nodes
            for node_idx in to_remove {
                if let Some(original_node) = graph.graph.node_weight(node_idx) {
                    result.entry(original_node.id.clone()).or_insert(k as i64);
                }
                working_graph.remove_node(node_idx);
            }
            
            if !removed_any {
                if working_graph.node_count() == 0 {
                    break;
                }
                k += 1;
            }
        }
        
        // Set remaining nodes to highest k value (they form the k-core)
        for node_idx in working_graph.node_indices() {
            // We need to find the corresponding node in the original graph
            if let Some(working_node) = working_graph.node_weight(node_idx) {
                // Find matching node in original graph by ID
                for orig_idx in graph.graph.node_indices() {
                    if let Some(orig_node) = graph.graph.node_weight(orig_idx) {
                        if orig_node.id == working_node.id {
                            result.insert(orig_node.id.clone(), k as i64);
                            break;
                        }
                    }
                }
            }
        }
        
        result
    }

    /// Calculate clustering coefficient
    fn calculate_clustering(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Calculating clustering coefficient");
        
        let mut result = HashMap::new();
        
        for node_idx in graph.graph.node_indices() {
            let neighbors: HashSet<NodeIndex> = graph
                .graph
                .neighbors_undirected(node_idx)
                .collect();
            
            let neighbor_count = neighbors.len();
            if neighbor_count < 2 {
                if let Some(node) = graph.graph.node_weight(node_idx) {
                    result.insert(node.id.clone(), 0.0);
                }
                continue;
            }
            
            // Count edges between neighbors
            let mut triangle_count = 0;
            let neighbors_vec: Vec<NodeIndex> = neighbors.iter().copied().collect();
            
            for i in 0..neighbors_vec.len() {
                for j in (i + 1)..neighbors_vec.len() {
                    if graph.graph.find_edge(neighbors_vec[i], neighbors_vec[j]).is_some()
                        || graph.graph.find_edge(neighbors_vec[j], neighbors_vec[i]).is_some()
                    {
                        triangle_count += 1;
                    }
                }
            }
            
            let possible_triangles = neighbor_count * (neighbor_count - 1) / 2;
            let clustering = if possible_triangles > 0 {
                triangle_count as f64 / possible_triangles as f64
            } else {
                0.0
            };
            
            if let Some(node) = graph.graph.node_weight(node_idx) {
                result.insert(node.id.clone(), clustering);
            }
        }
        
        result
    }
}

impl Metric for CentralityMetrics {
    fn calculate(&self, graph: &CodeGraph) -> Result<MetricResults> {
        let mut results = MetricResults::new("centrality".to_string());

        // Calculate all centrality metrics
        let degree_centrality = self.calculate_degree(graph);
        let betweenness = self.calculate_betweenness(graph);
        let closeness = self.calculate_closeness(graph);
        let k_core = self.calculate_k_core(graph);
        let clustering = self.calculate_clustering(graph);

        // Store degree centrality
        for (node_id, (in_degree, out_degree)) in degree_centrality {
            results.add_value(
                format!("{}_in_degree", node_id),
                MetricValue::Float(in_degree),
            );
            results.add_value(
                format!("{}_out_degree", node_id),
                MetricValue::Float(out_degree),
            );
            results.add_value(
                format!("{}_degree", node_id),
                MetricValue::Float((in_degree + out_degree) / 2.0),
            );
        }

        // Store betweenness
        results.add_value("betweenness_map".to_string(), MetricValue::Map(betweenness.clone()));
        for (node_id, value) in betweenness {
            results.add_value(
                format!("{}_betweenness", node_id),
                MetricValue::Float(value),
            );
        }

        // Store closeness
        results.add_value("closeness_map".to_string(), MetricValue::Map(closeness.clone()));
        for (node_id, value) in closeness {
            results.add_value(
                format!("{}_closeness", node_id),
                MetricValue::Float(value),
            );
        }

        // Store k-core
        for (node_id, value) in k_core {
            results.add_value(
                format!("{}_k_core", node_id),
                MetricValue::Integer(value),
            );
        }

        // Store clustering
        results.add_value("clustering_map".to_string(), MetricValue::Map(clustering.clone()));
        for (node_id, value) in clustering {
            results.add_value(
                format!("{}_clustering", node_id),
                MetricValue::Float(value),
            );
        }

        // Calculate eigenvector if enabled
        if self.calculate_eigenvector {
            let eigenvector = self.calculate_eigenvector(graph);
            results.add_value("eigenvector_map".to_string(), MetricValue::Map(eigenvector.clone()));
            for (node_id, value) in eigenvector {
                results.add_value(
                    format!("{}_eigenvector", node_id),
                    MetricValue::Float(value),
                );
            }
        }

        Ok(results)
    }

    fn name(&self) -> &str {
        "centrality"
    }
}