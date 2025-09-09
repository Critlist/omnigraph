use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Graph representation for Neo4j
#[derive(Debug, Clone)]
pub struct CodeGraph {
    pub nodes: Vec<GraphNode>,
    pub relationships: Vec<GraphRelationship>,
    pub metadata: GraphMetadata,
}

/// Node in the graph database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub labels: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Relationship in the graph database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphRelationship {
    pub id: String,
    pub rel_type: String,
    pub source: String,
    pub target: String,
    pub properties: HashMap<String, serde_json::Value>,
}

/// Graph metadata
#[derive(Debug, Clone)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_relationships: usize,
    pub languages: Vec<String>,
    pub generated_at: u64, // Unix timestamp
}

/// Visualization-ready graph data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationGraph {
    pub nodes: Vec<VisualizationNode>,
    pub links: Vec<VisualizationLink>,
}

/// Node for 3D visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationNode {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub color: Option<String>,
    pub size: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

/// Link for 3D visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationLink {
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub link_type: String,
    pub value: Option<f32>,
}