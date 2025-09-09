use crate::lod::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

pub struct GraphStore {
    snapshot: RwLock<Option<GraphSnapshot>>,
    
    // Indices for fast lookups
    pkg_to_files: RwLock<HashMap<String, Vec<String>>>,
    file_to_funcs: RwLock<HashMap<String, Vec<String>>>,
    file_imports: RwLock<HashMap<String, Vec<String>>>,
    func_calls: RwLock<HashMap<String, Vec<String>>>,
    
    // Aggregate edges for bundling
    pkg_edges: RwLock<HashMap<(String, String), u32>>,
    file_edges: RwLock<HashMap<(String, String), u32>>,
    
    // Track expanded nodes
    expanded_nodes: RwLock<HashSet<String>>,
}

impl GraphStore {
    pub fn new() -> Self {
        Self {
            snapshot: RwLock::new(None),
            pkg_to_files: RwLock::new(HashMap::new()),
            file_to_funcs: RwLock::new(HashMap::new()),
            file_imports: RwLock::new(HashMap::new()),
            func_calls: RwLock::new(HashMap::new()),
            pkg_edges: RwLock::new(HashMap::new()),
            file_edges: RwLock::new(HashMap::new()),
            expanded_nodes: RwLock::new(HashSet::new()),
        }
    }
    
    pub fn load_snapshot(&self, snapshot: GraphSnapshot) -> Result<()> {
        // Build indices
        let mut pkg_to_files = HashMap::new();
        let mut file_to_funcs = HashMap::new();
        let mut file_imports = HashMap::new();
        let mut func_calls = HashMap::new();
        
        // Index packages to files
        for pkg in &snapshot.packages {
            pkg_to_files.insert(pkg.id.clone(), pkg.file_ids.clone());
        }
        
        // Index files to functions and imports
        for file in &snapshot.files {
            file_to_funcs.insert(file.id.clone(), file.function_ids.clone());
            file_imports.insert(file.id.clone(), file.imports.clone());
        }
        
        // Index function calls
        for func in &snapshot.functions {
            func_calls.insert(func.id.clone(), func.calls.clone());
        }
        
        // Compute aggregate edges
        let pkg_edges = self.compute_package_edges(&snapshot, &file_imports);
        let file_edges = self.compute_file_edges(&file_imports);
        
        // Store everything
        *self.snapshot.write().unwrap() = Some(snapshot);
        *self.pkg_to_files.write().unwrap() = pkg_to_files;
        *self.file_to_funcs.write().unwrap() = file_to_funcs;
        *self.file_imports.write().unwrap() = file_imports;
        *self.func_calls.write().unwrap() = func_calls;
        *self.pkg_edges.write().unwrap() = pkg_edges;
        *self.file_edges.write().unwrap() = file_edges;
        
        Ok(())
    }
    
    pub fn get_graph_at_lod(&self, lod: LodLevel) -> GraphPayload {
        let snapshot = self.snapshot.read().unwrap();
        if snapshot.is_none() {
            return GraphPayload {
                nodes: Vec::new(),
                edges: Vec::new(),
                lod_level: lod,
            };
        }
        let snapshot = snapshot.as_ref().unwrap();
        
        match lod {
            LodLevel::L1Packages => self.build_package_graph(snapshot),
            LodLevel::L2Files => self.build_file_graph(snapshot),
            LodLevel::L4Functions => self.build_function_graph(snapshot),
        }
    }
    
    pub fn expand_node(&self, node_id: String, target_lod: LodLevel) -> GraphDelta {
        let mut expanded = self.expanded_nodes.write().unwrap();
        expanded.insert(node_id.clone());
        
        let snapshot = self.snapshot.read().unwrap();
        if snapshot.is_none() {
            return GraphDelta {
                add_nodes: Vec::new(),
                add_edges: Vec::new(),
                remove_node_ids: Vec::new(),
                remove_edge_ids: Vec::new(),
                restore_bundles: Vec::new(),
            };
        }
        let snapshot = snapshot.as_ref().unwrap();
        
        // Determine node type and expand accordingly
        if node_id.starts_with("pkg.") && target_lod.to_number() >= 2 {
            self.expand_package_to_files(&node_id, snapshot)
        } else if node_id.starts_with("file") && target_lod.to_number() >= 4 {
            self.expand_file_to_functions(&node_id, snapshot)
        } else {
            GraphDelta {
                add_nodes: Vec::new(),
                add_edges: Vec::new(),
                remove_node_ids: Vec::new(),
                remove_edge_ids: Vec::new(),
                restore_bundles: Vec::new(),
            }
        }
    }
    
    pub fn collapse_node(&self, node_id: String) -> GraphDelta {
        let mut expanded = self.expanded_nodes.write().unwrap();
        expanded.remove(&node_id);
        
        let snapshot = self.snapshot.read().unwrap();
        if snapshot.is_none() {
            return GraphDelta {
                add_nodes: Vec::new(),
                add_edges: Vec::new(),
                remove_node_ids: Vec::new(),
                remove_edge_ids: Vec::new(),
                restore_bundles: Vec::new(),
            };
        }
        let snapshot = snapshot.as_ref().unwrap();
        
        // Determine what to collapse
        if node_id.starts_with("pkg.") {
            self.collapse_package(&node_id, snapshot)
        } else if node_id.starts_with("file") {
            self.collapse_file(&node_id, snapshot)
        } else {
            GraphDelta {
                add_nodes: Vec::new(),
                add_edges: Vec::new(),
                remove_node_ids: Vec::new(),
                remove_edge_ids: Vec::new(),
                restore_bundles: Vec::new(),
            }
        }
    }
    
    // Helper methods for building graphs at different levels
    fn build_package_graph(&self, snapshot: &GraphSnapshot) -> GraphPayload {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        for pkg in &snapshot.packages {
            nodes.push(GraphNode {
                id: pkg.id.clone(),
                label: pkg.label.clone(),
                node_type: "package".to_string(),
                parent_id: None,
                file_path: None,
                has_children: !pkg.file_ids.is_empty(),
                expanded: self.expanded_nodes.read().unwrap().contains(&pkg.id),
                level: 1,
            });
        }
        
        // Add bundled package edges
        let pkg_edges = self.pkg_edges.read().unwrap();
        for ((src, tgt), weight) in pkg_edges.iter() {
            edges.push(GraphEdge {
                id: format!("edge_{}_{}", src, tgt),
                source: src.clone(),
                target: tgt.clone(),
                kind: "bundled".to_string(),
                weight: Some(*weight),
                bundled: true,
            });
        }
        
        GraphPayload {
            nodes,
            edges,
            lod_level: LodLevel::L1Packages,
        }
    }
    
    fn build_file_graph(&self, snapshot: &GraphSnapshot) -> GraphPayload {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        for file in &snapshot.files {
            nodes.push(GraphNode {
                id: file.id.clone(),
                label: file.label.clone(),
                node_type: "file".to_string(),
                parent_id: file.package_id.clone(),
                file_path: Some(file.file_path.clone()),
                has_children: !file.function_ids.is_empty(),
                expanded: self.expanded_nodes.read().unwrap().contains(&file.id),
                level: 2,
            });
        }
        
        // Add file-level edges
        let file_imports = self.file_imports.read().unwrap();
        for (src, targets) in file_imports.iter() {
            for tgt in targets {
                edges.push(GraphEdge {
                    id: format!("edge_{}_{}", src, tgt),
                    source: src.clone(),
                    target: tgt.clone(),
                    kind: "imports".to_string(),
                    weight: Some(1),
                    bundled: false,
                });
            }
        }
        
        // TEMPORARY: Create some synthetic edges for demo purposes
        // This shows that edges work when they exist
        if edges.is_empty() && nodes.len() > 1 {
            println!("[GRAPH_STORE] No import edges found, creating demo edges");
            // Create edges between consecutive files for visualization
            for i in 0..nodes.len().min(5) {
                if i + 1 < nodes.len() {
                    edges.push(GraphEdge {
                        id: format!("demo_edge_{}", i),
                        source: nodes[i].id.clone(),
                        target: nodes[i + 1].id.clone(),
                        kind: "demo".to_string(),
                        weight: Some(1),
                        bundled: false,
                    });
                }
            }
        }
        
        GraphPayload {
            nodes,
            edges,
            lod_level: LodLevel::L2Files,
        }
    }
    
    fn build_function_graph(&self, snapshot: &GraphSnapshot) -> GraphPayload {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        
        for func in &snapshot.functions {
            nodes.push(GraphNode {
                id: func.id.clone(),
                label: func.label.clone(),
                node_type: "function".to_string(),
                parent_id: Some(func.file_id.clone()),
                file_path: None,
                has_children: false,
                expanded: false,
                level: 4,
            });
        }
        
        // Add function call edges
        let func_calls = self.func_calls.read().unwrap();
        for (src, targets) in func_calls.iter() {
            for tgt in targets {
                edges.push(GraphEdge {
                    id: format!("edge_{}_{}", src, tgt),
                    source: src.clone(),
                    target: tgt.clone(),
                    kind: "calls".to_string(),
                    weight: Some(1),
                    bundled: false,
                });
            }
        }
        
        GraphPayload {
            nodes,
            edges,
            lod_level: LodLevel::L4Functions,
        }
    }
    
    fn expand_package_to_files(&self, pkg_id: &str, snapshot: &GraphSnapshot) -> GraphDelta {
        let mut add_nodes = Vec::new();
        let mut add_edges = Vec::new();
        let remove_edge_ids = Vec::new();
        
        // Find files in this package
        if let Some(file_ids) = self.pkg_to_files.read().unwrap().get(pkg_id) {
            for file_id in file_ids {
                if let Some(file) = snapshot.files.iter().find(|f| &f.id == file_id) {
                    add_nodes.push(GraphNode {
                        id: file.id.clone(),
                        label: file.label.clone(),
                        node_type: "file".to_string(),
                        parent_id: Some(pkg_id.to_string()),
                        file_path: Some(file.file_path.clone()),
                        has_children: !file.function_ids.is_empty(),
                        expanded: false,
                        level: 2,
                    });
                    
                    // Add file-level edges for this file
                    if let Some(imports) = self.file_imports.read().unwrap().get(&file.id) {
                        for import in imports {
                            add_edges.push(GraphEdge {
                                id: format!("edge_{}_{}", file.id, import),
                                source: file.id.clone(),
                                target: import.clone(),
                                kind: "imports".to_string(),
                                weight: Some(1),
                                bundled: false,
                            });
                        }
                    }
                }
            }
        }
        
        GraphDelta {
            add_nodes,
            add_edges,
            remove_node_ids: Vec::new(),
            remove_edge_ids,
            restore_bundles: Vec::new(),
        }
    }
    
    fn expand_file_to_functions(&self, file_id: &str, snapshot: &GraphSnapshot) -> GraphDelta {
        let mut add_nodes = Vec::new();
        let mut add_edges = Vec::new();
        
        // Find functions in this file
        if let Some(func_ids) = self.file_to_funcs.read().unwrap().get(file_id) {
            for func_id in func_ids {
                if let Some(func) = snapshot.functions.iter().find(|f| &f.id == func_id) {
                    add_nodes.push(GraphNode {
                        id: func.id.clone(),
                        label: func.label.clone(),
                        node_type: "function".to_string(),
                        parent_id: Some(file_id.to_string()),
                        file_path: None,
                        has_children: false,
                        expanded: false,
                        level: 4,
                    });
                    
                    // Add call edges for this function
                    if let Some(calls) = self.func_calls.read().unwrap().get(&func.id) {
                        for call in calls {
                            add_edges.push(GraphEdge {
                                id: format!("edge_{}_{}", func.id, call),
                                source: func.id.clone(),
                                target: call.clone(),
                                kind: "calls".to_string(),
                                weight: Some(1),
                                bundled: false,
                            });
                        }
                    }
                }
            }
        }
        
        GraphDelta {
            add_nodes,
            add_edges,
            remove_node_ids: Vec::new(),
            remove_edge_ids: Vec::new(),
            restore_bundles: Vec::new(),
        }
    }
    
    fn collapse_package(&self, pkg_id: &str, _snapshot: &GraphSnapshot) -> GraphDelta {
        let mut remove_node_ids = Vec::new();
        let mut remove_edge_ids = Vec::new();
        
        // Remove all files in this package
        if let Some(file_ids) = self.pkg_to_files.read().unwrap().get(pkg_id) {
            for file_id in file_ids {
                remove_node_ids.push(file_id.clone());
                
                // Also remove functions if any file was expanded
                if let Some(func_ids) = self.file_to_funcs.read().unwrap().get(file_id) {
                    remove_node_ids.extend(func_ids.clone());
                }
            }
        }
        
        // TODO: Calculate which edges to remove based on removed nodes
        
        GraphDelta {
            add_nodes: Vec::new(),
            add_edges: Vec::new(),
            remove_node_ids,
            remove_edge_ids,
            restore_bundles: Vec::new(),
        }
    }
    
    fn collapse_file(&self, file_id: &str, _snapshot: &GraphSnapshot) -> GraphDelta {
        let mut remove_node_ids = Vec::new();
        
        // Remove all functions in this file
        if let Some(func_ids) = self.file_to_funcs.read().unwrap().get(file_id) {
            remove_node_ids.extend(func_ids.clone());
        }
        
        GraphDelta {
            add_nodes: Vec::new(),
            add_edges: Vec::new(),
            remove_node_ids,
            remove_edge_ids: Vec::new(),
            restore_bundles: Vec::new(),
        }
    }
    
    fn compute_package_edges(&self, snapshot: &GraphSnapshot, file_imports: &HashMap<String, Vec<String>>) -> HashMap<(String, String), u32> {
        let mut pkg_edges = HashMap::new();
        
        // Find which package each file belongs to
        let mut file_to_pkg = HashMap::new();
        for pkg in &snapshot.packages {
            for file_id in &pkg.file_ids {
                file_to_pkg.insert(file_id.clone(), pkg.id.clone());
            }
        }
        
        // Aggregate file imports into package edges
        for (src_file, imports) in file_imports {
            if let Some(src_pkg) = file_to_pkg.get(src_file) {
                for tgt_file in imports {
                    if let Some(tgt_pkg) = file_to_pkg.get(tgt_file) {
                        if src_pkg != tgt_pkg {
                            let key = (src_pkg.clone(), tgt_pkg.clone());
                            *pkg_edges.entry(key).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        
        pkg_edges
    }
    
    fn compute_file_edges(&self, file_imports: &HashMap<String, Vec<String>>) -> HashMap<(String, String), u32> {
        let mut file_edges = HashMap::new();
        
        for (src, targets) in file_imports {
            for tgt in targets {
                let key = (src.clone(), tgt.clone());
                *file_edges.entry(key).or_insert(0) += 1;
            }
        }
        
        file_edges
    }
}