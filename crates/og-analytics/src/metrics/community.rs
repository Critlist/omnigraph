use super::{Metric, MetricResults, MetricValue};
use anyhow::Result;
use og_graph::graph::CodeGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Community detection using Louvain algorithm
pub struct CommunityDetection {
    resolution: f64,
    max_iterations: usize,
}

impl CommunityDetection {
    pub fn new(resolution: f64) -> Self {
        Self {
            resolution,
            max_iterations: 100,
        }
    }

    /// Run Louvain algorithm for community detection
    fn louvain(&self, graph: &CodeGraph) -> HashMap<String, i64> {
        debug!("Running Louvain community detection");
        
        if graph.graph.node_count() == 0 {
            return HashMap::new();
        }

        // Initialize each node in its own community
        let mut communities: HashMap<NodeIndex, usize> = HashMap::new();
        let mut community_id = 0;
        for node_idx in graph.graph.node_indices() {
            communities.insert(node_idx, community_id);
            community_id += 1;
        }

        // Calculate total weight of edges (with safety checks)
        let total_weight = graph
            .graph
            .edge_indices()
            .filter_map(|e| graph.graph.edge_weight(e))
            .map(|w| {
                if w.weight.is_finite() && w.weight > 0.0 {
                    w.weight
                } else {
                    1.0
                }
            })
            .sum::<f64>();

        if total_weight <= 0.0 {
            debug!("Total weight is zero or negative, returning initial communities");
            return self.node_indices_to_string_map(graph, &communities);
        }

        let mut improvement = true;
        let mut iteration = 0;

        println!("[LOUVAIN] Starting iterations (max_iterations = {})", self.max_iterations);
        while improvement && iteration < self.max_iterations {
            improvement = false;
            iteration += 1;
            println!("[LOUVAIN] Iteration {} of {}", iteration, self.max_iterations);

            // Phase 1: Local optimization
            let node_count = graph.graph.node_count();
            println!("[LOUVAIN] Processing {} nodes in iteration {}", node_count, iteration);
            for (idx, node_idx) in graph.graph.node_indices().enumerate() {
                if idx % 100 == 0 {
                    println!("[LOUVAIN] Processing node {} of {}", idx, node_count);
                }
                let current_community = communities[&node_idx];
                let mut best_community = current_community;
                let mut best_gain = 0.0;

                // Get neighboring communities
                let neighbor_communities = self.get_neighbor_communities(
                    graph,
                    node_idx,
                    &communities,
                );

                // Calculate modularity gain for each neighboring community
                for &neighbor_community in &neighbor_communities {
                    if neighbor_community != current_community {
                        let gain = self.calculate_modularity_gain(
                            graph,
                            node_idx,
                            current_community,
                            neighbor_community,
                            &communities,
                            total_weight,
                        );

                        if gain > best_gain {
                            best_gain = gain;
                            best_community = neighbor_community;
                        }
                    }
                }

                // Move node to best community if there's improvement
                if best_community != current_community && best_gain.is_finite() {
                    communities.insert(node_idx, best_community);
                    improvement = true;
                }
            }

            // Phase 2: Community aggregation (simplified)
            if improvement && iteration % 5 == 0 {
                // Renumber communities to be contiguous
                let mut new_id = 0;
                let mut id_map = HashMap::new();
                
                for community in communities.values_mut() {
                    if !id_map.contains_key(community) {
                        id_map.insert(*community, new_id);
                        new_id += 1;
                    }
                    *community = *id_map.get(community).unwrap_or(community);
                }
            }
        }

        println!("[LOUVAIN] Completed after {} iterations", iteration);
        debug!("Louvain completed after {} iterations", iteration);
        self.node_indices_to_string_map(graph, &communities)
    }

    /// Get neighboring communities of a node
    fn get_neighbor_communities(
        &self,
        graph: &CodeGraph,
        node: NodeIndex,
        communities: &HashMap<NodeIndex, usize>,
    ) -> HashSet<usize> {
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

        neighbor_communities
    }

    /// Calculate modularity gain from moving a node to a different community
    fn calculate_modularity_gain(
        &self,
        graph: &CodeGraph,
        node: NodeIndex,
        _from_community: usize,
        to_community: usize,
        communities: &HashMap<NodeIndex, usize>,
        total_weight: f64,
    ) -> f64 {
        // Safety check
        if total_weight <= 0.0 {
            return 0.0;
        }

        // Calculate sum of weights of edges from node to nodes in to_community
        let mut ki_in = 0.0;
        for edge in graph.graph.edges_directed(node, petgraph::Direction::Outgoing) {
            if communities.get(&edge.target()) == Some(&to_community) {
                let weight = edge.weight().weight;
                if weight.is_finite() && weight > 0.0 {
                    ki_in += weight;
                }
            }
        }
        for edge in graph.graph.edges_directed(node, petgraph::Direction::Incoming) {
            if communities.get(&edge.source()) == Some(&to_community) {
                let weight = edge.weight().weight;
                if weight.is_finite() && weight > 0.0 {
                    ki_in += weight;
                }
            }
        }

        // Calculate sum of weights of all edges incident to node
        let ki = graph
            .graph
            .edges_directed(node, petgraph::Direction::Outgoing)
            .map(|e| {
                let w = e.weight().weight;
                if w.is_finite() && w > 0.0 { w } else { 0.0 }
            })
            .sum::<f64>()
            + graph
                .graph
                .edges_directed(node, petgraph::Direction::Incoming)
                .map(|e| {
                    let w = e.weight().weight;
                    if w.is_finite() && w > 0.0 { w } else { 0.0 }
                })
                .sum::<f64>();

        // Calculate sum of weights in to_community
        let sigma_tot = self.calculate_community_weight(graph, to_community, communities);

        // Modularity gain formula with safety checks
        let gain = (ki_in / total_weight) 
            - self.resolution * (sigma_tot * ki) / (total_weight * total_weight);

        // Ensure finite result
        if gain.is_finite() {
            gain
        } else {
            0.0
        }
    }

    /// Calculate total weight of edges in a community
    fn calculate_community_weight(
        &self,
        graph: &CodeGraph,
        community: usize,
        communities: &HashMap<NodeIndex, usize>,
    ) -> f64 {
        let mut weight = 0.0;

        for edge_idx in graph.graph.edge_indices() {
            if let Some((source, target)) = graph.graph.edge_endpoints(edge_idx) {
                if communities.get(&source) == Some(&community)
                    || communities.get(&target) == Some(&community)
                {
                    if let Some(edge) = graph.graph.edge_weight(edge_idx) {
                        weight += edge.weight;
                    }
                }
            }
        }

        weight
    }

    /// Convert node indices to string IDs
    fn node_indices_to_string_map(
        &self,
        graph: &CodeGraph,
        communities: &HashMap<NodeIndex, usize>,
    ) -> HashMap<String, i64> {
        let mut result = HashMap::new();

        for (node_idx, &community) in communities {
            if let Some(node) = graph.graph.node_weight(*node_idx) {
                result.insert(node.id.clone(), community as i64);
            }
        }

        result
    }

    /// Calculate modularity of current partition
    fn calculate_modularity(
        &self,
        graph: &CodeGraph,
        communities: &HashMap<String, i64>,
    ) -> f64 {
        let total_weight = graph
            .graph
            .edge_indices()
            .filter_map(|e| graph.graph.edge_weight(e))
            .map(|w| {
                if w.weight.is_finite() && w.weight > 0.0 {
                    w.weight
                } else {
                    1.0
                }
            })
            .sum::<f64>();

        if total_weight <= 0.0 {
            return 0.0;
        }

        let mut modularity = 0.0;

        // For each pair of nodes
        for edge_idx in graph.graph.edge_indices() {
            if let Some((source_idx, target_idx)) = graph.graph.edge_endpoints(edge_idx) {
                if let (Some(source), Some(target)) = (
                    graph.graph.node_weight(source_idx),
                    graph.graph.node_weight(target_idx),
                ) {
                    if let (Some(&source_comm), Some(&target_comm)) = (
                        communities.get(&source.id),
                        communities.get(&target.id),
                    ) {
                        if source_comm == target_comm {
                            if let Some(edge) = graph.graph.edge_weight(edge_idx) {
                                modularity += edge.weight / total_weight;
                            }
                        }
                    }
                }
            }
        }

        // Subtract expected edges
        for node_idx in graph.graph.node_indices() {
            let degree = graph
                .graph
                .edges_directed(node_idx, petgraph::Direction::Outgoing)
                .map(|e| e.weight().weight)
                .sum::<f64>()
                + graph
                    .graph
                    .edges_directed(node_idx, petgraph::Direction::Incoming)
                    .map(|e| e.weight().weight)
                    .sum::<f64>();

            let denominator = total_weight * total_weight;
            if denominator > 0.0 && degree.is_finite() {
                modularity -= self.resolution * (degree * degree) / denominator;
            }
        }

        // Ensure finite result
        if modularity.is_finite() {
            modularity
        } else {
            0.0
        }
    }

    /// Identify tightly coupled clusters
    fn identify_clusters(&self, communities: &HashMap<String, i64>) -> Vec<Vec<String>> {
        let mut clusters: HashMap<i64, Vec<String>> = HashMap::new();

        for (node_id, &community) in communities {
            clusters
                .entry(community)
                .or_insert_with(Vec::new)
                .push(node_id.clone());
        }

        // Sort clusters by size (largest first)
        let mut cluster_list: Vec<Vec<String>> = clusters.into_values().collect();
        cluster_list.sort_by_key(|c| std::cmp::Reverse(c.len()));

        cluster_list
    }

    /// Find natural boundaries for refactoring
    fn find_refactoring_boundaries(
        &self,
        graph: &CodeGraph,
        communities: &HashMap<String, i64>,
    ) -> HashMap<String, f64> {
        debug!("Finding refactoring boundaries");
        
        let mut boundary_scores = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                if let Some(&node_community) = communities.get(&node.id) {
                    // Count edges to other communities
                    let mut internal_edges = 0;
                    let mut external_edges = 0;

                    for edge in graph.graph.edges_directed(node_idx, petgraph::Direction::Outgoing) {
                        if let Some(target) = graph.graph.node_weight(edge.target()) {
                            if let Some(&target_community) = communities.get(&target.id) {
                                if target_community == node_community {
                                    internal_edges += 1;
                                } else {
                                    external_edges += 1;
                                }
                            }
                        }
                    }

                    for edge in graph.graph.edges_directed(node_idx, petgraph::Direction::Incoming) {
                        if let Some(source) = graph.graph.node_weight(edge.source()) {
                            if let Some(&source_community) = communities.get(&source.id) {
                                if source_community == node_community {
                                    internal_edges += 1;
                                } else {
                                    external_edges += 1;
                                }
                            }
                        }
                    }

                    // Boundary score: high external edges relative to internal
                    let total_edges = internal_edges + external_edges;
                    let boundary_score = if total_edges > 0 {
                        external_edges as f64 / total_edges as f64
                    } else {
                        0.0
                    };

                    boundary_scores.insert(node.id.clone(), boundary_score);
                }
            }
        }

        boundary_scores
    }
}

impl Metric for CommunityDetection {
    fn calculate(&self, graph: &CodeGraph) -> Result<MetricResults> {
        println!("[COMMUNITY] Starting community detection");
        let mut results = MetricResults::new("community".to_string());

        // Run Louvain algorithm
        println!("[COMMUNITY] Running Louvain algorithm on {} nodes...", graph.graph.node_count());
        let communities = self.louvain(graph);
        println!("[COMMUNITY] Louvain complete, found {} community assignments", communities.len());

        // Store community assignments
        println!("[COMMUNITY] Storing community assignments...");
        for (node_id, community) in &communities {
            results.add_value(
                format!("{}_community", node_id),
                MetricValue::Integer(*community),
            );
        }

        // Calculate modularity
        println!("[COMMUNITY] Calculating modularity...");
        let modularity = self.calculate_modularity(graph, &communities);
        println!("[COMMUNITY] Modularity = {}", modularity);
        results.add_value("modularity".to_string(), MetricValue::Float(modularity));

        // Identify clusters
        println!("[COMMUNITY] Identifying clusters...");
        let clusters = self.identify_clusters(&communities);
        println!("[COMMUNITY] Found {} clusters", clusters.len());
        results.add_value(
            "num_communities".to_string(),
            MetricValue::Integer(clusters.len() as i64),
        );

        // Store cluster sizes
        println!("[COMMUNITY] Storing cluster sizes...");
        for (i, cluster) in clusters.iter().enumerate() {
            results.add_value(
                format!("community_{}_size", i),
                MetricValue::Integer(cluster.len() as i64),
            );
        }

        // Find refactoring boundaries
        println!("[COMMUNITY] Finding refactoring boundaries...");
        let boundaries = self.find_refactoring_boundaries(graph, &communities);
        println!("[COMMUNITY] Found {} boundaries", boundaries.len());
        for (node_id, score) in boundaries {
            results.add_value(
                format!("{}_boundary_score", node_id),
                MetricValue::Float(score),
            );
        }

        println!("[COMMUNITY] Community detection complete");
        Ok(results)
    }

    fn name(&self) -> &str {
        "community"
    }
}