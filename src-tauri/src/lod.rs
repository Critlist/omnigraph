use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use og_graph::graph::{GraphData, GraphNode as OgGraphNode, GraphLink};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum LodLevel {
    L1Packages,
    L2Files,
    L4Functions,
}

impl LodLevel {
    pub fn to_number(&self) -> u8 {
        match self {
            LodLevel::L1Packages => 1,
            LodLevel::L2Files => 2,
            LodLevel::L4Functions => 4,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub parent_id: Option<String>,
    pub file_path: Option<String>,
    pub has_children: bool,
    pub expanded: bool,
    pub level: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub kind: String,
    pub weight: Option<u32>,
    pub bundled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphPayload {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub lod_level: LodLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphDelta {
    pub add_nodes: Vec<GraphNode>,
    pub add_edges: Vec<GraphEdge>,
    pub remove_node_ids: Vec<String>,
    pub remove_edge_ids: Vec<String>,
    pub restore_bundles: Vec<GraphEdge>,
}

// Snapshot structures for persisting parsed data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageNode {
    pub id: String,
    pub label: String,
    pub file_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileNode {
    pub id: String,
    pub label: String,
    pub package_id: Option<String>,
    pub function_ids: Vec<String>,
    pub imports: Vec<String>,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FunctionNode {
    pub id: String,
    pub label: String,
    pub file_id: String,
    pub calls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GraphSnapshot {
    pub packages: Vec<PackageNode>,
    pub files: Vec<FileNode>,
    pub functions: Vec<FunctionNode>,
}

impl GraphSnapshot {
    pub fn from_parsed_graph(graph_data: &GraphData) -> Self {
        // Convert from the existing parsed graph format
        let mut packages: HashMap<String, PackageNode> = HashMap::new();
        let mut files: HashMap<String, FileNode> = HashMap::new();
        let mut functions: HashMap<String, FunctionNode> = HashMap::new();
        
        // Process nodes to build hierarchy
        for node in &graph_data.nodes {
            match node.node_type.as_str() {
                "file" => {
                    let package_id = Self::derive_package_from_path(&node.file_path);
                    
                    // Create or update package
                    let pkg_entry = packages.entry(package_id.clone()).or_insert(PackageNode {
                        id: package_id.clone(),
                        label: Self::extract_package_name(&package_id),
                        file_ids: Vec::new(),
                    });
                    pkg_entry.file_ids.push(node.id.clone());
                    
                    // Create file node
                    files.insert(node.id.clone(), FileNode {
                        id: node.id.clone(),
                        label: node.name.clone(),
                        package_id: Some(package_id),
                        function_ids: Vec::new(),
                        imports: Vec::new(),
                        file_path: node.file_path.clone().unwrap_or_default(),
                    });
                }
                "function" | "class" | "method" => {
                    // Find parent file
                    let file_id = Self::find_parent_file(&node.id, &graph_data.links);
                    if let Some(file_id) = file_id {
                        if let Some(file) = files.get_mut(&file_id) {
                            file.function_ids.push(node.id.clone());
                        }
                        
                        functions.insert(node.id.clone(), FunctionNode {
                            id: node.id.clone(),
                            label: node.name.clone(),
                            file_id,
                            calls: Vec::new(),
                        });
                    }
                }
                _ => {}
            }
        }
        
        // Process edges to extract imports and calls
        println!("[LOD] Processing {} links", graph_data.links.len());
        
        // Create a mapping from file nodes to their IDs for import resolution
        let file_id_map: HashMap<String, String> = files.iter()
            .map(|(id, file)| (file.label.clone(), id.clone()))
            .collect();
        println!("[LOD] File ID map: {:?}", file_id_map);
        
        // Collect import relationships to add later
        let mut file_import_relationships: Vec<(String, String)> = Vec::new();
        let mut func_call_relationships: Vec<(String, String)> = Vec::new();
        
        for link in &graph_data.links {
            println!("[LOD] Link: {} -> {} (type: {})", link.source, link.target, link.link_type);
            match link.link_type.as_str() {
                "imports" | "includes" => {
                    if files.contains_key(&link.source) {
                        // Try to resolve the target to an actual file
                        // The target might be a file ID that doesn't exist (for system includes)
                        // Check if target exists in our files
                        if files.contains_key(&link.target) {
                            file_import_relationships.push((link.source.clone(), link.target.clone()));
                            println!("[LOD] Added import: {} imports {}", link.source, link.target);
                        } else {
                            // Try to find the file by name
                            // Extract filename from include path if it looks like a path
                            let target_name = link.target
                                .split('/')
                                .last()
                                .unwrap_or(&link.target)
                                .replace("file_", "")
                                .replace("_h", ".h")
                                .replace("_c", ".c");
                            
                            if let Some(target_id) = file_id_map.get(&target_name) {
                                file_import_relationships.push((link.source.clone(), target_id.clone()));
                                println!("[LOD] Resolved import: {} imports {} (via {})", link.source, target_id, target_name);
                            } else {
                                println!("[LOD] Could not resolve import target: {}", link.target);
                            }
                        }
                    }
                }
                "calls" => {
                    if functions.contains_key(&link.source) {
                        func_call_relationships.push((link.source.clone(), link.target.clone()));
                        println!("[LOD] Added call: {} calls {}", link.source, link.target);
                    }
                }
                "contains" => {
                    // Contains relationships establish hierarchy, already handled via parent_id
                    // But we can use them to verify our hierarchy is correct
                }
                _ => {
                    println!("[LOD] Unknown link type: {}", link.link_type);
                }
            }
        }
        
        // Now apply the collected relationships
        for (source, target) in file_import_relationships {
            if let Some(file) = files.get_mut(&source) {
                file.imports.push(target);
            }
        }
        
        for (source, target) in func_call_relationships {
            if let Some(func) = functions.get_mut(&source) {
                func.calls.push(target);
            }
        }
        
        let snapshot = GraphSnapshot {
            packages: packages.into_values().collect(),
            files: files.into_values().collect(),
            functions: functions.into_values().collect(),
        };
        
        snapshot
    }
    
    fn derive_package_from_path(path: &Option<String>) -> String {
        if let Some(p) = path {
            // Extract package from directory structure
            let parts: Vec<&str> = p.split('/').collect();
            if parts.len() > 1 {
                return format!("pkg.{}", parts[0]);
            }
        }
        "pkg.root".to_string()
    }
    
    fn extract_package_name(pkg_id: &str) -> String {
        pkg_id.replace("pkg.", "")
    }
    
    fn find_parent_file(node_id: &str, links: &[GraphLink]) -> Option<String> {
        for link in links {
            if link.target == node_id && link.link_type == "contains" {
                // Check if source is a file
                if link.source.contains("file") {
                    return Some(link.source.clone());
                }
            }
        }
        None
    }
}