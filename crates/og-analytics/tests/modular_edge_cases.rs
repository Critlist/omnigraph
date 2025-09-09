use og_analytics::{AnalyticsEngineV2, AnalyticsConfigV2};
use og_graph::graph::{CodeGraph, GraphNode, GraphEdge};
use std::time::Duration;

/// Test empty graph handling
#[tokio::test]
async fn test_modular_empty_graph() {
    let graph = CodeGraph::new();
    let config = AnalyticsConfigV2::default();
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should handle empty graph gracefully
    assert_eq!(report.centrality.degree.len(), 0);
    assert_eq!(report.community.num_communities, 0);
    assert_eq!(report.risk.high_risk_count, 0);
    assert_eq!(report.quality.total_code_smells, 0);
}

/// Test single node graph
#[tokio::test]
async fn test_modular_single_node() {
    let mut graph = CodeGraph::new();
    graph.add_node(GraphNode {
        id: "single".to_string(),
        name: "Single Node".to_string(),
        node_type: "file".to_string(),
        file_path: Some("/single.js".to_string()),
        size: 100.0,
        color: "blue".to_string(),
    });
    
    let config = AnalyticsConfigV2::default();
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should handle single node gracefully
    assert_eq!(report.centrality.degree.len(), 1);
    assert!(report.centrality.pagerank.contains_key("single"));
}

/// Test disconnected graph
#[tokio::test]
async fn test_modular_disconnected_graph() {
    let mut graph = CodeGraph::new();
    
    // Create two disconnected components
    for i in 0..2 {
        let component_base = i * 3;
        for j in 0..3 {
            let node_id = format!("node{}", component_base + j);
            graph.add_node(GraphNode {
                id: node_id.clone(),
                name: format!("Node {}", component_base + j),
                node_type: "file".to_string(),
                file_path: Some(format!("/file{}.js", component_base + j)),
                size: 100.0,
                color: "blue".to_string(),
            });
        }
        
        // Connect within component
        graph.add_edge(
            &format!("node{}", component_base),
            &format!("node{}", component_base + 1),
            GraphEdge {
                edge_type: "imports".to_string(),
                weight: 1.0,
            },
        );
        graph.add_edge(
            &format!("node{}", component_base + 1),
            &format!("node{}", component_base + 2),
            GraphEdge {
                edge_type: "imports".to_string(),
                weight: 1.0,
            },
        );
    }
    
    let config = AnalyticsConfigV2::default();
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should handle disconnected components
    assert_eq!(report.centrality.degree.len(), 6);
    assert!(report.community.num_communities >= 2);
}

/// Test graph with self-loops
#[tokio::test]
async fn test_modular_self_loops() {
    let mut graph = CodeGraph::new();
    
    graph.add_node(GraphNode {
        id: "self".to_string(),
        name: "Self Loop".to_string(),
        node_type: "file".to_string(),
        file_path: Some("/self.js".to_string()),
        size: 100.0,
        color: "blue".to_string(),
    });
    
    // Add self-loop
    graph.add_edge("self", "self", GraphEdge {
        edge_type: "imports".to_string(),
        weight: 1.0,
    });
    
    let config = AnalyticsConfigV2::default();
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should handle self-loops without crashing
    assert!(report.centrality.degree.contains_key("self"));
}

/// Test graph with NaN edge weights
#[tokio::test]
async fn test_modular_nan_weights() {
    let mut graph = CodeGraph::new();
    
    for i in 0..3 {
        graph.add_node(GraphNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            node_type: "file".to_string(),
            file_path: Some(format!("/file{}.js", i)),
            size: 100.0,
            color: "blue".to_string(),
        });
    }
    
    // Add edge with NaN weight
    graph.add_edge("node0", "node1", GraphEdge {
        edge_type: "imports".to_string(),
        weight: f64::NAN,
    });
    
    // Add edge with infinity weight
    graph.add_edge("node1", "node2", GraphEdge {
        edge_type: "imports".to_string(),
        weight: f64::INFINITY,
    });
    
    let config = AnalyticsConfigV2::default();
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should handle invalid weights gracefully
    assert_eq!(report.centrality.degree.len(), 3);
    for (_, value) in &report.centrality.pagerank {
        assert!(value.is_finite());
    }
}

/// Test circular dependencies
#[tokio::test]
async fn test_modular_circular_deps() {
    let mut graph = CodeGraph::new();
    
    // Create a cycle: A -> B -> C -> A
    for i in 0..3 {
        graph.add_node(GraphNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            node_type: "file".to_string(),
            file_path: Some(format!("/file{}.js", i)),
            size: 100.0,
            color: "blue".to_string(),
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
    
    let config = AnalyticsConfigV2::default();
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should detect circular dependency
    assert_eq!(report.risk.total_circular_deps, 1);
    assert_eq!(report.risk.circular_dependencies[0].len(), 3);
}

/// Test very large graph with sampling
#[tokio::test]
async fn test_modular_large_graph_sampling() {
    let mut graph = CodeGraph::new();
    
    // Create a large graph
    for i in 0..2000 {
        graph.add_node(GraphNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            node_type: "file".to_string(),
            file_path: Some(format!("/file{}.js", i)),
            size: 100.0,
            color: "blue".to_string(),
        });
    }
    
    // Create some edges
    for i in 0..1999 {
        graph.add_edge(
            &format!("node{}", i),
            &format!("node{}", i + 1),
            GraphEdge {
                edge_type: "imports".to_string(),
                weight: 1.0,
            },
        );
    }
    
    let mut config = AnalyticsConfigV2::default();
    config.use_sampling = true;
    config.betweenness_sample_size = 100; // Sample only 100 nodes
    config.metric_timeout = Duration::from_secs(5); // Shorter timeout for test
    
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should complete within timeout using sampling
    assert_eq!(report.centrality.degree.len(), 2000);
    assert!(report.centrality.betweenness.len() > 0);
}

/// Test timeout handling
#[tokio::test]
async fn test_modular_timeout_handling() {
    let mut graph = CodeGraph::new();
    
    // Create a moderately complex graph
    for i in 0..100 {
        graph.add_node(GraphNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            node_type: "file".to_string(),
            file_path: Some(format!("/file{}.js", i)),
            size: 100.0,
            color: "blue".to_string(),
        });
    }
    
    // Create dense connections
    for i in 0..100 {
        for j in (i+1)..100.min(i+10) {
            graph.add_edge(
                &format!("node{}", i),
                &format!("node{}", j),
                GraphEdge {
                    edge_type: "imports".to_string(),
                    weight: 1.0,
                },
            );
        }
    }
    
    let mut config = AnalyticsConfigV2::default();
    config.metric_timeout = Duration::from_millis(1); // Very short timeout to force failure
    config.parallel_metrics = false; // Sequential to test individual timeouts
    
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should have some errors but not crash
    assert!(!report.errors.is_empty());
    println!("Timeout test errors: {:?}", report.errors);
}

/// Test parallel vs sequential execution
#[tokio::test]
async fn test_modular_parallel_vs_sequential() {
    let mut graph = CodeGraph::new();
    
    // Create a simple graph
    for i in 0..10 {
        graph.add_node(GraphNode {
            id: format!("node{}", i),
            name: format!("Node {}", i),
            node_type: "file".to_string(),
            file_path: Some(format!("/file{}.js", i)),
            size: 100.0,
            color: "blue".to_string(),
        });
    }
    
    for i in 0..9 {
        graph.add_edge(
            &format!("node{}", i),
            &format!("node{}", i + 1),
            GraphEdge {
                edge_type: "imports".to_string(),
                weight: 1.0,
            },
        );
    }
    
    // Test parallel execution
    let mut config_parallel = AnalyticsConfigV2::default();
    config_parallel.parallel_metrics = true;
    let engine_parallel = AnalyticsEngineV2::new(config_parallel);
    let report_parallel = engine_parallel.analyze(&graph).await.unwrap();
    
    // Test sequential execution
    let mut config_sequential = AnalyticsConfigV2::default();
    config_sequential.parallel_metrics = false;
    let engine_sequential = AnalyticsEngineV2::new(config_sequential);
    let report_sequential = engine_sequential.analyze(&graph).await.unwrap();
    
    // Results should be the same
    assert_eq!(report_parallel.centrality.degree.len(), report_sequential.centrality.degree.len());
    assert_eq!(report_parallel.community.num_communities, report_sequential.community.num_communities);
}

/// Test god object detection
#[tokio::test]
async fn test_modular_god_object() {
    let mut graph = CodeGraph::new();
    
    // Create a god object with many connections
    graph.add_node(GraphNode {
        id: "god".to_string(),
        name: "GodObject".to_string(),
        node_type: "class".to_string(),
        file_path: Some("/god.js".to_string()),
        size: 1000.0,
        color: "blue".to_string(),
    });
    
    // Add many dependencies
    for i in 0..50 {
        let node_id = format!("dep{}", i);
        graph.add_node(GraphNode {
            id: node_id.clone(),
            name: format!("Dep{}", i),
            node_type: "file".to_string(),
            file_path: Some(format!("/dep{}.js", i)),
            size: 100.0,
            color: "blue".to_string(),
        });
        
        graph.add_edge("god", &node_id, GraphEdge {
            edge_type: "imports".to_string(),
            weight: 1.0,
        });
    }
    
    let config = AnalyticsConfigV2::default();
    let engine = AnalyticsEngineV2::new(config);
    
    let report = engine.analyze(&graph).await.unwrap();
    
    // Should identify god object
    assert!(report.quality.code_smells.contains_key("god"));
    assert!(report.risk.risk_scores["god"].overall > 0.5);
}