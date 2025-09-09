mod engine_v2;

use engine_v2::{Engine, AnalyzedGraph, AnalysisSummary};
use og_graph::graph::GraphData;
use og_utils::ProgressReporter;
use serde::{Deserialize, Serialize};
use std::path::{PathBuf, Path};
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use tauri::Emitter;

#[derive(Debug, Serialize, Deserialize)]
pub struct ParseResult {
    file_count: usize,
    node_count: usize,
    edge_count: usize,
    languages: Vec<String>,
    files: Vec<FileNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileNode {
    name: String,
    path: String,
    #[serde(rename = "type")]
    node_type: String,
    children: Option<Vec<FileNode>>,
    size: Option<usize>,
    extension: Option<String>,
    line_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    current: usize,
    total: usize,
    percentage: f32,
    message: String,
}

// Progress reporter implementation for Tauri
struct TauriProgressReporter {
    window: tauri::Window,
    total: usize,
}

impl ProgressReporter for TauriProgressReporter {
    fn report(&self, message: &str, percentage: f32) {
        // Add logging to track progress reports
        println!("[PROGRESS] {}% - {}", percentage, message);
        
        // Wrap in panic catcher to prevent crashes in progress reporting
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let current = (self.total as f32 * percentage / 100.0) as usize;
            self.window.emit("parse-progress", ProgressUpdate {
                current,
                total: self.total,
                percentage,
                message: message.to_string(),
            }).ok();
        }));
        
        if let Err(e) = result {
            println!("[PROGRESS] ERROR: Failed to emit progress: {:?}", e);
        }
    }
    
    fn complete(&self, message: Option<&str>) {
        self.report(message.unwrap_or("Complete"), 100.0);
    }
    
    fn error(&self, message: &str, error: Option<&dyn std::error::Error>) {
        let error_msg = if let Some(err) = error {
            format!("{}: {}", message, err)
        } else {
            message.to_string()
        };
        self.window.emit("parse-error", error_msg).ok();
    }
}

// Helper function to build file tree from graph data
fn build_file_tree(graph_data: &GraphData) -> Vec<FileNode> {
    println!("Building file tree from {} nodes", graph_data.nodes.len());
    
    // Collect all unique file paths
    let mut file_paths: HashSet<String> = HashSet::new();
    for node in &graph_data.nodes {
        if node.node_type == "file" {
            if let Some(file_path) = &node.file_path {
                file_paths.insert(file_path.clone());
            }
        }
    }
    
    if file_paths.is_empty() {
        println!("No file nodes found in graph data");
        return Vec::new();
    }
    
    println!("Found {} unique file paths", file_paths.len());
    
    // Find the common base path
    let base_path = find_common_base_path(&file_paths);
    println!("Base path: {:?}", base_path);
    
    // Build the tree structure
    let mut tree_map: HashMap<String, FileNode> = HashMap::new();
    let mut children_map: HashMap<String, Vec<String>> = HashMap::new();
    
    // Process each file
    for file_path in &file_paths {
        let full_path = Path::new(file_path);
        let relative_path = if let Some(base) = &base_path {
            full_path.strip_prefix(base).unwrap_or(full_path)
        } else {
            full_path
        };
        
        // Add the file node
        let file_name = full_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let extension = full_path.extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());
        
        let relative_path_str = relative_path.to_str().unwrap_or("").to_string();
        
        tree_map.insert(relative_path_str.clone(), FileNode {
            name: file_name,
            path: file_path.clone(), // Keep full path for opening files
            node_type: "file".to_string(),
            children: None,
            size: None,
            extension,
            line_count: None,
        });
        
        // Create parent directories
        let mut current = relative_path;
        while let Some(parent) = current.parent() {
            if parent.as_os_str().is_empty() {
                // Reached root
                children_map.entry(String::new()).or_insert_with(Vec::new).push(relative_path_str.clone());
                break;
            }
            
            let parent_str = parent.to_str().unwrap_or("").to_string();
            let current_str = current.to_str().unwrap_or("").to_string();
            
            // Add to children map
            children_map.entry(parent_str.clone()).or_insert_with(Vec::new).push(current_str.clone());
            
            // Create directory node if it doesn't exist
            if !tree_map.contains_key(&parent_str) {
                let dir_name = parent.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(parent.to_str().unwrap_or(""))
                    .to_string();
                
                let full_dir_path = if let Some(base) = &base_path {
                    base.join(parent).to_str().unwrap_or("").to_string()
                } else {
                    parent.to_str().unwrap_or("").to_string()
                };
                
                tree_map.insert(parent_str.clone(), FileNode {
                    name: dir_name,
                    path: full_dir_path,
                    node_type: "folder".to_string(),
                    children: Some(Vec::new()),
                    size: None,
                    extension: None,
                    line_count: None,
                });
            }
            
            current = parent;
        }
    }
    
    println!("Created {} nodes in tree", tree_map.len());
    println!("Children map has {} entries", children_map.len());
    
    // Build parent-child relationships
    for (parent_path, child_paths) in &children_map {
        // Remove duplicates from child_paths
        let unique_children: HashSet<&String> = child_paths.iter().collect();
        
        if parent_path.is_empty() {
            // These are root items, we'll collect them later
            continue;
        }
        
        // Collect child nodes first to avoid borrow issues
        let child_nodes: Vec<FileNode> = unique_children
            .iter()
            .filter_map(|path| tree_map.get(*path).cloned())
            .collect();
        
        if let Some(parent_node) = tree_map.get_mut(parent_path) {
            if parent_node.children.is_none() {
                parent_node.children = Some(Vec::new());
            }
            
            if let Some(ref mut children) = parent_node.children {
                for child_node in child_nodes {
                    // Avoid duplicates
                    if !children.iter().any(|c| c.name == child_node.name) {
                        children.push(child_node);
                    }
                }
                // Sort children by name
                children.sort_by(|a, b| {
                    // Folders first, then files
                    match (a.node_type.as_str(), b.node_type.as_str()) {
                        ("folder", "file") => std::cmp::Ordering::Less,
                        ("file", "folder") => std::cmp::Ordering::Greater,
                        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                    }
                });
            }
        }
    }
    
    // Find root nodes
    let mut root_nodes = Vec::new();
    
    // Get root items from children_map (items with empty parent)
    if let Some(root_children) = children_map.get("") {
        for child_path in root_children {
            if let Some(node) = tree_map.get(child_path) {
                root_nodes.push(node.clone());
            }
        }
    }
    
    // If no root nodes found, find the top-level directories
    if root_nodes.is_empty() {
        println!("No root nodes found in children_map, finding top-level items");
        
        // Find all nodes that are not children of any other node
        let all_children: HashSet<&String> = children_map.values()
            .flat_map(|v| v.iter())
            .collect();
        
        for (path, node) in &tree_map {
            if !all_children.contains(path) && !path.is_empty() {
                root_nodes.push(node.clone());
            }
        }
    }
    
    // If still no root nodes, just show the project name with all files
    if root_nodes.is_empty() && !tree_map.is_empty() {
        println!("Creating synthetic root node");
        
        // Get project name from base path
        let project_name = if let Some(base) = &base_path {
            base.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Project")
                .to_string()
        } else {
            "Project".to_string()
        };
        
        let mut all_items: Vec<FileNode> = tree_map.values().cloned().collect();
        all_items.sort_by(|a, b| {
            // Folders first, then files
            match (a.node_type.as_str(), b.node_type.as_str()) {
                ("folder", "file") => std::cmp::Ordering::Less,
                ("file", "folder") => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
        
        root_nodes.push(FileNode {
            name: project_name,
            path: base_path.as_ref().map(|p| p.to_str().unwrap_or("").to_string()).unwrap_or_default(),
            node_type: "folder".to_string(),
            children: Some(all_items),
            size: None,
            extension: None,
            line_count: None,
        });
    } else {
        // Sort root nodes
        root_nodes.sort_by(|a, b| {
            // Folders first, then files
            match (a.node_type.as_str(), b.node_type.as_str()) {
                ("folder", "file") => std::cmp::Ordering::Less,
                ("file", "folder") => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            }
        });
    }
    
    println!("Returning {} root nodes", root_nodes.len());
    for (i, node) in root_nodes.iter().take(3).enumerate() {
        println!("  Root {}: {} ({})", i, node.name, node.node_type);
    }
    
    root_nodes
}

// Helper function to find common base path
fn find_common_base_path(paths: &HashSet<String>) -> Option<PathBuf> {
    if paths.is_empty() {
        return None;
    }
    
    let mut path_iter = paths.iter();
    let first_path = Path::new(path_iter.next().unwrap());
    
    let mut common = first_path.parent()?.to_path_buf();
    
    for path_str in path_iter {
        let path = Path::new(path_str);
        while !path.starts_with(&common) {
            common = common.parent()?.to_path_buf();
        }
    }
    
    Some(common)
}

// Global engine instance
struct AppState {
    engine: Option<Engine>,
    current_graph: Option<GraphData>,
    analyzed_graph: Option<AnalyzedGraph>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            engine: None,
            current_graph: None,
            analyzed_graph: None,
        }
    }
}

// Parse codebase command
#[tauri::command]
async fn parse_codebase(
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
    window: tauri::Window,
) -> Result<ParseResult, String> {
    println!("Parsing codebase at: {}", path);
    
    let path_buf = PathBuf::from(path);
    
    // Create or update engine with the base path
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.engine = Some(Engine::new(path_buf.clone()));
    }
    
    // Create progress reporter
    let progress: Arc<dyn ProgressReporter + Send + Sync> = Arc::new(TauriProgressReporter {
        window,
        total: 100, // Use percentage-based progress
    });
    
    // Parse and build graph
    let engine_clone = {
        let state_guard = state.lock().unwrap();
        state_guard.engine.as_ref()
            .ok_or_else(|| "Engine not initialized".to_string())?
            .clone()
    };
    
    // Analyze codebase (outside of mutex lock)
    let graph_data = engine_clone.analyze_codebase(Some(progress.clone()))
        .await
        .map_err(|e| format!("Failed to analyze codebase: {}", e))?;
    
    // Build file tree from graph nodes
    println!("Graph data stats: files={}, nodes={}, edges={}", 
        graph_data.stats.file_count, 
        graph_data.stats.node_count, 
        graph_data.stats.link_count
    );
    
    let files = build_file_tree(&graph_data);
    println!("Built file tree with {} root items", files.len());
    
    // Debug: print first few file names
    for (i, file) in files.iter().take(3).enumerate() {
        println!("  Root item {}: {} (type: {})", i, file.name, file.node_type);
    }
    
    let result = ParseResult {
        file_count: graph_data.stats.file_count,
        node_count: graph_data.stats.node_count,
        edge_count: graph_data.stats.link_count,
        languages: vec!["JavaScript".to_string(), "TypeScript".to_string(), "Python".to_string()],
        files,
    };
    
    // Store the graph for later use
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.current_graph = Some(graph_data);
    }
    
    Ok(result)
}

// Generate graph from parsed data
#[tauri::command]
async fn generate_graph(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<GraphData, String> {
    println!("Generating graph...");
    
    let state_guard = state.lock().unwrap();
    
    if let Some(ref graph_data) = state_guard.current_graph {
        Ok(graph_data.clone())
    } else {
        Err("No parsed data available. Please parse a codebase first.".to_string())
    }
}

// Analyze with metrics
#[tauri::command]
async fn analyze_with_metrics(
    path: String,
    state: tauri::State<'_, Mutex<AppState>>,
    window: tauri::Window,
) -> Result<AnalyzedGraph, String> {
    println!("[ANALYZE] Starting analyze_with_metrics at: {}", path);
    tracing::info!("[ANALYZE] Starting analyze_with_metrics at: {}", path);
    
    let path_buf = PathBuf::from(path);
    
    // Create or update engine with the base path
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.engine = Some(Engine::new(path_buf.clone()));
    }
    
    // Create progress reporter
    let progress: Arc<dyn ProgressReporter + Send + Sync> = Arc::new(TauriProgressReporter {
        window: window.clone(),
        total: 100,
    });
    
    // Get engine clone (outside of mutex)
    let engine_clone = {
        let state_guard = state.lock().unwrap();
        state_guard.engine.as_ref()
            .ok_or_else(|| "Engine not initialized".to_string())?
            .clone()
    };
    
    // Try to analyze with metrics, but fall back to basic analysis if it fails
    println!("[ANALYZE] Calling engine.analyze_with_metrics...");
    tracing::info!("[ANALYZE] Calling engine.analyze_with_metrics...");
    
    // Try to run the analysis without blocking
    let analyzed_graph = match engine_clone.analyze_with_metrics(Some(progress.clone())).await {
        Ok(graph) => {
            println!("[ANALYZE] Success! Got analyzed graph");
            tracing::info!("[ANALYZE] Success! Got analyzed graph");
            graph
        },
        Err(e) => {
            println!("[ANALYZE] ERROR: Metrics analysis failed: {}", e);
            tracing::error!("[ANALYZE] ERROR: Metrics analysis failed: {}", e);
            println!("[ANALYZE] Attempting fallback to basic analysis...");
            
            // Try basic analysis without metrics
            let graph_data = engine_clone.analyze_codebase(Some(progress))
                .await
                .map_err(|e| format!("Failed to analyze codebase: {}", e))?;
            
            // Return graph without metrics
            AnalyzedGraph {
                graph_data,
                metrics: Vec::new(),
                summary: AnalysisSummary {
                    total_nodes: 0,
                    total_edges: 0,
                    num_communities: 0,
                    modularity: 0.0,
                    avg_complexity: 0.0,
                    high_risk_count: 0,
                    circular_dependencies: 0,
                },
            }
        }
    };
    
    // Store the results
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.current_graph = Some(analyzed_graph.graph_data.clone());
        state_guard.analyzed_graph = Some(analyzed_graph.clone());
    }
    
    Ok(analyzed_graph)
}

// Connect to Neo4j (placeholder)
#[tauri::command]
async fn connect_neo4j(uri: String, _username: String, _password: String) -> Result<bool, String> {
    println!("Connecting to Neo4j at: {}", uri);
    
    // TODO: Implement actual connection using og-db crate
    Ok(true)
}

// Get saved graph data
#[tauri::command]
async fn get_saved_graph() -> Result<Option<GraphData>, String> {
    // TODO: Load from persistence
    Ok(None)
}

// Reset app state
#[tauri::command]
async fn reset_app(
    state: tauri::State<'_, Mutex<AppState>>,
) -> Result<(), String> {
    println!("Resetting app state...");
    
    let mut state_guard = state.lock().unwrap();
    
    // Clear all state
    state_guard.engine = None;
    state_guard.current_graph = None;
    state_guard.analyzed_graph = None;
    
    println!("App state reset successfully");
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(Mutex::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            parse_codebase,
            connect_neo4j,
            generate_graph,
            analyze_with_metrics,
            get_saved_graph,
            reset_app
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}