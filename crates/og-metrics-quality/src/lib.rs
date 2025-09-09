use anyhow::Result;
use og_graph::graph::CodeGraph;
use petgraph::Direction;
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use tracing::{debug, warn};

/// Quality metrics analyzer with robust error handling
pub struct QualityAnalyzer {
    pub complexity_threshold: f64,
    pub cohesion_threshold: f64,
    pub size_threshold: usize,
}

impl Default for QualityAnalyzer {
    fn default() -> Self {
        Self {
            complexity_threshold: 10.0,
            cohesion_threshold: 0.5,
            size_threshold: 500,
        }
    }
}

impl QualityAnalyzer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Analyze all quality metrics with error recovery
    pub fn analyze_quality(&self, graph: &CodeGraph) -> Result<QualityResults> {
        let mut results = QualityResults::default();
        
        // Validate input
        if graph.graph.node_count() == 0 {
            debug!("Empty graph, returning default quality results");
            return Ok(results);
        }

        // Calculate each quality metric with individual error handling
        match self.calculate_complexity_metrics(graph) {
            Ok(complexity) => results.complexity_metrics = complexity,
            Err(e) => {
                warn!("Complexity metrics failed: {}", e);
                results.errors.push(format!("Complexity: {}", e));
            }
        }

        match self.calculate_cohesion_metrics(graph) {
            Ok(cohesion) => results.cohesion_metrics = cohesion,
            Err(e) => {
                warn!("Cohesion metrics failed: {}", e);
                results.errors.push(format!("Cohesion: {}", e));
            }
        }

        match self.analyze_code_smells(graph) {
            Ok(smells) => results.code_smells = smells,
            Err(e) => {
                warn!("Code smell detection failed: {}", e);
                results.errors.push(format!("Code smells: {}", e));
            }
        }

        match self.calculate_maintainability_index(graph) {
            Ok(maintainability) => results.maintainability = maintainability,
            Err(e) => {
                warn!("Maintainability index failed: {}", e);
                results.errors.push(format!("Maintainability: {}", e));
            }
        }

        // Calculate summary statistics
        results.avg_complexity = if !results.complexity_metrics.is_empty() {
            results.complexity_metrics.values()
                .map(|m| m.cyclomatic_complexity)
                .sum::<f64>() / results.complexity_metrics.len() as f64
        } else {
            0.0
        };

        results.avg_cohesion = if !results.cohesion_metrics.is_empty() {
            results.cohesion_metrics.values()
                .map(|m| m.cohesion_score)
                .sum::<f64>() / results.cohesion_metrics.len() as f64
        } else {
            0.0
        };

        results.total_code_smells = results.code_smells.values()
            .map(|s| s.smells.len())
            .sum();

        Ok(results)
    }

    /// Calculate complexity metrics with validation
    fn calculate_complexity_metrics(&self, graph: &CodeGraph) -> Result<HashMap<String, ComplexityMetrics>> {
        let mut complexity_map = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Calculate cyclomatic complexity (simplified)
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();
                
                let in_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count();

                // Estimate cyclomatic complexity based on node type and connections
                let cyclomatic = match node.node_type.as_str() {
                    "function" | "method" => {
                        // Functions: complexity increases with branches
                        (1.0 + out_degree as f64).min(50.0)
                    }
                    "class" => {
                        // Classes: complexity based on methods and dependencies
                        ((out_degree + in_degree) as f64 / 2.0).min(100.0)
                    }
                    "file" | "module" => {
                        // Files: complexity based on imports and exports
                        ((out_degree + in_degree) as f64 / 3.0).min(50.0)
                    }
                    _ => 1.0,
                };

                // Calculate cognitive complexity (simplified)
                let cognitive = cyclomatic * 1.2; // Slightly higher than cyclomatic

                // Lines of code estimate based on size
                let loc = (node.size as f64).min(10000.0);

                // Depth of inheritance (simplified - based on incoming edges)
                let depth = (in_degree as f64).sqrt().min(10.0);

                complexity_map.insert(
                    node.id.clone(),
                    ComplexityMetrics {
                        cyclomatic_complexity: cyclomatic,
                        cognitive_complexity: cognitive,
                        lines_of_code: loc,
                        depth_of_inheritance: depth,
                    },
                );
            }
        }

        Ok(complexity_map)
    }

    /// Calculate cohesion metrics
    fn calculate_cohesion_metrics(&self, graph: &CodeGraph) -> Result<HashMap<String, CohesionMetrics>> {
        let mut cohesion_map = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Get all connected nodes
                let mut connected_nodes = HashSet::new();
                
                for edge in graph.graph.edges(node_idx) {
                    connected_nodes.insert(edge.target());
                }
                
                for edge in graph.graph.edges_directed(node_idx, Direction::Incoming) {
                    connected_nodes.insert(edge.source());
                }

                // Calculate internal vs external connections
                let mut internal_connections = 0;
                let mut external_connections = 0;

                for &connected in &connected_nodes {
                    if let Some(connected_node) = graph.graph.node_weight(connected) {
                        // Check if nodes are in same module/package
                        if Self::same_module(&node.file_path, &connected_node.file_path) {
                            internal_connections += 1;
                        } else {
                            external_connections += 1;
                        }
                    }
                }

                // Calculate cohesion score
                let total_connections = internal_connections + external_connections;
                let cohesion_score = if total_connections > 0 {
                    (internal_connections as f64 / total_connections as f64).clamp(0.0, 1.0)
                } else {
                    1.0 // No connections means perfect cohesion
                };

                // LCOM (Lack of Cohesion of Methods) - simplified
                let lcom = if node.node_type == "class" {
                    ((external_connections as f64) / (total_connections.max(1) as f64))
                        .clamp(0.0, 1.0)
                } else {
                    0.0
                };

                cohesion_map.insert(
                    node.id.clone(),
                    CohesionMetrics {
                        cohesion_score,
                        lcom,
                        internal_connections,
                        external_connections,
                    },
                );
            }
        }

        Ok(cohesion_map)
    }

    /// Detect code smells
    fn analyze_code_smells(&self, graph: &CodeGraph) -> Result<HashMap<String, CodeSmells>> {
        let mut smells_map = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                let mut smells = Vec::new();

                // God class/module detection
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count();
                let in_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count();

                if out_degree + in_degree > 30 {
                    smells.push(CodeSmell {
                        smell_type: "God Object".to_string(),
                        severity: if out_degree + in_degree > 50 { 
                            "High".to_string() 
                        } else { 
                            "Medium".to_string() 
                        },
                        description: format!("High coupling: {} connections", out_degree + in_degree),
                    });
                }

                // Long method/file detection
                if node.size as usize > self.size_threshold {
                    smells.push(CodeSmell {
                        smell_type: "Large File".to_string(),
                        severity: if node.size as usize > self.size_threshold * 2 { 
                            "High".to_string() 
                        } else { 
                            "Medium".to_string() 
                        },
                        description: format!("File size: {} lines", node.size as usize),
                    });
                }

                // Feature envy detection (high external dependencies)
                if out_degree > 15 {
                    smells.push(CodeSmell {
                        smell_type: "Feature Envy".to_string(),
                        severity: "Medium".to_string(),
                        description: format!("High external dependencies: {}", out_degree),
                    });
                }

                // Shotgun surgery detection (many dependents)
                if in_degree > 20 {
                    smells.push(CodeSmell {
                        smell_type: "Shotgun Surgery".to_string(),
                        severity: "High".to_string(),
                        description: format!("Many dependents: {} files depend on this", in_degree),
                    });
                }

                if !smells.is_empty() {
                    smells_map.insert(
                        node.id.clone(),
                        CodeSmells { smells },
                    );
                }
            }
        }

        Ok(smells_map)
    }

    /// Calculate maintainability index
    fn calculate_maintainability_index(&self, graph: &CodeGraph) -> Result<HashMap<String, f64>> {
        let mut maintainability_map = HashMap::new();

        for node_idx in graph.graph.node_indices() {
            if let Some(node) = graph.graph.node_weight(node_idx) {
                // Simplified maintainability index calculation
                let out_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Outgoing)
                    .count() as f64;
                let in_degree = graph
                    .graph
                    .edges_directed(node_idx, Direction::Incoming)
                    .count() as f64;

                // Factors affecting maintainability
                let complexity_factor = (1.0 / (1.0 + out_degree / 10.0)).clamp(0.0, 1.0);
                let size_factor = (1.0 / (1.0 + node.size as f64 / 500.0)).clamp(0.0, 1.0);
                let coupling_factor = (1.0 / (1.0 + in_degree / 10.0)).clamp(0.0, 1.0);

                // Calculate maintainability index (0-100 scale)
                let maintainability = (
                    complexity_factor * 0.4 +
                    size_factor * 0.3 +
                    coupling_factor * 0.3
                ) * 100.0;

                maintainability_map.insert(
                    node.id.clone(),
                    maintainability.clamp(0.0, 100.0),
                );
            }
        }

        Ok(maintainability_map)
    }

    /// Check if two file paths are in the same module
    fn same_module(path1: &Option<String>, path2: &Option<String>) -> bool {
        match (path1, path2) {
            (Some(p1), Some(p2)) => {
                // Simple heuristic: same directory
                let dir1 = p1.rsplit_once('/').map(|x| x.0);
                let dir2 = p2.rsplit_once('/').map(|x| x.0);
                dir1 == dir2
            }
            _ => false,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct QualityResults {
    pub complexity_metrics: HashMap<String, ComplexityMetrics>,
    pub cohesion_metrics: HashMap<String, CohesionMetrics>,
    pub code_smells: HashMap<String, CodeSmells>,
    pub maintainability: HashMap<String, f64>,
    pub avg_complexity: f64,
    pub avg_cohesion: f64,
    pub total_code_smells: usize,
    pub errors: Vec<String>,
}

#[derive(Debug, Default, Clone)]
pub struct ComplexityMetrics {
    pub cyclomatic_complexity: f64,
    pub cognitive_complexity: f64,
    pub lines_of_code: f64,
    pub depth_of_inheritance: f64,
}

#[derive(Debug, Default, Clone)]
pub struct CohesionMetrics {
    pub cohesion_score: f64,
    pub lcom: f64, // Lack of Cohesion of Methods
    pub internal_connections: usize,
    pub external_connections: usize,
}

#[derive(Debug, Clone)]
pub struct CodeSmells {
    pub smells: Vec<CodeSmell>,
}

#[derive(Debug, Clone)]
pub struct CodeSmell {
    pub smell_type: String,
    pub severity: String,
    pub description: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_graph::graph::{GraphNode, GraphEdge};

    #[test]
    fn test_empty_graph() {
        let graph = CodeGraph::new();
        let analyzer = QualityAnalyzer::new();
        let results = analyzer.analyze_quality(&graph).unwrap();
        assert_eq!(results.total_code_smells, 0);
        assert_eq!(results.avg_complexity, 0.0);
    }

    #[test]
    fn test_god_object_detection() {
        let mut graph = CodeGraph::new();
        
        // Create a god object with many connections
        graph.add_node(GraphNode {
            id: "god".to_string(),
            name: "GodObject".to_string(),
            node_type: "class".to_string(),
            file_path: Some("/god.js".to_string()),
            size: 1000,
            color: None,
        });
        
        // Add many dependencies
        for i in 0..35 {
            let node_id = format!("dep{}", i);
            graph.add_node(GraphNode {
                id: node_id.clone(),
                name: format!("Dep{}", i),
                node_type: "file".to_string(),
                file_path: Some(format!("/dep{}.js", i)),
                size: 100,
                color: None,
            });
            
            graph.add_edge("god", &node_id, GraphEdge {
                edge_type: "imports".to_string(),
                weight: 1.0,
            });
        }
        
        let analyzer = QualityAnalyzer::new();
        let results = analyzer.analyze_quality(&graph).unwrap();
        
        // Should detect god object code smell
        assert!(results.code_smells.contains_key("god"));
        let god_smells = &results.code_smells["god"];
        assert!(god_smells.smells.iter().any(|s| s.smell_type == "God Object"));
    }
}