use super::{Metric, MetricResults, MetricValue};
use anyhow::Result;
use og_graph::graph::CodeGraph;
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use tracing::debug;

/// Code quality metrics calculator
pub struct QualityMetrics {
    calculate_cohesion: bool,
}

impl QualityMetrics {
    pub fn new() -> Self {
        Self {
            calculate_cohesion: true,
        }
    }

    /// Calculate coupling metrics (afferent and efferent)
    fn calculate_coupling(&self, graph: &CodeGraph) -> HashMap<String, CouplingMetrics> {
        debug!("Calculating coupling metrics");
        
        let mut coupling_map = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Skip non-module nodes for coupling
                if !["file", "module", "class"].contains(&node.node_type.as_str()) {
                    continue;
                }

                // Afferent coupling: number of modules that depend on this module
                let mut afferent_sources = HashSet::new();
                for edge in graph.graph.edges_directed(node_idx, Direction::Incoming) {
                    let source_idx = edge.source();
                    if let Some(source) = graph.graph.node_weight(source_idx) {
                        if ["file", "module", "class"].contains(&source.node_type.as_str()) {
                            afferent_sources.insert(source.id.clone());
                        }
                    }
                }
                let afferent = afferent_sources.len();

                // Efferent coupling: number of modules this module depends on
                let mut efferent_targets = HashSet::new();
                for edge in graph.graph.edges_directed(node_idx, Direction::Outgoing) {
                    let target_idx = edge.target();
                    if let Some(target) = graph.graph.node_weight(target_idx) {
                        if ["file", "module", "class"].contains(&target.node_type.as_str()) {
                            efferent_targets.insert(target.id.clone());
                        }
                    }
                }
                let efferent = efferent_targets.len();

                // Instability: efferent / (afferent + efferent)
                let total = afferent + efferent;
                let instability = if total > 0 {
                    efferent as f64 / total as f64
                } else {
                    0.0
                };

                coupling_map.insert(
                    node.id.clone(),
                    CouplingMetrics {
                        afferent: afferent as f64,
                        efferent: efferent as f64,
                        instability,
                    },
                );
            }
        }

        coupling_map
    }

    /// Calculate cohesion for classes and modules
    fn calculate_cohesion(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Calculating cohesion metrics");
        
        let mut cohesion_map = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Only calculate for classes and modules
                if !["class", "module"].contains(&node.node_type.as_str()) {
                    continue;
                }

                // Get all internal components (functions/methods)
                let internal_nodes: Vec<_> = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .filter_map(|edge| {
                        let target_idx = edge.target();
                        let target = graph.graph.node_weight(target_idx)?;
                        if target.node_type == "function" || target.node_type == "method" {
                            Some(target_idx)
                        } else {
                            None
                        }
                    })
                    .collect();

                if internal_nodes.len() < 2 {
                    cohesion_map.insert(node.id.clone(), 1.0);
                    continue;
                }

                // Calculate connections between internal components
                let mut internal_connections = 0;
                for &src in &internal_nodes {
                    for &tgt in &internal_nodes {
                        if src != tgt {
                            // Check if there's a connection (call or reference)
                            if graph.graph.find_edge(src, tgt).is_some() {
                                internal_connections += 1;
                            }
                        }
                    }
                }

                // Maximum possible connections
                let max_connections = internal_nodes.len() * (internal_nodes.len() - 1);
                
                let cohesion = if max_connections > 0 {
                    internal_connections as f64 / max_connections as f64
                } else {
                    1.0
                };

                cohesion_map.insert(node.id.clone(), cohesion);
            }
        }

        cohesion_map
    }

    /// Calculate complexity aggregation
    fn calculate_complexity(&self, graph: &CodeGraph) -> HashMap<String, ComplexityMetrics> {
        debug!("Calculating complexity metrics");
        
        let mut complexity_map = HashMap::new();

        // First pass: collect base complexities
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Base complexity based on node type and connections
                let edge_count = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();
                
                let base_complexity = match node.node_type.as_str() {
                    "function" | "method" => {
                        // Cyclomatic complexity approximation: edges + 1
                        (edge_count + 1) as f64
                    }
                    "class" => {
                        // Class complexity: number of methods + fields
                        let method_count = graph
                            .graph
                            .edges_directed(node_idx, Direction::Outgoing)
                            .filter(|edge| {
                                graph.graph.node_weight(edge.target())
                                    .map(|n| n.node_type == "method" || n.node_type == "function")
                                    .unwrap_or(false)
                            })
                            .count();
                        (method_count * 2 + edge_count) as f64
                    }
                    "file" | "module" => {
                        // File complexity: sum of contained complexities
                        let contained_complexity: f64 = graph
                            .graph
                            .edges_directed(node_idx, Direction::Outgoing)
                            .filter_map(|edge| {
                                let target = graph.graph.node_weight(edge.target())?;
                                match target.node_type.as_str() {
                                    "function" => Some(5.0),  // Base function complexity
                                    "class" => Some(10.0),     // Base class complexity
                                    _ => Some(1.0),
                                }
                            })
                            .sum();
                        contained_complexity
                    }
                    _ => 1.0,
                };

                // Calculate depth in hierarchy
                let depth = self.calculate_depth(graph, node_idx);
                
                // Weighted complexity considering depth
                let weighted_complexity = base_complexity * (1.0 + depth as f64 * 0.1);

                complexity_map.insert(
                    node.id.clone(),
                    ComplexityMetrics {
                        cyclomatic: base_complexity,
                        cognitive: weighted_complexity,
                        depth,
                        dependencies: edge_count as f64,
                    },
                );
            }
        }

        complexity_map
    }

    /// Calculate depth of a node in the graph
    fn calculate_depth(&self, graph: &CodeGraph, node_idx: petgraph::graph::NodeIndex) -> usize {
        let mut visited = HashSet::new();
        let mut queue = vec![(node_idx, 0)];
        let mut max_depth = 0;

        while let Some((current, depth)) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current);
            max_depth = max_depth.max(depth);

            for edge in graph.graph.edges_directed(current, Direction::Incoming) {
                queue.push((edge.source(), depth + 1));
            }
        }

        max_depth
    }

    /// Calculate maintainability index
    fn calculate_maintainability(&self, complexity: &HashMap<String, ComplexityMetrics>) -> HashMap<String, f64> {
        let mut maintainability = HashMap::new();
        
        for (node_id, metrics) in complexity {
            // Simplified maintainability index formula
            // Higher is better, range approximately 0-100
            let halstead_volume = metrics.dependencies * (metrics.dependencies + 2.0).ln();
            let lines_estimate = metrics.cyclomatic * 10.0;  // Rough estimate
            
            let mi = 171.0
                - 5.2 * halstead_volume.ln().max(0.0)
                - 0.23 * metrics.cyclomatic
                - 16.2 * lines_estimate.ln().max(0.0);
            
            // Normalize to 0-1 range
            let normalized = (mi / 100.0).clamp(0.0, 1.0);
            maintainability.insert(node_id.clone(), normalized);
        }
        
        maintainability
    }
}

#[derive(Debug, Clone)]
struct CouplingMetrics {
    afferent: f64,
    efferent: f64,
    instability: f64,
}

#[derive(Debug, Clone)]
struct ComplexityMetrics {
    cyclomatic: f64,
    cognitive: f64,
    depth: usize,
    dependencies: f64,
}

impl Metric for QualityMetrics {
    fn calculate(&self, graph: &CodeGraph) -> Result<MetricResults> {
        let mut results = MetricResults::new("quality".to_string());

        // Calculate coupling metrics
        let coupling = self.calculate_coupling(graph);
        for (node_id, metrics) in &coupling {
            results.add_value(
                format!("{}_afferent_coupling", node_id),
                MetricValue::Float(metrics.afferent),
            );
            results.add_value(
                format!("{}_efferent_coupling", node_id),
                MetricValue::Float(metrics.efferent),
            );
            results.add_value(
                format!("{}_instability", node_id),
                MetricValue::Float(metrics.instability),
            );
        }

        // Calculate cohesion if enabled
        if self.calculate_cohesion {
            let cohesion = self.calculate_cohesion(graph);
            for (node_id, value) in cohesion {
                results.add_value(
                    format!("{}_cohesion", node_id),
                    MetricValue::Float(value),
                );
            }
        }

        // Calculate complexity
        let complexity = self.calculate_complexity(graph);
        for (node_id, metrics) in &complexity {
            results.add_value(
                format!("{}_cyclomatic_complexity", node_id),
                MetricValue::Float(metrics.cyclomatic),
            );
            results.add_value(
                format!("{}_cognitive_complexity", node_id),
                MetricValue::Float(metrics.cognitive),
            );
            results.add_value(
                format!("{}_depth", node_id),
                MetricValue::Integer(metrics.depth as i64),
            );
        }

        // Calculate maintainability
        let maintainability = self.calculate_maintainability(&complexity);
        for (node_id, value) in maintainability {
            results.add_value(
                format!("{}_maintainability", node_id),
                MetricValue::Float(value),
            );
        }

        Ok(results)
    }

    fn name(&self) -> &str {
        "quality"
    }
}