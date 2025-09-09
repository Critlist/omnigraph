use super::{Metric, MetricResults, MetricValue};
use anyhow::Result;
use og_graph::graph::CodeGraph;
use petgraph::algo::tarjan_scc;
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::debug;

/// Risk analysis metrics
pub struct RiskAnalysis {
    #[allow(dead_code)]
    churn_threshold: f64,
    complexity_threshold: f64,
}

impl RiskAnalysis {
    pub fn new() -> Self {
        Self {
            churn_threshold: 10.0,
            complexity_threshold: 15.0,
        }
    }

    /// Identify high-risk nodes (high complexity + high centrality)
    fn identify_high_risk_nodes(&self, graph: &CodeGraph) -> HashMap<String, RiskScore> {
        debug!("Identifying high-risk nodes");
        
        let mut risk_scores = HashMap::new();

        // Calculate basic metrics for risk assessment
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Calculate degree centrality
                let in_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count();
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();
                
                let total_degree = in_degree + out_degree;
                let node_count = graph.graph.node_count().max(1);
                let normalized_degree = total_degree as f64 / node_count as f64;

                // Estimate complexity based on node type and connections
                let complexity_estimate = match node.node_type.as_str() {
                    "function" | "method" => (out_degree + 1) as f64,
                    "class" => (out_degree * 2) as f64,
                    "file" | "module" => {
                        // Files with many dependencies are complex
                        (out_degree as f64).sqrt() * 5.0
                    }
                    _ => 1.0,
                };

                // Calculate risk factors
                let complexity_risk = (complexity_estimate / self.complexity_threshold).min(1.0);
                let centrality_risk = normalized_degree.min(1.0);
                
                // Check if node is a single point of failure
                let is_bottleneck = in_degree > 5 && out_degree > 5;
                let bottleneck_risk = if is_bottleneck { 0.5 } else { 0.0 };

                // Combined risk score
                let risk_score = (complexity_risk * 0.4 
                    + centrality_risk * 0.4 
                    + bottleneck_risk * 0.2)
                    .clamp(0.0, 1.0);

                risk_scores.insert(
                    node.id.clone(),
                    RiskScore {
                        overall: risk_score,
                        complexity: complexity_risk,
                        centrality: centrality_risk,
                        bottleneck: bottleneck_risk,
                    },
                );
            }
        }

        risk_scores
    }

    /// Find architectural chokepoints
    fn find_chokepoints(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Finding architectural chokepoints");
        
        let mut chokepoints = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Simplified chokepoint detection based on betweenness-like metric
                let in_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count();
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();
                
                // A node is a chokepoint if it has high in and out degree
                // and is on many paths (approximation)
                let total_degree = in_degree + out_degree;
                let node_count = graph.graph.node_count().max(1);
                
                // Simple heuristic: nodes with many connections in both directions
                // are likely chokepoints
                let chokepoint_score = if in_degree > 0 && out_degree > 0 {
                    let balance = (in_degree.min(out_degree) as f64) / (in_degree.max(out_degree).max(1) as f64);
                    let connectivity = (total_degree as f64) / (node_count as f64);
                    (balance * connectivity).min(1.0)
                } else {
                    0.0
                };

                chokepoints.insert(node.id.clone(), chokepoint_score);
            }
        }

        chokepoints
    }

    /// Detect circular dependencies using Tarjan's algorithm
    fn detect_circular_dependencies(&self, graph: &CodeGraph) -> Vec<Vec<String>> {
        debug!("Detecting circular dependencies");
        
        // Find strongly connected components
        let sccs = tarjan_scc(&graph.graph);
        
        let mut circular_deps = Vec::new();
        
        for scc in sccs {
            // Only consider SCCs with more than one node (cycles)
            if scc.len() > 1 {
                let cycle: Vec<String> = scc
                    .iter()
                    .filter_map(|&idx| {
                        graph.graph.node_weight(idx).map(|n| n.id.clone())
                    })
                    .collect();
                
                if !cycle.is_empty() {
                    circular_deps.push(cycle);
                }
            }
        }

        circular_deps
    }

    /// Calculate technical debt score
    fn calculate_technical_debt(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Calculating technical debt");
        
        let mut debt_scores = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                let mut debt_factors = Vec::new();

                // Factor 1: High coupling
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();
                let coupling_debt = (out_degree as f64 / 10.0).min(1.0);
                debt_factors.push(coupling_debt);

                // Factor 2: Deep nesting (approximated by path length from root)
                let depth = self.calculate_max_depth_from_roots(graph, node_idx);
                let depth_debt = (depth as f64 / 10.0).min(1.0);
                debt_factors.push(depth_debt);

                // Factor 3: Size (for files/classes)
                let size_factor = match node.node_type.as_str() {
                    "file" | "module" => {
                        let contained = graph
                            .graph
                            .edges_directed(node_idx, Direction::Outgoing)
                            .count();
                        (contained as f64 / 20.0).min(1.0)
                    }
                    "class" => {
                        let methods = graph
                            .graph
                            .edges_directed(node_idx, Direction::Outgoing)
                            .filter(|edge| {
                                graph.graph.node_weight(edge.target())
                                    .map(|n| n.node_type == "method")
                                    .unwrap_or(false)
                            })
                            .count();
                        (methods as f64 / 15.0).min(1.0)
                    }
                    _ => 0.0,
                };
                debt_factors.push(size_factor);

                // Calculate average debt
                let debt_score = if !debt_factors.is_empty() {
                    debt_factors.iter().sum::<f64>() / debt_factors.len() as f64
                } else {
                    0.0
                };

                debt_scores.insert(node.id.clone(), debt_score);
            }
        }

        debt_scores
    }

    /// Calculate maximum depth from root nodes
    fn calculate_max_depth_from_roots(
        &self,
        graph: &CodeGraph,
        target: petgraph::graph::NodeIndex,
    ) -> usize {
        // Find potential root nodes (nodes with no incoming edges)
        let roots: Vec<_> = graph
            .graph
            .node_indices()
            .filter(|&idx| {
                graph
                    .graph
                    .edges_directed(idx, Direction::Incoming)
                    .count()
                    == 0
            })
            .collect();

        if roots.is_empty() {
            return 0;
        }

        let mut max_depth = 0;

        for root in roots {
            // BFS from root to target
            let mut visited = HashSet::new();
            let mut queue = VecDeque::new();
            queue.push_back((root, 0));

            while let Some((current, depth)) = queue.pop_front() {
                if current == target {
                    max_depth = max_depth.max(depth);
                    continue;
                }

                if visited.contains(&current) {
                    continue;
                }
                visited.insert(current);

                for edge in graph.graph.edges_directed(current, Direction::Outgoing) {
                    queue.push_back((edge.target(), depth + 1));
                }
            }
        }

        max_depth
    }

    /// Calculate change propagation probability
    fn calculate_change_propagation(&self, graph: &CodeGraph) -> HashMap<String, f64> {
        debug!("Calculating change propagation probability");
        
        let mut propagation_scores = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Calculate how changes to this node might propagate
                let direct_dependents = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count();
                
                // Estimate transitive dependents (simplified)
                let mut visited = HashSet::new();
                let mut queue = VecDeque::new();
                queue.push_back(node_idx);
                
                let mut transitive_count = 0;
                while let Some(current) = queue.pop_front() {
                    if visited.contains(&current) {
                        continue;
                    }
                    visited.insert(current);
                    
                    for edge in graph.graph.edges_directed(current, Direction::Incoming) {
                        let source = edge.source();
                        if !visited.contains(&source) {
                            transitive_count += 1;
                            queue.push_back(source);
                        }
                    }
                }

                // Propagation probability based on dependent count
                let total_nodes = graph.graph.node_count().max(1) as f64;
                let propagation = ((direct_dependents + transitive_count / 2) as f64 / total_nodes)
                    .clamp(0.0, 1.0);

                propagation_scores.insert(node.id.clone(), propagation);
            }
        }

        propagation_scores
    }
}

#[derive(Debug, Clone)]
struct RiskScore {
    overall: f64,
    complexity: f64,
    centrality: f64,
    bottleneck: f64,
}

impl Metric for RiskAnalysis {
    fn calculate(&self, graph: &CodeGraph) -> Result<MetricResults> {
        println!("[RISK] Starting risk analysis");
        let mut results = MetricResults::new("risk".to_string());

        // Calculate risk scores
        println!("[RISK] Identifying high-risk nodes...");
        let risk_scores = self.identify_high_risk_nodes(graph);
        println!("[RISK] Found {} high-risk nodes", risk_scores.len());
        for (node_id, scores) in risk_scores {
            results.add_value(
                format!("{}_risk", node_id),
                MetricValue::Float(scores.overall),
            );
            results.add_value(
                format!("{}_complexity_risk", node_id),
                MetricValue::Float(scores.complexity),
            );
            results.add_value(
                format!("{}_centrality_risk", node_id),
                MetricValue::Float(scores.centrality),
            );
            results.add_value(
                format!("{}_bottleneck_risk", node_id),
                MetricValue::Float(scores.bottleneck),
            );
        }

        // Find chokepoints
        println!("[RISK] Finding chokepoints...");
        let chokepoints = self.find_chokepoints(graph);
        println!("[RISK] Found {} chokepoints", chokepoints.len());
        for (node_id, score) in chokepoints {
            results.add_value(
                format!("{}_chokepoint", node_id),
                MetricValue::Float(score),
            );
        }

        // Detect circular dependencies
        println!("[RISK] Detecting circular dependencies...");
        let circular_deps = self.detect_circular_dependencies(graph);
        println!("[RISK] Found {} circular dependencies", circular_deps.len());
        results.add_value(
            "circular_dependencies".to_string(),
            MetricValue::Integer(circular_deps.len() as i64),
        );

        // Store circular dependency groups
        for (i, cycle) in circular_deps.iter().enumerate() {
            let cycle_map: HashMap<String, f64> = cycle
                .iter()
                .enumerate()
                .map(|(j, id)| (id.clone(), j as f64))
                .collect();
            results.add_value(
                format!("circular_dep_group_{}", i),
                MetricValue::Map(cycle_map),
            );
        }

        // Calculate technical debt
        println!("[RISK] Calculating technical debt...");
        let debt_scores = self.calculate_technical_debt(graph);
        println!("[RISK] Calculated debt for {} nodes", debt_scores.len());
        for (node_id, score) in debt_scores {
            results.add_value(
                format!("{}_technical_debt", node_id),
                MetricValue::Float(score),
            );
        }

        // Calculate change propagation
        println!("[RISK] Calculating change propagation...");
        let propagation = self.calculate_change_propagation(graph);
        println!("[RISK] Calculated propagation for {} nodes", propagation.len());
        for (node_id, score) in propagation {
            results.add_value(
                format!("{}_change_propagation", node_id),
                MetricValue::Float(score),
            );
        }

        println!("[RISK] Risk analysis complete");
        Ok(results)
    }

    fn name(&self) -> &str {
        "risk"
    }
}