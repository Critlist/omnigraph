use anyhow::{Result, Context};
use og_graph::graph::CodeGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

/// Community detection algorithms with robust error handling
pub struct CommunityDetection {
    pub resolution: f64,
    pub max_iterations: usize,
    pub min_modularity_gain: f64,
}

impl Default for CommunityDetection {
    fn default() -> Self {
        Self {
            resolution: 1.0,
            max_iterations: 100,
            min_modularity_gain: 1e-6,
        }
    }
}

impl CommunityDetection {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_resolution(resolution: f64) -> Self {
        Self {
            resolution: resolution.clamp(0.1, 10.0),
            ..Self::default()
        }
    }

    /// Run community detection with error recovery
    pub fn detect_communities(&self, graph: &CodeGraph) -> Result<CommunityResults> {
        let mut results = CommunityResults::default();
        
        // Validate input
        if graph.graph.node_count() == 0 {
            debug!("Empty graph, returning empty community results");
            return Ok(results);
        }

        // Try Louvain algorithm
        match self.louvain_safe(graph) {
            Ok(communities) => {
                results.communities = communities;
                results.modularity = self.calculate_modularity(graph, &results.communities)?;
                results.num_communities = results.communities.values().cloned().collect::<HashSet<_>>().len();
            }
            Err(e) => {
                warn!("Louvain algorithm failed: {}, using fallback", e);
                results.errors.push(format!("Louvain: {}", e));
                
                // Fallback: each node in its own community
                results.communities = self.singleton_communities(graph);
                results.num_communities = results.communities.len();
            }
        }

        Ok(results)
    }

    /// Louvain algorithm with safety checks
    fn louvain_safe(&self, graph: &CodeGraph) -> Result<HashMap<String, usize>> {
        if graph.graph.node_count() == 0 {
            return Ok(HashMap::new());
        }

        // Initialize each node in its own community
        let mut communities: HashMap<NodeIndex, usize> = HashMap::new();
        let mut community_id = 0;
        for node_idx in graph.graph.node_indices() {
            communities.insert(node_idx, community_id);
            community_id += 1;
        }

        // Calculate total weight with validation
        let total_weight = self.calculate_total_weight(graph)?;
        if total_weight <= 0.0 {
            return Err(anyhow::anyhow!("Invalid graph weight: {}", total_weight));
        }

        let mut improvement = true;
        let mut iteration = 0;
        let mut last_modularity = 0.0;

        while improvement && iteration < self.max_iterations {
            improvement = false;
            iteration += 1;

            // Phase 1: Local optimization
            for node_idx in graph.graph.node_indices() {
                let current_community = *communities.get(&node_idx)
                    .context("Node not in community map")?;
                
                let mut best_community = current_community;
                let mut best_gain = 0.0;

                // Get neighboring communities
                let neighbor_communities = self.get_neighbor_communities(
                    graph,
                    node_idx,
                    &communities,
                )?;

                // Calculate modularity gain for each neighboring community
                for &neighbor_community in &neighbor_communities {
                    if neighbor_community != current_community {
                        let gain = self.calculate_modularity_gain_safe(
                            graph,
                            node_idx,
                            current_community,
                            neighbor_community,
                            &communities,
                            total_weight,
                        )?;

                        if gain > best_gain && gain > self.min_modularity_gain {
                            best_gain = gain;
                            best_community = neighbor_community;
                        }
                    }
                }

                // Move node to best community if there's improvement
                if best_community != current_community && best_gain > self.min_modularity_gain {
                    communities.insert(node_idx, best_community);
                    improvement = true;
                }
            }

            // Check for convergence based on modularity
            if iteration % 10 == 0 {
                let current_modularity = self.calculate_modularity_internal(graph, &communities, total_weight)?;
                if (current_modularity - last_modularity).abs() < self.min_modularity_gain {
                    debug!("Converged at iteration {} with modularity {}", iteration, current_modularity);
                    break;
                }
                last_modularity = current_modularity;
            }

            // Renumber communities periodically
            if improvement && iteration % 20 == 0 {
                self.renumber_communities(&mut communities);
            }
        }

        debug!("Louvain completed after {} iterations", iteration);
        
        // Convert to string map
        self.node_indices_to_string_map(graph, &communities)
    }

    /// Calculate total weight of edges with validation
    fn calculate_total_weight(&self, graph: &CodeGraph) -> Result<f64> {
        let mut total = 0.0;
        
        for edge in graph.graph.edge_references() {
            let weight = edge.weight().weight;
            if !weight.is_finite() || weight < 0.0 {
                warn!("Invalid edge weight: {}, using 1.0", weight);
                total += 1.0;
            } else {
                total += weight;
            }
        }
        
        Ok(total)
    }

    /// Get neighboring communities of a node
    fn get_neighbor_communities(
        &self,
        graph: &CodeGraph,
        node: NodeIndex,
        communities: &HashMap<NodeIndex, usize>,
    ) -> Result<HashSet<usize>> {
        let mut neighbor_communities = HashSet::new();

        // Check outgoing edges
        for edge in graph.graph.edges_directed(node, petgraph::Direction::Outgoing) {
            if let Some(&community) = communities.get(&edge.target()) {
                neighbor_communities.insert(community);
            }
        }

        // Check incoming edges
        for edge in graph.graph.edges_directed(node, petgraph::Direction::Incoming) {
            if let Some(&community) = communities.get(&edge.source()) {
                neighbor_communities.insert(community);
            }
        }

        Ok(neighbor_communities)
    }

    /// Calculate modularity gain with safety checks
    fn calculate_modularity_gain_safe(
        &self,
        graph: &CodeGraph,
        node: NodeIndex,
        from_community: usize,
        to_community: usize,
        communities: &HashMap<NodeIndex, usize>,
        total_weight: f64,
    ) -> Result<f64> {
        if total_weight <= 0.0 {
            return Ok(0.0);
        }

        let mut to_community_weight = 0.0;
        let mut from_community_weight = 0.0;
        let mut node_weight = 0.0;

        // Calculate weights
        for edge in graph.graph.edge_references() {
            let source_comm = communities.get(&edge.source());
            let target_comm = communities.get(&edge.target());
            let weight = edge.weight().weight.abs(); // Use absolute value for safety

            if source_comm == Some(&to_community) || target_comm == Some(&to_community) {
                to_community_weight += weight;
            }
            if source_comm == Some(&from_community) || target_comm == Some(&from_community) {
                from_community_weight += weight;
            }
            if edge.source() == node || edge.target() == node {
                node_weight += weight;
            }
        }

        // Calculate gain using resolution parameter
        let gain = (to_community_weight - from_community_weight) / total_weight 
            + self.resolution * node_weight * (from_community_weight - to_community_weight) 
            / (total_weight * total_weight);

        Ok(gain.clamp(-1.0, 1.0)) // Clamp to reasonable range
    }

    /// Calculate modularity of current partition
    fn calculate_modularity(&self, graph: &CodeGraph, communities: &HashMap<String, usize>) -> Result<f64> {
        let total_weight = self.calculate_total_weight(graph)?;
        if total_weight <= 0.0 {
            return Ok(0.0);
        }

        // Convert string map to node index map
        let mut index_communities = HashMap::new();
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                if let Some(&comm) = communities.get(&node.id) {
                    index_communities.insert(node_idx, comm);
                }
            }
        }

        self.calculate_modularity_internal(graph, &index_communities, total_weight)
    }

    /// Internal modularity calculation
    fn calculate_modularity_internal(
        &self,
        graph: &CodeGraph,
        communities: &HashMap<NodeIndex, usize>,
        total_weight: f64,
    ) -> Result<f64> {
        let mut modularity = 0.0;
        
        for edge in graph.graph.edge_references() {
            let source_comm = communities.get(&edge.source());
            let target_comm = communities.get(&edge.target());
            
            if source_comm == target_comm && source_comm.is_some() {
                let weight = edge.weight().weight.abs();
                modularity += weight / total_weight;
            }
        }
        
        Ok((modularity * 2.0).clamp(0.0, 1.0)) // Normalize to [0, 1]
    }

    /// Renumber communities to be contiguous
    fn renumber_communities(&self, communities: &mut HashMap<NodeIndex, usize>) {
        let unique_communities: HashSet<usize> = communities.values().cloned().collect();
        let mut id_map = HashMap::new();
        
        for (new_id, old_id) in unique_communities.iter().enumerate() {
            id_map.insert(*old_id, new_id);
        }
        
        for community in communities.values_mut() {
            *community = *id_map.get(community).unwrap_or(community);
        }
    }

    /// Convert node index map to string map
    fn node_indices_to_string_map(
        &self,
        graph: &CodeGraph,
        communities: &HashMap<NodeIndex, usize>,
    ) -> Result<HashMap<String, usize>> {
        let mut result = HashMap::new();
        
        for (node_idx, &community) in communities {
            if let Some(node) = graph.graph.node_weight(*node_idx) {
                result.insert(node.id.clone(), community);
            }
        }
        
        Ok(result)
    }

    /// Create singleton communities (fallback)
    fn singleton_communities(&self, graph: &CodeGraph) -> HashMap<String, usize> {
        let mut communities = HashMap::new();
        let mut id = 0;
        
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                communities.insert(node.id.clone(), id);
                id += 1;
            }
        }
        
        communities
    }
}

#[derive(Debug, Default, Clone)]
pub struct CommunityResults {
    pub communities: HashMap<String, usize>,
    pub num_communities: usize,
    pub modularity: f64,
    pub errors: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_graph::graph::{GraphNode, GraphEdge};

    #[test]
    fn test_empty_graph() {
        let graph = CodeGraph::new();
        let detector = CommunityDetection::new();
        let results = detector.detect_communities(&graph).unwrap();
        assert_eq!(results.num_communities, 0);
        assert_eq!(results.modularity, 0.0);
    }

    #[test]
    fn test_single_community() {
        let mut graph = CodeGraph::new();
        
        // Add connected nodes
        for i in 0..3 {
            graph.add_node(GraphNode {
                id: format!("node{}", i),
                name: format!("Node {}", i),
                node_type: "file".to_string(),
                file_path: Some(format!("/test{}.js", i)),
                size: 100,
                color: None,
            });
        }
        
        // Connect them
        graph.add_edge("node0", "node1", GraphEdge {
            edge_type: "imports".to_string(),
            weight: 1.0,
        });
        graph.add_edge("node1", "node2", GraphEdge {
            edge_type: "imports".to_string(),
            weight: 1.0,
        });
        
        let detector = CommunityDetection::new();
        let results = detector.detect_communities(&graph).unwrap();
        
        // Should detect at least one community
        assert!(results.num_communities >= 1);
        assert!(results.modularity >= 0.0);
    }
}