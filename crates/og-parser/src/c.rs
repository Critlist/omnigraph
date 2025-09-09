use og_types::{
    AstNode, Language, NodeType, ParsedFile, Relationship, RelationshipType,
    EngineResult, EngineError, FileMetrics, ParseError,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tree_sitter::{Node, Parser as TSParser, TreeCursor};
use tracing::{debug, trace};

pub struct CParser {
    parser: Mutex<TSParser>,
    base_path: PathBuf,
}

impl CParser {
    pub fn new() -> Self {
        Self::with_base_path(PathBuf::from("."))
    }
    
    pub fn with_base_path(base_path: PathBuf) -> Self {
        let mut parser = TSParser::new();
        parser
            .set_language(tree_sitter_c::language())
            .expect("Error loading C grammar");
        Self {
            parser: Mutex::new(parser),
            base_path,
        }
    }

    fn generate_unique_id(file_path: &str, node_type: &str, counter: usize) -> String {
        let mut hasher = DefaultHasher::new();
        file_path.hash(&mut hasher);
        let file_hash = hasher.finish();
        format!("{}_{:x}_{}", node_type, file_hash, counter)
    }

    fn extract_nodes(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &str,
    ) -> (Vec<AstNode>, Vec<Relationship>) {
        let mut nodes = Vec::new();
        let mut relationships = Vec::new();
        
        // Generate unique file ID based on file path
        let file_id = Self::generate_unique_id(file_path, "file", 0);
        println!("[C_PARSER] Parsing file: {} with ID: {}", file_path, file_id);
        
        let mut node_counter = 0;
        
        nodes.push(AstNode {
            id: file_id.clone(),
            node_type: NodeType::File,
            name: Path::new(file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| file_path.to_string()),
            start_line: 0,
            end_line: source.lines().count(),
            children: Vec::new(),
        });

        self.walk_tree(
            cursor,
            source,
            file_path,
            &file_id,
            &mut nodes,
            &mut relationships,
            &mut node_counter,
        );

        (nodes, relationships)
    }

    fn walk_tree(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &str,
        parent_id: &str,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        node_counter: &mut usize,
    ) {
        loop {
            let node = cursor.node();
            let node_kind = node.kind();

            trace!("Processing node: {} at {:?}", node_kind, node.range());

            match node_kind {
                "preproc_include" => {
                    if let Some((include_node, include_path)) = self.extract_include_with_path(node, source, file_path, node_counter, file_path) {
                        let include_id = include_node.id.clone();
                        
                        // Add include node
                        nodes.push(include_node);
                        
                        // Create contains relationship
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: include_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                        
                        // Create import relationship to the actual file
                        if let Some(resolved_path) = self.resolve_include_path(&include_path, file_path) {
                            println!("[C_PARSER] Creating import: {} -> {} (from include: {})", 
                                     parent_id, resolved_path, include_path);
                            relationships.push(Relationship {
                                source: parent_id.to_string(),
                                target: resolved_path,
                                relationship_type: RelationshipType::Imports,
                            });
                        } else {
                            println!("[C_PARSER] Could not resolve include path: {}", include_path);
                        }
                    }
                }
                "function_definition" => {
                    if let Some(func_node) = self.extract_function(node, source, node_counter, file_path) {
                        let func_id = func_node.id.clone();
                        let func_name = func_node.name.clone();
                        
                        // Add function node
                        nodes.push(func_node);
                        
                        // Create contains relationship
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: func_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                        
                        // Extract function calls from the body
                        let calls = self.extract_function_calls(node, source);
                        for called_func in calls {
                            relationships.push(Relationship {
                                source: func_id.clone(),
                                target: format!("function_{}", called_func),
                                relationship_type: RelationshipType::Calls,
                            });
                        }
                        
                        // Recursively process function body
                        if cursor.goto_first_child() {
                            self.walk_tree(
                                cursor,
                                source,
                                file_path,
                                &func_id,
                                nodes,
                                relationships,
                                node_counter,
                            );
                            cursor.goto_parent();
                        }
                    }
                }
                "struct_specifier" => {
                    if let Some(struct_node) = self.extract_struct(node, source, node_counter, file_path) {
                        let struct_id = struct_node.id.clone();
                        
                        // Add struct node
                        nodes.push(struct_node);
                        
                        // Create contains relationship
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: struct_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                    }
                }
                "enum_specifier" => {
                    if let Some(enum_node) = self.extract_enum(node, source, node_counter, file_path) {
                        let enum_id = enum_node.id.clone();
                        
                        // Add enum node
                        nodes.push(enum_node);
                        
                        // Create contains relationship
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: enum_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                    }
                }
                "declaration" => {
                    // Check for function declarations (prototypes)
                    if self.is_function_declaration(node, source) {
                        if let Some(decl_node) = self.extract_function_declaration(node, source, node_counter) {
                            let decl_id = decl_node.id.clone();
                            nodes.push(decl_node);
                            relationships.push(Relationship {
                                source: parent_id.to_string(),
                                target: decl_id.clone(),
                                relationship_type: RelationshipType::Contains,
                            });
                        }
                    }
                    // Check for global variables
                    else if let Some(var_node) = self.extract_global_variable(node, source, node_counter) {
                        let var_id = var_node.id.clone();
                        
                        // Add variable node
                        nodes.push(var_node);
                        
                        // Create contains relationship
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: var_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                    }
                }
                "preproc_def" => {
                    // Extract macro definitions
                    if let Some(macro_node) = self.extract_macro_definition(node, source, node_counter, file_path) {
                        let macro_id = macro_node.id.clone();
                        nodes.push(macro_node);
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: macro_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                    }
                }
                "type_definition" => {
                    // Extract typedef
                    if let Some(typedef_node) = self.extract_typedef(node, source, node_counter, file_path) {
                        let typedef_id = typedef_node.id.clone();
                        nodes.push(typedef_node);
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: typedef_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                    }
                }
                _ => {
                    // Recursively process other nodes
                    if cursor.goto_first_child() {
                        self.walk_tree(
                            cursor,
                            source,
                            file_path,
                            parent_id,
                            nodes,
                            relationships,
                            node_counter,
                        );
                        cursor.goto_parent();
                    }
                }
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    fn extract_include_with_path(
        &self,
        node: Node,
        source: &str,
        current_file: &str,
        node_counter: &mut usize,
        file_path: &str,
    ) -> Option<(AstNode, String)> {
        // Find the path node
        let mut cursor = node.walk();
        let mut include_path = None;
        let mut raw_include = None;
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "string_literal" || child.kind() == "system_lib_string" {
                    let text = child.utf8_text(source.as_bytes()).ok()?;
                    raw_include = Some(text.to_string());
                    // Remove quotes and angle brackets for display
                    include_path = Some(text.trim_matches(|c| c == '"' || c == '<' || c == '>').to_string());
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        let path = include_path?;
        let raw = raw_include?;
        
        let id = Self::generate_unique_id(file_path, "include", *node_counter);
        *node_counter += 1;
        
        let node = AstNode {
            id,
            node_type: NodeType::Import,
            name: format!("#include {}", raw),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        };
        
        Some((node, path))
    }

    fn resolve_include_path(&self, include_path: &str, current_file: &str) -> Option<String> {
        // Generate a file ID that matches how we create file IDs
        // This should match the ID of the target file when it's parsed
        
        // For system includes like <stdio.h>, we might not have the actual file
        // For local includes, we need to resolve to the actual file path
        
        // Try to resolve the include path to an actual file path
        let current_dir = Path::new(current_file).parent()?;
        
        // Remove quotes or angle brackets from include path
        let clean_path = include_path
            .trim_start_matches('"')
            .trim_end_matches('"')
            .trim_start_matches('<')
            .trim_end_matches('>');
        
        // Try to resolve relative to the current file's directory
        let resolved_path = if clean_path.starts_with('/') {
            // Absolute path
            PathBuf::from(clean_path)
        } else {
            // Relative path - resolve relative to current file's directory
            current_dir.join(clean_path)
        };
        
        // Convert to a string path that matches how files are parsed
        let path_str = resolved_path.to_str()?;
        
        // Generate the same ID that would be generated when this file is parsed
        let file_id = Self::generate_unique_id(path_str, "file", 0);
        
        println!("[C_PARSER] Resolved include '{}' to path '{}' with ID '{}'", 
                 include_path, path_str, file_id);
        
        Some(file_id)
    }
    
    fn extract_function_calls(&self, func_node: Node, source: &str) -> Vec<String> {
        let mut calls = Vec::new();
        let mut cursor = func_node.walk();
        
        self.find_calls_recursive(&mut cursor, source, &mut calls);
        calls
    }
    
    fn find_calls_recursive(&self, cursor: &mut TreeCursor, source: &str, calls: &mut Vec<String>) {
        loop {
            let node = cursor.node();
            
            if node.kind() == "call_expression" {
                // Extract the function name being called
                if let Some(name) = self.extract_call_name(node, source) {
                    if !calls.contains(&name) {
                        calls.push(name);
                    }
                }
            }
            
            // Recurse into children
            if cursor.goto_first_child() {
                self.find_calls_recursive(cursor, source, calls);
                cursor.goto_parent();
            }
            
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    
    fn extract_call_name(&self, node: Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "identifier" {
                    return child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }
    
    fn is_function_declaration(&self, node: Node, source: &str) -> bool {
        // Check if this declaration contains a function declarator
        let mut cursor = node.walk();
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "function_declarator" {
                    return true;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        false
    }
    
    fn extract_function_declaration(
        &self,
        node: Node,
        source: &str,
        node_counter: &mut usize,
    ) -> Option<AstNode> {
        let mut cursor = node.walk();
        let mut function_name = None;
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "function_declarator" {
                    // Get the identifier from the declarator
                    let mut decl_cursor = child.walk();
                    if decl_cursor.goto_first_child() {
                        loop {
                            let decl_child = decl_cursor.node();
                            if decl_child.kind() == "identifier" {
                                function_name = decl_child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                                break;
                            }
                            if !decl_cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        let name = function_name?;
        
        let id = format!("func_decl_{}", node_counter);
        *node_counter += 1;
        
        Some(AstNode {
            id,
            node_type: NodeType::Function,
            name: format!("{} (declaration)", name),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        })
    }
    
    fn extract_macro_definition(
        &self,
        node: Node,
        source: &str,
        node_counter: &mut usize,
        file_path: &str,
    ) -> Option<AstNode> {
        let mut cursor = node.walk();
        let mut macro_name = None;
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "identifier" {
                    macro_name = child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        let name = macro_name.unwrap_or_else(|| "MACRO".to_string());
        
        let id = Self::generate_unique_id(file_path, "macro", *node_counter);
        *node_counter += 1;
        
        Some(AstNode {
            id,
            node_type: NodeType::Variable, // Using Variable for macros
            name: format!("#define {}", name),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        })
    }
    
    fn extract_typedef(
        &self,
        node: Node,
        source: &str,
        node_counter: &mut usize,
        file_path: &str,
    ) -> Option<AstNode> {
        let text = node.utf8_text(source.as_bytes()).ok()?;
        
        // Simple extraction - get the last identifier as the typedef name
        let words: Vec<&str> = text.split_whitespace().collect();
        let name = words.last()?.trim_end_matches(';').to_string();
        
        let id = Self::generate_unique_id(file_path, "typedef", *node_counter);
        *node_counter += 1;
        
        Some(AstNode {
            id,
            node_type: NodeType::Class, // Using Class for typedefs
            name: format!("typedef {}", name),
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        })
    }

    fn extract_function(
        &self,
        node: Node,
        source: &str,
        node_counter: &mut usize,
        file_path: &str,
    ) -> Option<AstNode> {
        let mut cursor = node.walk();
        let mut function_name = None;
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "function_declarator" {
                    // Get the identifier from the declarator
                    let mut decl_cursor = child.walk();
                    if decl_cursor.goto_first_child() {
                        loop {
                            let decl_child = decl_cursor.node();
                            if decl_child.kind() == "identifier" {
                                function_name = decl_child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                                break;
                            }
                            if !decl_cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        let name = function_name.unwrap_or_else(|| "anonymous".to_string());
        
        let id = Self::generate_unique_id(file_path, "function", *node_counter);
        *node_counter += 1;
        
        Some(AstNode {
            id,
            node_type: NodeType::Function,
            name,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        })
    }

    fn extract_struct(
        &self,
        node: Node,
        source: &str,
        node_counter: &mut usize,
        file_path: &str,
    ) -> Option<AstNode> {
        let mut cursor = node.walk();
        let mut struct_name = None;
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "type_identifier" {
                    struct_name = child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        let name = struct_name.unwrap_or_else(|| "anonymous_struct".to_string());
        
        let id = Self::generate_unique_id(file_path, "struct", *node_counter);
        *node_counter += 1;
        
        Some(AstNode {
            id,
            node_type: NodeType::Class, // Using Class type for structs
            name,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        })
    }

    fn extract_enum(
        &self,
        node: Node,
        source: &str,
        node_counter: &mut usize,
        file_path: &str,
    ) -> Option<AstNode> {
        let mut cursor = node.walk();
        let mut enum_name = None;
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "type_identifier" {
                    enum_name = child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        let name = enum_name.unwrap_or_else(|| "anonymous_enum".to_string());
        
        let id = Self::generate_unique_id(file_path, "enum", *node_counter);
        *node_counter += 1;
        
        Some(AstNode {
            id,
            node_type: NodeType::Module, // Using Module type for enums
            name,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        })
    }

    fn extract_global_variable(
        &self,
        node: Node,
        source: &str,
        node_counter: &mut usize,
    ) -> Option<AstNode> {
        // Skip typedef declarations
        let text = node.utf8_text(source.as_bytes()).ok()?;
        if text.starts_with("typedef") {
            return None;
        }
        
        let mut cursor = node.walk();
        let mut var_name = None;
        
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "init_declarator" {
                    // Get the identifier from the init_declarator
                    let mut init_cursor = child.walk();
                    if init_cursor.goto_first_child() {
                        loop {
                            let init_child = init_cursor.node();
                            if init_child.kind() == "identifier" {
                                var_name = init_child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                                break;
                            }
                            if !init_cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }
                    break;
                } else if child.kind() == "identifier" {
                    var_name = child.utf8_text(source.as_bytes()).ok().map(|s| s.to_string());
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        
        let name = var_name?;
        
        let id = format!("variable_{}", node_counter);
        *node_counter += 1;
        
        Some(AstNode {
            id,
            node_type: NodeType::Variable,
            name,
            start_line: node.start_position().row + 1,
            end_line: node.end_position().row + 1,
            children: Vec::new(),
        })
    }

    fn calculate_metrics(&self, source: &str, nodes: &[AstNode]) -> FileMetrics {
        let lines: Vec<&str> = source.lines().collect();
        let mut code_lines = 0;
        let mut in_multiline_comment = false;
        
        for line in &lines {
            let trimmed = line.trim();
            
            // Handle multi-line comments
            if in_multiline_comment {
                if trimmed.contains("*/") {
                    in_multiline_comment = false;
                }
            } else if trimmed.starts_with("/*") {
                if !trimmed.contains("*/") {
                    in_multiline_comment = true;
                }
            } else if !trimmed.starts_with("//") && !trimmed.is_empty() {
                code_lines += 1;
            }
        }
        
        let functions = nodes.iter().filter(|n| n.node_type == NodeType::Function).count();
        let classes = nodes.iter().filter(|n| n.node_type == NodeType::Class).count();
        let imports = nodes.iter().filter(|n| n.node_type == NodeType::Import).count();
        
        FileMetrics {
            lines_of_code: code_lines,
            complexity: 1, // Basic complexity
            functions,
            classes,
            imports,
            exports: 0, // C doesn't have explicit exports
        }
    }
}

impl crate::parser_trait::Parser for CParser {
    fn supported_extensions(&self) -> &[&str] {
        &[".c", ".h"]
    }

    fn language(&self) -> Language {
        Language::C
    }

    fn parse(&self, path: &Path, content: &str) -> EngineResult<ParsedFile> {
        let mut parser = self.parser.lock().unwrap();
        
        debug!("Parsing C file: {:?}", path);
        
        let tree = parser
            .parse(content, None)
            .ok_or_else(|| EngineError::Parse(ParseError::ParseFailed(format!("Failed to parse {:?}", path))))?;
        
        let mut cursor = tree.root_node().walk();
        let file_path = path.to_string_lossy().to_string();
        
        let (nodes, relationships) = self.extract_nodes(&mut cursor, content, &file_path);
        
        debug!(
            "Extracted {} nodes and {} relationships from {:?}",
            nodes.len(),
            relationships.len(),
            path
        );
        
        // Calculate metrics
        let metrics = self.calculate_metrics(content, &nodes);
        
        Ok(ParsedFile {
            path: path.to_path_buf(),
            language: Language::C,
            nodes,
            relationships,
            metrics,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Parser;
    use std::path::PathBuf;

    #[test]
    fn test_parse_c_function() {
        let parser = CParser::new();
        let source = r#"
#include <stdio.h>

int add(int a, int b) {
    return a + b;
}

int main() {
    printf("Hello, World!\n");
    return 0;
}
"#;
        let path = PathBuf::from("test.c");
        let result = parser.parse(&path, source).unwrap();
        
        // Should have at least: file node, include node, and 2 functions
        assert!(result.nodes.len() >= 4);
        
        // Check for function nodes
        let functions: Vec<_> = result.nodes.iter()
            .filter(|n| n.node_type == NodeType::Function)
            .collect();
        assert_eq!(functions.len(), 2);
        
        let function_names: Vec<_> = functions.iter().map(|f| f.name.as_str()).collect();
        assert!(function_names.contains(&"add"));
        assert!(function_names.contains(&"main"));
    }

    #[test]
    fn test_parse_c_struct() {
        let parser = CParser::new();
        let source = r#"
struct Point {
    int x;
    int y;
};

struct Rectangle {
    struct Point top_left;
    struct Point bottom_right;
};
"#;
        let path = PathBuf::from("test.c");
        let result = parser.parse(&path, source).unwrap();
        
        // Check for struct nodes
        let structs: Vec<_> = result.nodes.iter()
            .filter(|n| n.node_type == NodeType::Class)
            .collect();
        assert_eq!(structs.len(), 2);
        
        let struct_names: Vec<_> = structs.iter().map(|s| s.name.as_str()).collect();
        assert!(struct_names.contains(&"Point"));
        assert!(struct_names.contains(&"Rectangle"));
    }

    #[test]
    fn test_parse_c_enum() {
        let parser = CParser::new();
        let source = r#"
enum Color {
    RED,
    GREEN,
    BLUE
};

enum Status {
    SUCCESS = 0,
    ERROR = -1
};
"#;
        let path = PathBuf::from("test.c");
        let result = parser.parse(&path, source).unwrap();
        
        // Check for enum nodes (stored as Module type)
        let enums: Vec<_> = result.nodes.iter()
            .filter(|n| n.node_type == NodeType::Module)
            .collect();
        assert_eq!(enums.len(), 2);
        
        let enum_names: Vec<_> = enums.iter().map(|e| e.name.as_str()).collect();
        assert!(enum_names.contains(&"Color"));
        assert!(enum_names.contains(&"Status"));
    }

    #[test]
    fn test_parse_c_includes() {
        let parser = CParser::new();
        let source = r#"
#include <stdio.h>
#include <stdlib.h>
#include "myheader.h"
"#;
        let path = PathBuf::from("test.c");
        let result = parser.parse(&path, source).unwrap();
        
        // Check for include nodes
        let includes: Vec<_> = result.nodes.iter()
            .filter(|n| n.node_type == NodeType::Import)
            .collect();
        assert_eq!(includes.len(), 3);
        
        let include_names: Vec<_> = includes.iter().map(|i| i.name.as_str()).collect();
        assert!(include_names.contains(&"stdio.h"));
        assert!(include_names.contains(&"stdlib.h"));
        assert!(include_names.contains(&"myheader.h"));
    }
}