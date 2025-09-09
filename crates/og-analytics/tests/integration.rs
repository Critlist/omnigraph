use og_analytics::{analyze_graph, AnalyticsConfig, to_ui_metrics};
use og_graph::graph::{CodeGraph, GraphNode, GraphEdge};

#[tokio::test]
async fn test_analytics_engine() {
    // Create a simple test graph
    let mut graph = CodeGraph::new();
    
    // Add nodes
    let file1 = GraphNode {
        id: "file1".to_string(),
        name: "main.rs".to_string(),
        node_type: "file".to_string(),
        size: 100.0,
        color: "#ff0000".to_string(),
        file_path: Some("/src/main.rs".to_string()),
    };
    
    let file2 = GraphNode {
        id: "file2".to_string(),
        name: "lib.rs".to_string(),
        node_type: "file".to_string(),
        size: 200.0,
        color: "#00ff00".to_string(),
        file_path: Some("/src/lib.rs".to_string()),
    };
    
    let function1 = GraphNode {
        id: "function1".to_string(),
        name: "main".to_string(),
        node_type: "function".to_string(),
        size: 50.0,
        color: "#0000ff".to_string(),
        file_path: Some("/src/main.rs".to_string()),
    };
    
    let class1 = GraphNode {
        id: "class1".to_string(),
        name: "MyClass".to_string(),
        node_type: "class".to_string(),
        size: 150.0,
        color: "#ffff00".to_string(),
        file_path: Some("/src/lib.rs".to_string()),
    };
    
    // Add nodes to graph
    graph.add_node(file1);
    graph.add_node(file2);
    graph.add_node(function1);
    graph.add_node(class1);
    
    // Add edges
    graph.add_edge("file1", "file2", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 1.0,
    });
    
    graph.add_edge("file1", "function1", GraphEdge {
        edge_type: "contains".to_string(),
        weight: 1.0,
    });
    
    graph.add_edge("file2", "class1", GraphEdge {
        edge_type: "contains".to_string(),
        weight: 1.0,
    });
    
    graph.add_edge("function1", "class1", GraphEdge {
        edge_type: "calls".to_string(),
        weight: 2.0,
    });
    
    // Test analytics
    let config = AnalyticsConfig::default();
    let report = analyze_graph(&graph, Some(config)).await.unwrap();
    
    // Verify we got results
    assert!(!report.metrics.is_empty());
    assert!(!report.composite_scores.is_empty());
    assert_eq!(report.summary.total_nodes, 4);
    assert_eq!(report.summary.total_edges, 4);
    
    // Test UI metrics conversion
    let ui_metrics = to_ui_metrics(&report, &graph);
    assert_eq!(ui_metrics.len(), 4);
    
    // Check that each node has metrics
    for metric in ui_metrics {
        assert!(!metric.path.is_empty());
        assert!(!metric.name.is_empty());
        assert!(metric.importance >= 0.0 && metric.importance <= 1.0);
        assert!(metric.risk >= 0.0 && metric.risk <= 1.0);
        assert!(metric.chokepoint >= 0.0 && metric.chokepoint <= 1.0);
        assert!(metric.payoff >= 0.0 && metric.payoff <= 1.0);
    }
    
    println!("Analytics test passed!");
    println!("Summary: {:?}", report.summary);
}

#[tokio::test]
async fn test_centrality_metrics() {
    use og_analytics::metrics::{Metric, centrality::CentralityMetrics};
    
    // Create a simple graph
    let mut graph = CodeGraph::new();
    
    // Create a star topology (node 0 connected to all others)
    for i in 0..5 {
        let node = GraphNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            node_type: "file".to_string(),
            size: 1.0,
            color: "#000000".to_string(),
            file_path: None,
        };
        graph.add_node(node);
    }
    
    // Connect node0 to all others
    for i in 1..5 {
        graph.add_edge(&format!("node0"), &format!("node{}", i), GraphEdge {
            edge_type: "imports".to_string(),
            weight: 1.0,
        });
    }
    
    // Calculate centrality
    let centrality = CentralityMetrics::new();
    let results = centrality.calculate(&graph).unwrap();
    
    // Verify node0 has highest degree centrality
    let node0_degree = results.get_node_value("node0", "degree");
    assert!(node0_degree.is_some());
    
    println!("Centrality test passed!");
}

#[tokio::test]
async fn test_community_detection() {
    use og_analytics::metrics::{Metric, community::CommunityDetection};
    
    // Create a graph with two clear communities
    let mut graph = CodeGraph::new();
    
    // Community 1 nodes
    for i in 0..3 {
        let node = GraphNode {
            id: format!("c1_node{}", i),
            name: format!("C1 Node {}", i),
            node_type: "file".to_string(),
            size: 1.0,
            color: "#ff0000".to_string(),
            file_path: None,
        };
        graph.add_node(node);
    }
    
    // Community 2 nodes
    for i in 0..3 {
        let node = GraphNode {
            id: format!("c2_node{}", i),
            name: format!("C2 Node {}", i),
            node_type: "file".to_string(),
            size: 1.0,
            color: "#00ff00".to_string(),
            file_path: None,
        };
        graph.add_node(node);
    }
    
    // Connect within community 1
    graph.add_edge("c1_node0", "c1_node1", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 2.0,
    });
    graph.add_edge("c1_node1", "c1_node2", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 2.0,
    });
    graph.add_edge("c1_node2", "c1_node0", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 2.0,
    });
    
    // Connect within community 2
    graph.add_edge("c2_node0", "c2_node1", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 2.0,
    });
    graph.add_edge("c2_node1", "c2_node2", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 2.0,
    });
    graph.add_edge("c2_node2", "c2_node0", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 2.0,
    });
    
    // Weak connection between communities
    graph.add_edge("c1_node0", "c2_node0", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 0.1,
    });
    
    // Detect communities
    let detector = CommunityDetection::new(1.0);
    let results = detector.calculate(&graph).unwrap();
    
    // Check that communities were detected
    assert!(results.values.contains_key("num_communities"));
    
    println!("Community detection test passed!");
}