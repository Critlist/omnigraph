use og_graph::graph::CodeGraph;
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::debug;

/// Change impact analysis
#[derive(Debug, Clone)]
pub struct ImpactAnalysis {
    pub propagation_probability: HashMap<String, f64>,
    pub blast_radius: HashMap<String, BlastRadius>,
    pub dependency_depth: HashMap<String, usize>,
}

#[derive(Debug, Clone)]
pub struct BlastRadius {
    pub direct_impact: Vec<String>,
    pub indirect_impact: Vec<String>,
    pub total_affected: usize,
    pub severity: f64,
}

impl ImpactAnalysis {
    /// Perform impact analysis on the graph
    pub fn analyze(graph: &CodeGraph) -> Self {
        debug!("Performing impact analysis");
        
        let propagation_probability = Self::calculate_propagation_probability(graph);
        let blast_radius = Self::calculate_blast_radius(graph);
        let dependency_depth = Self::calculate_dependency_depth(graph);

        Self {
            propagation_probability,
            blast_radius,
            dependency_depth,
        }
    }

    /// Calculate propagation probability for each node
    fn calculate_propagation_probability(graph: &CodeGraph) -> HashMap<String, f64> {
        let mut probabilities = HashMap::new();
        let total_nodes = graph.graph.node_count() as f64;

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Use BFS to find all reachable nodes
                let mut visited = HashSet::new();
                let mut queue = VecDeque::new();
                queue.push_back((node_idx, 1.0));

                let mut total_probability = 0.0;
                let mut depth = 0;

                while !queue.is_empty() && depth < 5 {
                    let level_size = queue.len();
                    depth += 1;

                    for _ in 0..level_size {
                        if let Some((current, prob)) = queue.pop_front() {
                            if visited.contains(&current) {
                                continue;
                            }
                            visited.insert(current);

                            // Add to total probability (diminishing with distance)
                            total_probability += prob;

                            // Propagate to dependents with reduced probability
                            for edge in graph.graph.edges_directed(current, Direction::Incoming) {
                                let source = edge.source();
                                if !visited.contains(&source) {
                                    // Reduce probability by coupling strength
                                    let new_prob = prob * 0.7;  // 70% propagation factor
                                    queue.push_back((source, new_prob));
                                }
                            }
                        }
                    }
                }

                // Normalize by total nodes
                let normalized_prob = (total_probability / total_nodes).min(1.0);
                probabilities.insert(node.id.clone(), normalized_prob);
            }
        }

        probabilities
    }

    /// Calculate blast radius for each node
    fn calculate_blast_radius(graph: &CodeGraph) -> HashMap<String, BlastRadius> {
        let mut blast_radii = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                let mut direct_impact = Vec::new();
                let mut indirect_impact = Vec::new();
                let mut visited = HashSet::new();

                // Direct impact: immediate dependents
                for edge in graph.graph.edges_directed(node_idx, Direction::Incoming) {
                    let dependent = edge.source();
                    if let Some(dep_node) = graph.graph.node_weight(dependent) {
                        direct_impact.push(dep_node.id.clone());
                        visited.insert(dependent);
                    }
                }

                // Indirect impact: transitive dependents (up to depth 3)
                let mut queue = VecDeque::new();
                for &dependent in &visited {
                    queue.push_back((dependent, 1));
                }

                while let Some((current, depth)) = queue.pop_front() {
                    if depth >= 3 {
                        continue;
                    }

                    for edge in graph.graph.edges_directed(current, Direction::Incoming) {
                        let dependent = edge.source();
                        if !visited.contains(&dependent) {
                            visited.insert(dependent);
                            if let Some(dep_node) = graph.graph.node_weight(dependent) {
                                indirect_impact.push(dep_node.id.clone());
                            }
                            queue.push_back((dependent, depth + 1));
                        }
                    }
                }

                let total_affected = direct_impact.len() + indirect_impact.len();
                let severity = Self::calculate_severity(
                    direct_impact.len(),
                    indirect_impact.len(),
                    graph.graph.node_count(),
                );

                blast_radii.insert(
                    node.id.clone(),
                    BlastRadius {
                        direct_impact,
                        indirect_impact,
                        total_affected,
                        severity,
                    },
                );
            }
        }

        blast_radii
    }

    /// Calculate severity score based on impact
    fn calculate_severity(direct: usize, indirect: usize, total_nodes: usize) -> f64 {
        if total_nodes == 0 {
            return 0.0;
        }

        let direct_weight = 1.0;
        let indirect_weight = 0.5;

        let weighted_impact = direct as f64 * direct_weight + indirect as f64 * indirect_weight;
        let normalized = weighted_impact / total_nodes as f64;

        // Apply non-linear scaling for severity
        if normalized < 0.1 {
            normalized * 2.0  // Low impact
        } else if normalized < 0.3 {
            0.2 + (normalized - 0.1) * 3.0  // Medium impact
        } else {
            0.8 + (normalized - 0.3) * 0.5  // High impact
        }
        .min(1.0)
    }

    /// Calculate dependency depth for each node
    fn calculate_dependency_depth(graph: &CodeGraph) -> HashMap<String, usize> {
        let mut depths = HashMap::new();

        // Find root nodes (no outgoing dependencies)
        let roots: Vec<_> = graph
            .graph
            .node_indices()
            .filter(|&idx| {
                graph
                    .graph
                    .edges_directed(idx, Direction::Outgoing)
                    .count()
                    == 0
            })
            .collect();

        // BFS from roots to calculate depths
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        for root in roots {
            queue.push_back((root, 0));
        }

        while let Some((current, depth)) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);

            if let Some(node) = graph.graph.node_weight(current) {
                depths
                    .entry(node.id.clone())
                    .and_modify(|d: &mut usize| *d = (*d).max(depth))
                    .or_insert(depth);
            }

            for edge in graph.graph.edges_directed(current, Direction::Incoming) {
                let dependent = edge.source();
                if !visited.contains(&dependent) {
                    queue.push_back((dependent, depth + 1));
                }
            }
        }

        // Handle nodes not reachable from roots
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                depths.entry(node.id.clone()).or_insert(0);
            }
        }

        depths
    }

    /// Get high-impact nodes (top N by blast radius)
    pub fn get_high_impact_nodes(&self, limit: usize) -> Vec<(&String, &BlastRadius)> {
        let mut nodes: Vec<_> = self.blast_radius.iter().collect();
        nodes.sort_by(|a, b| {
            b.1.severity
                .partial_cmp(&a.1.severity)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        nodes.truncate(limit);
        nodes
    }

    /// Get deep dependency chains
    pub fn get_deep_dependencies(&self, min_depth: usize) -> Vec<(&String, &usize)> {
        self.dependency_depth
            .iter()
            .filter(|(_, &depth)| depth >= min_depth)
            .collect()
    }
}