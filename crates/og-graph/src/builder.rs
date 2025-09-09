use crate::graph::{CodeGraph, GraphEdge, GraphNode};
use og_types::{AstNode, NodeType, ParsedFile, Relationship, RelationshipType};
use std::collections::HashMap;
use tracing::{debug, info};

/// Graph builder that converts parsed files into a code graph
pub struct GraphBuilder {
    graph: CodeGraph,
    color_map: HashMap<String, String>,
}

impl GraphBuilder {
    pub fn new() -> Self {
        let mut color_map = HashMap::new();
        // Define colors for different node types
        color_map.insert("file".to_string(), "#4a9eff".to_string());
        color_map.insert("module".to_string(), "#4a9eff".to_string());
        color_map.insert("class".to_string(), "#ff9800".to_string());
        color_map.insert("interface".to_string(), "#9c27b0".to_string());
        color_map.insert("function".to_string(), "#4caf50".to_string());
        color_map.insert("method".to_string(), "#8bc34a".to_string());
        color_map.insert("variable".to_string(), "#607d8b".to_string());
        color_map.insert("property".to_string(), "#795548".to_string());
        color_map.insert("type".to_string(), "#e91e63".to_string());
        color_map.insert("enum".to_string(), "#ff5722".to_string());

        Self {
            graph: CodeGraph::new(),
            color_map,
        }
    }

    /// Build graph from parsed files
    pub fn build_from_files(mut self, files: Vec<ParsedFile>) -> CodeGraph {
        info!("Building graph from {} parsed files", files.len());

        // First pass: Add all nodes
        for file in &files {
            for node in &file.nodes {
                self.add_ast_node(node, &file.path.to_string_lossy());
            }
        }

        // Second pass: Add all relationships
        for file in &files {
            for relationship in &file.relationships {
                self.add_relationship(relationship);
            }
        }

        info!(
            "Graph built with {} nodes and {} edges",
            self.graph.node_map.len(),
            self.graph.graph.edge_count()
        );

        self.graph
    }

    /// Add an AST node to the graph
    fn add_ast_node(&mut self, node: &AstNode, file_path: &str) {
        let node_type_str = node.node_type.as_str();
        let size = self.calculate_node_size(node);
        let color = self
            .color_map
            .get(node_type_str)
            .cloned()
            .unwrap_or_else(|| "#cccccc".to_string());

        let graph_node = GraphNode {
            id: node.id.clone(),
            name: node.name.clone(),
            node_type: node_type_str.to_string(),
            size,
            color,
            file_path: Some(file_path.to_string()),
        };

        self.graph.add_node(graph_node);
        debug!("Added node: {} ({})", node.name, node_type_str);
    }

    /// Add a relationship to the graph
    fn add_relationship(&mut self, relationship: &Relationship) {
        // Check if both nodes exist before adding edge
        if self.graph.node_map.contains_key(&relationship.source)
            && self.graph.node_map.contains_key(&relationship.target)
        {
            let edge = GraphEdge {
                edge_type: relationship.relationship_type.as_str().to_string(),
                weight: self.calculate_edge_weight(&relationship.relationship_type),
            };

            self.graph
                .add_edge(&relationship.source, &relationship.target, edge);

            debug!(
                "Added edge: {} -> {} ({})",
                relationship.source,
                relationship.target,
                relationship.relationship_type.as_str()
            );
        } else {
            debug!(
                "Skipping edge: {} -> {} (one or both nodes don't exist)",
                relationship.source, relationship.target
            );
        }
    }

    /// Calculate node size based on its type and metrics
    fn calculate_node_size(&self, node: &AstNode) -> f64 {
        let base_size = match node.node_type {
            NodeType::File | NodeType::Module => 20.0,
            NodeType::Class | NodeType::Interface => 15.0,
            NodeType::Function | NodeType::Method => 10.0,
            _ => 5.0,
        };

        // Scale by lines of code
        let line_count = (node.end_line - node.start_line + 1) as f64;
        base_size + (line_count / 10.0).min(10.0)
    }

    /// Calculate edge weight based on relationship type
    fn calculate_edge_weight(&self, rel_type: &RelationshipType) -> f64 {
        match rel_type {
            RelationshipType::Contains => 1.0,
            RelationshipType::Imports => 2.0,
            RelationshipType::Exports => 2.0,
            RelationshipType::Extends => 3.0,
            RelationshipType::Implements => 3.0,
            RelationshipType::Calls => 1.5,
            RelationshipType::References => 1.0,
        }
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}
