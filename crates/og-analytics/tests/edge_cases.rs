use og_analytics::{analyze_graph, AnalyticsConfig};
use og_graph::graph::{CodeGraph, GraphEdge, GraphNode};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

#[tokio::test]
async fn test_empty_graph() {
    let graph = CodeGraph {
        graph: DiGraph::new(),
        node_map: HashMap::new(),
    };
    
    let config = AnalyticsConfig::default();
    let result = analyze_graph(&graph, Some(config)).await;
    
    assert!(result.is_ok(), "Empty graph should not crash");
}

#[tokio::test]
async fn test_single_node_graph() {
    let mut graph = DiGraph::new();
    let node = GraphNode {
        id: "test".to_string(),
        name: "test".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#ff0000".to_string(),
        file_path: Some("/test.js".to_string()),
    };
    
    let idx = graph.add_node(node.clone());
    
    let mut node_map = HashMap::new();
    node_map.insert("test".to_string(), idx);
    
    let code_graph = CodeGraph { graph, node_map };
    
    let config = AnalyticsConfig::default();
    let result = analyze_graph(&code_graph, Some(config)).await;
    
    assert!(result.is_ok(), "Single node graph should not crash");
}

#[tokio::test]
async fn test_graph_with_self_loop() {
    let mut graph = DiGraph::new();
    let node = GraphNode {
        id: "test".to_string(),
        name: "test".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#ff0000".to_string(),
        file_path: Some("/test.js".to_string()),
    };
    
    let idx = graph.add_node(node.clone());
    
    // Add self-loop
    graph.add_edge(idx, idx, GraphEdge {
        edge_type: "imports".to_string(),
        weight: 1.0,
    });
    
    let mut node_map = HashMap::new();
    node_map.insert("test".to_string(), idx);
    
    let code_graph = CodeGraph { graph, node_map };
    
    let config = AnalyticsConfig::default();
    let result = analyze_graph(&code_graph, Some(config)).await;
    
    assert!(result.is_ok(), "Graph with self-loop should not crash");
}

#[tokio::test]
async fn test_disconnected_graph() {
    let mut graph = DiGraph::new();
    
    // Create two disconnected components
    let node1 = GraphNode {
        id: "node1".to_string(),
        name: "node1".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#ff0000".to_string(),
        file_path: Some("/node1.js".to_string()),
    };
    
    let node2 = GraphNode {
        id: "node2".to_string(),
        name: "node2".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#00ff00".to_string(),
        file_path: Some("/node2.js".to_string()),
    };
    
    let node3 = GraphNode {
        id: "node3".to_string(),
        name: "node3".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#0000ff".to_string(),
        file_path: Some("/node3.js".to_string()),
    };
    
    let idx1 = graph.add_node(node1.clone());
    let idx2 = graph.add_node(node2.clone());
    let idx3 = graph.add_node(node3.clone());
    
    // Connect node1 and node2, leave node3 disconnected
    graph.add_edge(idx1, idx2, GraphEdge {
        edge_type: "imports".to_string(),
        weight: 1.0,
    });
    
    let mut node_map = HashMap::new();
    node_map.insert("node1".to_string(), idx1);
    node_map.insert("node2".to_string(), idx2);
    node_map.insert("node3".to_string(), idx3);
    
    let code_graph = CodeGraph { graph, node_map };
    
    let config = AnalyticsConfig::default();
    let result = analyze_graph(&code_graph, Some(config)).await;
    
    assert!(result.is_ok(), "Disconnected graph should not crash");
}

#[tokio::test]
async fn test_graph_with_nan_weights() {
    let mut graph = DiGraph::new();
    
    let node1 = GraphNode {
        id: "node1".to_string(),
        name: "node1".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#ff0000".to_string(),
        file_path: Some("/node1.js".to_string()),
    };
    
    let node2 = GraphNode {
        id: "node2".to_string(),
        name: "node2".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#00ff00".to_string(),
        file_path: Some("/node2.js".to_string()),
    };
    
    let idx1 = graph.add_node(node1.clone());
    let idx2 = graph.add_node(node2.clone());
    
    // Add edge with NaN weight
    graph.add_edge(idx1, idx2, GraphEdge {
        edge_type: "imports".to_string(),
        weight: f64::NAN,
    });
    
    let mut node_map = HashMap::new();
    node_map.insert("node1".to_string(), idx1);
    node_map.insert("node2".to_string(), idx2);
    
    let code_graph = CodeGraph { graph, node_map };
    
    let config = AnalyticsConfig::default();
    let result = analyze_graph(&code_graph, Some(config)).await;
    
    assert!(result.is_ok(), "Graph with NaN weights should not crash");
}

#[tokio::test]
async fn test_graph_with_infinite_weights() {
    let mut graph = DiGraph::new();
    
    let node1 = GraphNode {
        id: "node1".to_string(),
        name: "node1".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#ff0000".to_string(),
        file_path: Some("/node1.js".to_string()),
    };
    
    let node2 = GraphNode {
        id: "node2".to_string(),
        name: "node2".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#00ff00".to_string(),
        file_path: Some("/node2.js".to_string()),
    };
    
    let idx1 = graph.add_node(node1.clone());
    let idx2 = graph.add_node(node2.clone());
    
    // Add edge with infinite weight
    graph.add_edge(idx1, idx2, GraphEdge {
        edge_type: "imports".to_string(),
        weight: f64::INFINITY,
    });
    
    let mut node_map = HashMap::new();
    node_map.insert("node1".to_string(), idx1);
    node_map.insert("node2".to_string(), idx2);
    
    let code_graph = CodeGraph { graph, node_map };
    
    let config = AnalyticsConfig::default();
    let result = analyze_graph(&code_graph, Some(config)).await;
    
    assert!(result.is_ok(), "Graph with infinite weights should not crash");
}

#[tokio::test]
async fn test_circular_dependency() {
    let mut graph = DiGraph::new();
    
    let node1 = GraphNode {
        id: "node1".to_string(),
        name: "node1".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#ff0000".to_string(),
        file_path: Some("/node1.js".to_string()),
    };
    
    let node2 = GraphNode {
        id: "node2".to_string(),
        name: "node2".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#00ff00".to_string(),
        file_path: Some("/node2.js".to_string()),
    };
    
    let node3 = GraphNode {
        id: "node3".to_string(),
        name: "node3".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#0000ff".to_string(),
        file_path: Some("/node3.js".to_string()),
    };
    
    let idx1 = graph.add_node(node1.clone());
    let idx2 = graph.add_node(node2.clone());
    let idx3 = graph.add_node(node3.clone());
    
    // Create circular dependency
    graph.add_edge(idx1, idx2, GraphEdge {
        edge_type: "imports".to_string(),
        weight: 1.0,
    });
    graph.add_edge(idx2, idx3, GraphEdge {
        edge_type: "imports".to_string(),
        weight: 1.0,
    });
    graph.add_edge(idx3, idx1, GraphEdge {
        edge_type: "imports".to_string(),
        weight: 1.0,
    });
    
    let mut node_map = HashMap::new();
    node_map.insert("node1".to_string(), idx1);
    node_map.insert("node2".to_string(), idx2);
    node_map.insert("node3".to_string(), idx3);
    
    let code_graph = CodeGraph { graph, node_map };
    
    let config = AnalyticsConfig::default();
    let result = analyze_graph(&code_graph, Some(config)).await;
    
    assert!(result.is_ok(), "Graph with circular dependency should not crash");
}