use anyhow::Result;
use og_graph::graph::CodeGraph;
use petgraph::algo::tarjan_scc;
use petgraph::Direction;
// Removed unused EdgeRef import
use std::collections::HashMap;
use tracing::{debug, warn};

/// Risk analysis with robust error handling
pub struct RiskAnalyzer {
    pub complexity_threshold: f64,
    pub high_coupling_threshold: usize,
    pub bottleneck_threshold: usize,
}

impl Default for RiskAnalyzer {
    fn default() -> Self {
        Self {
            complexity_threshold: 15.0,
            high_coupling_threshold: 10,
            bottleneck_threshold: 5,
        }
    }
}

impl RiskAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze all risk factors with error recovery
    pub fn analyze_risks(&self, graph: &CodeGraph) -> Result<RiskResults> {
        let mut results = RiskResults::default();
        
        // Validate input
        if graph.graph.node_count() == 0 {
            debug!("Empty graph, returning default risk results");
            return Ok(results);
        }

        // Calculate each risk metric with individual error handling
        match self.identify_high_risk_nodes(graph) {
            Ok(risk_scores) => results.risk_scores = risk_scores,
            Err(e) => {
                warn!("Risk score calculation failed: {}", e);
                results.errors.push(format!("Risk scores: {}", e));
            }
        }

        match self.find_chokepoints(graph) {
            Ok(chokepoints) => results.chokepoints = chokepoints,
            Err(e) => {
                warn!("Chokepoint detection failed: {}", e);
                results.errors.push(format!("Chokepoints: {}", e));
            }
        }

        match self.detect_circular_dependencies(graph) {
            Ok(cycles) => results.circular_dependencies = cycles,
            Err(e) => {
                warn!("Circular dependency detection failed: {}", e);
                results.errors.push(format!("Circular deps: {}", e));
            }
        }

        match self.calculate_coupling_metrics(graph) {
            Ok(coupling) => results.coupling_metrics = coupling,
            Err(e) => {
                warn!("Coupling metrics failed: {}", e);
                results.errors.push(format!("Coupling: {}", e));
            }
        }

        // Calculate summary statistics
        results.high_risk_count = results.risk_scores.values()
            .filter(|score| score.overall > 0.7)
            .count();
        
        results.total_circular_deps = results.circular_dependencies.len();
        
        results.avg_risk_score = if !results.risk_scores.is_empty() {
            results.risk_scores.values().map(|s| s.overall).sum::<f64>() 
                / results.risk_scores.len() as f64
        } else {
            0.0
        };

        Ok(results)
    }

    /// Identify high-risk nodes with validation
    fn identify_high_risk_nodes(&self, graph: &CodeGraph) -> Result<HashMap<String, RiskScore>> {
        let mut risk_scores = HashMap::new();
        let node_count = graph.graph.node_count().max(1) as f64;

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
                let normalized_degree = (total_degree as f64 / node_count).clamp(0.0, 1.0);

                // Estimate complexity based on node type and connections
                let complexity_estimate = match node.node_type.as_str() {
                    "function" | "method" => ((out_degree + 1) as f64).min(50.0),
                    "class" => ((out_degree * 2) as f64).min(100.0),
                    "file" | "module" => ((out_degree as f64).sqrt() * 5.0).min(50.0),
                    _ => 1.0,
                };

                // Calculate risk factors with bounds
                let complexity_risk = (complexity_estimate / self.complexity_threshold)
                    .clamp(0.0, 1.0);
                let centrality_risk = normalized_degree.clamp(0.0, 1.0);
                
                // Check if node is a bottleneck
                let is_bottleneck = in_degree > self.bottleneck_threshold 
                    && out_degree > self.bottleneck_threshold;
                let bottleneck_risk = if is_bottleneck { 0.5 } else { 0.0 };

                // Check coupling risk
                let coupling_risk = if total_degree > self.high_coupling_threshold {
                    ((total_degree - self.high_coupling_threshold) as f64 / 10.0)
                        .clamp(0.0, 1.0)
                } else {
                    0.0
                };

                // Combined risk score with weights
                let overall_risk = (
                    complexity_risk * 0.3 + 
                    centrality_risk * 0.3 + 
                    bottleneck_risk * 0.2 +
                    coupling_risk * 0.2
                ).clamp(0.0, 1.0);

                risk_scores.insert(
                    node.id.clone(),
                    RiskScore {
                        overall: overall_risk,
                        complexity: complexity_risk,
                        centrality: centrality_risk,
                        bottleneck: bottleneck_risk,
                        coupling: coupling_risk,
                    },
                );
            }
        }

        Ok(risk_scores)
    }

    /// Find architectural chokepoints with safety checks
    fn find_chokepoints(&self, graph: &CodeGraph) -> Result<HashMap<String, f64>> {
        let mut chokepoints = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                let in_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count();
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();

                // A chokepoint has high in and out degree
                if in_degree > self.bottleneck_threshold || out_degree > self.bottleneck_threshold {
                    // Calculate chokepoint score
                    let in_score = (in_degree as f64 / (self.bottleneck_threshold * 2) as f64)
                        .clamp(0.0, 1.0);
                    let out_score = (out_degree as f64 / (self.bottleneck_threshold * 2) as f64)
                        .clamp(0.0, 1.0);
                    
                    // Chokepoint score is product of in and out scores
                    let chokepoint_score = (in_score * out_score).sqrt().clamp(0.0, 1.0);
                    
                    if chokepoint_score > 0.1 {
                        chokepoints.insert(node.id.clone(), chokepoint_score);
                    }
                }
            }
        }

        Ok(chokepoints)
    }

    /// Detect circular dependencies using Tarjan's algorithm
    fn detect_circular_dependencies(&self, graph: &CodeGraph) -> Result<Vec<Vec<String>>> {
        debug!("Detecting circular dependencies");
        
        // Use Tarjan's strongly connected components algorithm
        let sccs = tarjan_scc(&graph.graph);
        
        let mut cycles = Vec::new();
        
        for scc in sccs {
            // Only consider SCCs with more than one node (actual cycles)
            if scc.len() > 1 {
                let mut cycle_nodes = Vec::new();
                
                for &node_idx in &scc {
                    if let Some(node) = graph.graph.node_weight(node_idx) {
                        cycle_nodes.push(node.id.clone());
                    }
                }
                
                if !cycle_nodes.is_empty() {
                    // Sort for consistent output
                    cycle_nodes.sort();
                    cycles.push(cycle_nodes);
                }
            }
        }
        
        // Limit number of cycles reported to avoid overwhelming output
        if cycles.len() > 100 {
            warn!("Found {} cycles, truncating to 100", cycles.len());
            cycles.truncate(100);
        }
        
        Ok(cycles)
    }

    /// Calculate coupling metrics
    fn calculate_coupling_metrics(&self, graph: &CodeGraph) -> Result<HashMap<String, CouplingMetrics>> {
        let mut coupling_metrics = HashMap::new();
        
        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Calculate afferent coupling (incoming dependencies)
                let afferent = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count();
                
                // Calculate efferent coupling (outgoing dependencies)
                let efferent = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();
                
                // Calculate instability (efferent / (afferent + efferent))
                let instability = if afferent + efferent > 0 {
                    (efferent as f64 / (afferent + efferent) as f64).clamp(0.0, 1.0)
                } else {
                    0.0
                };
                
                // Coupling score based on total connections
                let coupling_score = ((afferent + efferent) as f64 / 20.0).clamp(0.0, 1.0);
                
                coupling_metrics.insert(
                    node.id.clone(),
                    CouplingMetrics {
                        afferent_coupling: afferent,
                        efferent_coupling: efferent,
                        instability,
                        coupling_score,
                    },
                );
            }
        }
        
        Ok(coupling_metrics)
    }
}

#[derive(Debug, Default, Clone)]
pub struct RiskResults {
    pub risk_scores: HashMap<String, RiskScore>,
    pub chokepoints: HashMap<String, f64>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub coupling_metrics: HashMap<String, CouplingMetrics>,
    pub high_risk_count: usize,
    pub total_circular_deps: usize,
    pub avg_risk_score: f64,
    pub errors: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct RiskScore {
    pub overall: f64,
    pub complexity: f64,
    pub centrality: f64,
    pub bottleneck: f64,
    pub coupling: f64,
}

#[derive(Debug, Default, Clone)]
pub struct CouplingMetrics {
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability: f64,
    pub coupling_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_graph::graph::{GraphNode, GraphEdge};

    #[test]
    fn test_empty_graph() {
        let graph = CodeGraph::new();
        let analyzer = RiskAnalyzer::new();
        let results = analyzer.analyze_risks(&graph).unwrap();
        assert_eq!(results.high_risk_count, 0);
        assert_eq!(results.total_circular_deps, 0);
    }

    #[test]
    fn test_circular_dependency() {
        let mut graph = CodeGraph::new();
        
        // Create a cycle: A -> B -> C -> A
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
        
        graph.add_edge("node0", "node1", GraphEdge {
            edge_type: "imports".to_string(),
            weight: 1.0,
        });
        graph.add_edge("node1", "node2", GraphEdge {
            edge_type: "imports".to_string(),
            weight: 1.0,
        });
        graph.add_edge("node2", "node0", GraphEdge {
            edge_type: "imports".to_string(),
            weight: 1.0,
        });
        
        let analyzer = RiskAnalyzer::new();
        let results = analyzer.analyze_risks(&graph).unwrap();
        
        // Should detect the circular dependency
        assert_eq!(results.total_circular_deps, 1);
        assert_eq!(results.circular_dependencies[0].len(), 3);
    }
}