use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Code graph representation using petgraph
pub struct CodeGraph {
    pub graph: DiGraph<GraphNode, GraphEdge>,
    pub node_map: HashMap<String, NodeIndex>,
}

impl CodeGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: GraphNode) -> NodeIndex {
        let id = node.id.clone();
        let idx = self.graph.add_node(node);
        self.node_map.insert(id, idx);
        idx
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, source_id: &str, target_id: &str, edge: GraphEdge) {
        if let (Some(&source), Some(&target)) = (self.node_map.get(source_id), self.node_map.get(target_id)) {
            self.graph.add_edge(source, target, edge);
        }
    }

    /// Get all nodes
    pub fn nodes(&self) -> Vec<&GraphNode> {
        self.graph.node_weights().collect()
    }

    /// Get all edges
    pub fn edges(&self) -> Vec<(&GraphNode, &GraphNode, &GraphEdge)> {
        self.graph
            .edge_indices()
            .filter_map(|edge| {
                let (source, target) = self.graph.edge_endpoints(edge)?;
                let edge_weight = self.graph.edge_weight(edge)?;
                let source_node = self.graph.node_weight(source)?;
                let target_node = self.graph.node_weight(target)?;
                Some((source_node, target_node, edge_weight))
            })
            .collect()
    }

    /// Calculate PageRank for nodes
    pub fn calculate_pagerank(&self, iterations: usize, damping_factor: f64) -> HashMap<String, f64> {
        let node_count = self.graph.node_count();
        if node_count == 0 {
            return HashMap::new();
        }

        let mut ranks = HashMap::new();
        let initial_rank = 1.0 / node_count as f64;
        
        // Initialize ranks
        for idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(idx) {
                ranks.insert(node.id.clone(), initial_rank);
            }
        }

        // Iterate
        for _ in 0..iterations {
            let mut new_ranks = HashMap::new();
            
            for idx in self.graph.node_indices() {
                if let Some(node) = self.graph.node_weight(idx) {
                    let mut rank = (1.0 - damping_factor) / node_count as f64;
                    
                    // Sum contributions from incoming edges
                    for edge in self.graph.edges_directed(idx, petgraph::Direction::Incoming) {
                        let source_idx = edge.source();
                        if let Some(source_node) = self.graph.node_weight(source_idx) {
                            let outgoing_count = self.graph.edges(source_idx).count();
                            if outgoing_count > 0 {
                                // Safely get the rank, default to initial_rank if not found
                                if let Some(source_rank) = ranks.get(&source_node.id) {
                                    rank += damping_factor * source_rank / outgoing_count as f64;
                                } else {
                                    // Log warning but continue
                                    eprintln!("Warning: PageRank - node {} not found in ranks", source_node.id);
                                    rank += damping_factor * initial_rank / outgoing_count as f64;
                                }
                            }
                        }
                    }
                    
                    new_ranks.insert(node.id.clone(), rank);
                }
            }
            
            ranks = new_ranks;
        }

        ranks
    }

    /// Convert to frontend-compatible format
    pub fn to_frontend_format(&self) -> GraphData {
        // Calculate connection counts for each node
        let mut node_connections: HashMap<String, usize> = HashMap::new();
        
        for idx in self.graph.node_indices() {
            if let Some(node) = self.graph.node_weight(idx) {
                let incoming = self.graph.edges_directed(idx, petgraph::Direction::Incoming).count();
                let outgoing = self.graph.edges_directed(idx, petgraph::Direction::Outgoing).count();
                node_connections.insert(node.id.clone(), incoming + outgoing);
            }
        }
        
        // Create nodes with size based on connections
        let nodes: Vec<GraphNode> = self.graph
            .node_weights()
            .map(|node| {
                let connections = node_connections.get(&node.id).unwrap_or(&0);
                let mut node = node.clone();
                // Set size based on number of connections
                node.size = (*connections as f64 * 10.0).max(10.0);
                node
            })
            .collect();

        let links: Vec<GraphLink> = self.graph
            .edge_indices()
            .filter_map(|edge| {
                let (source, target) = self.graph.edge_endpoints(edge)?;
                let source_node = self.graph.node_weight(source)?;
                let target_node = self.graph.node_weight(target)?;
                let edge_weight = self.graph.edge_weight(edge)?;
                
                // Only include edges where both nodes exist
                if self.node_map.contains_key(&source_node.id) && self.node_map.contains_key(&target_node.id) {
                    Some(GraphLink {
                        source: source_node.id.clone(),
                        target: target_node.id.clone(),
                        link_type: edge_weight.edge_type.clone(),
                        value: edge_weight.weight,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Calculate stats before moving the vectors
        let node_count = nodes.len();
        let link_count = links.len();
        let file_count = nodes.iter().filter(|n| n.node_type == "file").count();
        let function_count = nodes.iter().filter(|n| n.node_type == "function").count();
        let class_count = nodes.iter().filter(|n| n.node_type == "class").count();

        GraphData {
            nodes,
            links,
            stats: GraphStats {
                node_count,
                link_count,
                file_count,
                function_count,
                class_count,
            },
        }
    }
}

impl Default for CodeGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Graph node for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub size: f64,
    pub color: String,
    #[serde(rename = "filePath")]
    pub file_path: Option<String>,
}

/// Graph edge
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub edge_type: String,
    pub weight: f64,
}

/// Graph link for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub value: f64,
}

/// Complete graph data for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
    pub stats: GraphStats,
}

/// Graph statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub link_count: usize,
    pub file_count: usize,
    pub function_count: usize,
    pub class_count: usize,
}