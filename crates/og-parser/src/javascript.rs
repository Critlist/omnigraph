use og_types::{
    AstNode, Language, NodeType, ParsedFile, Relationship, RelationshipType,
    EngineResult, EngineError, FileMetrics,
};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tree_sitter::{Node, Parser as TSParser, TreeCursor};
use tracing::{debug, trace};
use crate::import_resolver::ImportResolver;

pub struct JavaScriptParser {
    parser: Mutex<TSParser>,
    base_path: PathBuf,
}

impl JavaScriptParser {
    pub fn new() -> Self {
        Self::with_base_path(PathBuf::from("."))
    }
    
    pub fn with_base_path(base_path: PathBuf) -> Self {
        let mut parser = TSParser::new();
        parser
            .set_language(tree_sitter_javascript::language())
            .expect("Error loading JavaScript grammar");
        Self {
            parser: Mutex::new(parser),
            base_path,
        }
    }

    fn extract_nodes(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &str,
    ) -> (Vec<AstNode>, Vec<Relationship>) {
        let mut nodes = Vec::new();
        let mut relationships = Vec::new();
        let mut node_counter = 0;

        // Create file node
        let file_id = format!("file_{}", node_counter);
        node_counter += 1;
        
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
                "import_statement" | "import_declaration" => {
                    if let Some(import_node) = self.extract_import(node, source, file_path, node_counter) {
                        let import_id = import_node.id.clone();
                        
                        // Add import node
                        nodes.push(import_node);
                        
                        // Create contains relationship
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: import_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                        
                        // Extract and resolve the import path
                        self.process_import_path(node, source, &parent_id, relationships, file_path);
                    }
                }
                "export_statement" | "export_declaration" | "export_default_declaration" => {
                    if let Some(export_node) = self.extract_export(node, source, file_path, node_counter) {
                        let export_id = export_node.id.clone();
                        
                        nodes.push(export_node);
                        
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: export_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });
                    }
                }
                "function_declaration" | "arrow_function" | "function_expression" => {
                    // Skip if this function is part of a variable declaration - it will be handled as a variable
                    if let Some(parent) = node.parent() {
                        if matches!(parent.kind(), "variable_declarator") {
                            // Skip - this will be handled as a variable with function value
                            if !cursor.goto_next_sibling() {
                                break;
                            }
                            continue;
                        }
                    }
                    
                    if let Some(func_node) = self.extract_function(node, source, file_path, node_counter) {
                        let func_id = func_node.id.clone();
                        
                        nodes.push(func_node);
                        
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: func_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });

                        // Recursively walk function body
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
                        
                        // Skip normal traversal since we handled children
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                        continue;
                    }
                }
                "class_declaration" => {
                    if let Some(class_node) = self.extract_class(node, source, file_path, node_counter) {
                        let class_id = class_node.id.clone();
                        
                        nodes.push(class_node);
                        
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: class_id.clone(),
                            relationship_type: RelationshipType::Contains,
                        });

                        // Recursively walk class body
                        if cursor.goto_first_child() {
                            self.walk_tree(
                                cursor,
                                source,
                                file_path,
                                &class_id,
                                nodes,
                                relationships,
                                node_counter,
                            );
                            cursor.goto_parent();
                        }
                        
                        // Skip normal traversal
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                        continue;
                    }
                }
                "variable_declaration" | "lexical_declaration" => {
                    if let Some(var_node) = self.extract_variable(node, source, file_path, node_counter) {
                        let var_id = var_node.id.clone();
                        
                        nodes.push(var_node);
                        
                        relationships.push(Relationship {
                            source: parent_id.to_string(),
                            target: var_id,
                            relationship_type: RelationshipType::Contains,
                        });
                    }
                }
                _ => {}
            }

            // Continue tree traversal
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

            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    fn extract_import(&self, node: Node, source: &str, _file_path: &str, counter: &mut usize) -> Option<AstNode> {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        
        let id = format!("import_{}", *counter);
        *counter += 1;

        Some(AstNode {
            id,
            node_type: NodeType::Import,
            name: node.utf8_text(source.as_bytes())
                .unwrap_or("import")
                .lines()
                .next()
                .unwrap_or("import")
                .to_string(),
            start_line: start_pos.row,
            end_line: end_pos.row,
            children: Vec::new(),
        })
    }

    fn process_import_path(&self, node: Node, source: &str, parent_id: &str, relationships: &mut Vec<Relationship>, file_path: &str) {
        // Find the import source string in the import statement
        let mut cursor = node.walk();
        
        // Try to find the source string node
        let import_path = self.find_import_source(&mut cursor, source);
        
        debug!("Processing import in {}: found path {:?}", file_path, import_path);
        
        if let Some(import_path) = import_path {
            // Resolve the import path
            let resolver = ImportResolver::new(self.base_path.clone());
            let file_path = Path::new(file_path);
            
            if let Some(resolved_path) = resolver.resolve_import(&import_path, file_path) {
                relationships.push(Relationship {
                    source: parent_id.to_string(),
                    target: resolved_path,
                    relationship_type: RelationshipType::Imports,
                });
            }
        }
    }
    
    fn find_import_source(&self, cursor: &mut TreeCursor, source: &str) -> Option<String> {
        // Navigate through the import statement to find the source string
        // Import statements have structure: import_statement -> ... -> from -> string
        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                
                debug!("Looking at node kind: {} with text: {:?}", 
                      node.kind(), 
                      node.utf8_text(source.as_bytes()).ok());
                
                // Look for string node (the import path) - this is what we want
                if node.kind() == "string" || node.kind() == "string_fragment" {
                    if let Ok(text) = node.utf8_text(source.as_bytes()) {
                        // Remove quotes
                        let import_path = text.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                        debug!("Found import path: {}", import_path);
                        cursor.goto_parent();
                        return Some(import_path.to_string());
                    }
                }
                
                // Also check for source field in import statements (TypeScript/newer syntax)
                if node.kind() == "source" {
                    if cursor.goto_first_child() {
                        let source_node = cursor.node();
                        if source_node.kind() == "string" || source_node.kind() == "string_fragment" {
                            if let Ok(text) = source_node.utf8_text(source.as_bytes()) {
                                let import_path = text.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                                cursor.goto_parent();
                                cursor.goto_parent();
                                debug!("Found import path from source: {}", import_path);
                                return Some(import_path.to_string());
                            }
                        }
                        cursor.goto_parent();
                    }
                }
                
                // Check if this is the 'from' keyword - the string usually follows
                if node.kind() == "from" {
                    // Look for the next sibling which should be the string
                    if cursor.goto_next_sibling() {
                        let next_node = cursor.node();
                        if next_node.kind() == "string" || next_node.kind() == "string_fragment" {
                            if let Ok(text) = next_node.utf8_text(source.as_bytes()) {
                                let import_path = text.trim_matches(|c| c == '"' || c == '\'' || c == '`');
                                cursor.goto_parent();
                                debug!("Found import path after from: {}", import_path);
                                return Some(import_path.to_string());
                            }
                        }
                    }
                }
                
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
        debug!("No import path found for node");
        None
    }

    fn extract_export(&self, node: Node, source: &str, _file_path: &str, counter: &mut usize) -> Option<AstNode> {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        
        let id = format!("export_{}", *counter);
        *counter += 1;

        Some(AstNode {
            id,
            node_type: NodeType::Export,
            name: node.utf8_text(source.as_bytes())
                .unwrap_or("export")
                .lines()
                .next()
                .unwrap_or("export")
                .to_string(),
            start_line: start_pos.row,
            end_line: end_pos.row,
            children: Vec::new(),
        })
    }

    fn extract_function(&self, node: Node, source: &str, _file_path: &str, counter: &mut usize) -> Option<AstNode> {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        
        // Try to get function name
        let mut name = node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .map(|s| s.to_string());
        
        // If no name, try to infer from context
        if name.is_none() {
            name = self.infer_function_name(node, source);
        }
        
        let name = name.unwrap_or_else(|| {
            // Last resort: try to get first line for context
            if let Ok(text) = node.utf8_text(source.as_bytes()) {
                let first_line = text.lines().next().unwrap_or("anonymous");
                if first_line.len() > 50 {
                    format!("anonymous_line_{}", start_pos.row + 1)
                } else {
                    format!("anonymous_line_{}", start_pos.row + 1)
                }
            } else {
                "anonymous".to_string()
            }
        });
        
        let id = format!("function_{}", *counter);
        *counter += 1;

        Some(AstNode {
            id,
            node_type: NodeType::Function,
            name,
            start_line: start_pos.row,
            end_line: end_pos.row,
            children: Vec::new(),
        })
    }

    fn extract_class(&self, node: Node, source: &str, _file_path: &str, counter: &mut usize) -> Option<AstNode> {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        
        // Get class name
        let name = node.child_by_field_name("name")
            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
            .unwrap_or("AnonymousClass")
            .to_string();
        
        let id = format!("class_{}", *counter);
        *counter += 1;

        Some(AstNode {
            id,
            node_type: NodeType::Class,
            name,
            start_line: start_pos.row,
            end_line: end_pos.row,
            children: Vec::new(),
        })
    }

    fn infer_function_name(&self, node: Node, source: &str) -> Option<String> {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                // Variable declarations: const myFunc = () => {}
                "variable_declarator" => {
                    return parent.child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                }
                // Assignments: obj.method = function() {}
                "assignment_expression" => {
                    if let Some(left) = parent.child_by_field_name("left") {
                        // Handle member expressions like obj.method or obj['method']
                        if left.kind() == "member_expression" {
                            if let Some(property) = left.child_by_field_name("property") {
                                if let Ok(prop_name) = property.utf8_text(source.as_bytes()) {
                                    // Also try to get the object name for better context
                                    if let Some(object) = left.child_by_field_name("object") {
                                        if let Ok(obj_name) = object.utf8_text(source.as_bytes()) {
                                            return Some(format!("{}.{}", obj_name, prop_name.trim_matches('"').trim_matches('\'')));
                                        }
                                    }
                                    return Some(prop_name.trim_matches('"').trim_matches('\'').to_string());
                                }
                            }
                        } else {
                            return left.utf8_text(source.as_bytes()).ok()
                                .map(|s| s.to_string());
                        }
                    }
                }
                // Object property: { method: function() {} } or { method() {} }
                "pair" | "method_definition" | "property" => {
                    // Get the key/name of the property
                    if let Some(key) = parent.child_by_field_name("key") {
                        return key.utf8_text(source.as_bytes()).ok()
                            .map(|s| s.trim_matches('"').trim_matches('\'').trim_matches('`').to_string());
                    }
                    // For shorthand method syntax
                    if let Some(name) = parent.child_by_field_name("name") {
                        return name.utf8_text(source.as_bytes()).ok()
                            .map(|s| s.to_string());
                    }
                }
                // Array methods: array.map(() => {})
                "call_expression" => {
                    if let Some(func) = parent.child_by_field_name("function") {
                        if let Ok(text) = func.utf8_text(source.as_bytes()) {
                            // Extract method name like "map", "filter", etc.
                            if let Some(method_name) = text.split('.').last() {
                                // Check if this is the first argument (common for callbacks)
                                let args = parent.child_by_field_name("arguments");
                                if let Some(args_node) = args {
                                    let mut cursor = args_node.walk();
                                    let children: Vec<Node> = args_node.children(&mut cursor).collect();
                                    // Check if our function is the first argument
                                    if let Some(first_arg) = children.iter().find(|n| {
                                        matches!(n.kind(), "arrow_function" | "function_expression" | "function")
                                    }) {
                                        if first_arg.id() == node.id() {
                                            return Some(format!("{}_callback", method_name));
                                        }
                                    }
                                }
                                return Some(format!("{}_callback", method_name));
                            }
                        }
                    }
                    // Handle cases like: addEventListener('click', () => {})
                    if let Some(args) = parent.child_by_field_name("arguments") {
                        let mut cursor = args.walk();
                        let children: Vec<Node> = args.children(&mut cursor).collect();
                        // Check for string literal as first argument (event name, etc.)
                        if let Some(first_arg) = children.iter().find(|n| n.kind() == "string") {
                            if let Ok(event_name) = first_arg.utf8_text(source.as_bytes()) {
                                let clean_name = event_name.trim_matches('"').trim_matches('\'');
                                return Some(format!("{}_handler", clean_name));
                            }
                        }
                    }
                }
                // Export statements: export default () => {}
                "export_statement" => {
                    if parent.child_by_field_name("default").is_some() {
                        return Some("default_export".to_string());
                    }
                    return Some("exported_function".to_string());
                }
                // Return statement: return () => {}
                "return_statement" => {
                    // Check if the parent function has a name
                    if let Some(grandparent) = parent.parent() {
                        if matches!(grandparent.kind(), "function_declaration" | "method_definition") {
                            if let Some(name) = grandparent.child_by_field_name("name") {
                                if let Ok(parent_name) = name.utf8_text(source.as_bytes()) {
                                    return Some(format!("{}_returned", parent_name));
                                }
                            }
                        }
                    }
                }
                // JSX props: <Component onClick={() => {}} />
                "jsx_attribute" => {
                    if let Some(name_node) = parent.child_by_field_name("name") {
                        if let Ok(prop_name) = name_node.utf8_text(source.as_bytes()) {
                            return Some(format!("{}_handler", prop_name));
                        }
                    }
                }
                _ => {}
            }
            
            // Try grandparent for nested structures
            if let Some(grandparent) = parent.parent() {
                match grandparent.kind() {
                    "variable_declarator" => {
                        return grandparent.child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string());
                    }
                    "pair" | "property" => {
                        if let Some(key) = grandparent.child_by_field_name("key") {
                            return key.utf8_text(source.as_bytes()).ok()
                                .map(|s| s.trim_matches('"').trim_matches('\'').to_string());
                        }
                    }
                    "assignment_expression" => {
                        if let Some(left) = grandparent.child_by_field_name("left") {
                            if left.kind() == "member_expression" {
                                if let Some(property) = left.child_by_field_name("property") {
                                    return property.utf8_text(source.as_bytes()).ok()
                                        .map(|s| s.trim_matches('"').trim_matches('\'').to_string());
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Last resort: try to infer from context or position
        if let Some(parent) = node.parent() {
            // Check if it's an IIFE (Immediately Invoked Function Expression)
            if parent.kind() == "call_expression" {
                if let Some(func_node) = parent.child_by_field_name("function") {
                    if func_node.id() == node.id() {
                        return Some("IIFE".to_string());
                    }
                }
            }
        }
        
        None
    }

    fn extract_variable(&self, node: Node, source: &str, _file_path: &str, counter: &mut usize) -> Option<AstNode> {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        
        // Find the variable declarator
        let declarator = node.children(&mut node.walk())
            .find(|n| n.kind() == "variable_declarator");
        
        if let Some(declarator_node) = declarator {
            // Get the variable name
            let name = declarator_node.child_by_field_name("name")
                .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                .unwrap_or("variable")
                .to_string();
            
            // Check if the value is a function
            let is_function = declarator_node.child_by_field_name("value")
                .map(|value_node| {
                    matches!(value_node.kind(), "arrow_function" | "function_expression" | "function")
                })
                .unwrap_or(false);
            
            let node_type = if is_function {
                NodeType::Function
            } else {
                NodeType::Variable
            };
            
            let id = if is_function {
                format!("function_{}", *counter)
            } else {
                format!("variable_{}", *counter)
            };
            *counter += 1;

            return Some(AstNode {
                id,
                node_type,
                name,
                start_line: start_pos.row,
                end_line: end_pos.row,
                children: Vec::new(),
            });
        }
        
        None
    }
}

impl crate::Parser for JavaScriptParser {
    fn supported_extensions(&self) -> &[&str] {
        &[".js", ".mjs", ".cjs", ".jsx"]
    }

    fn language(&self) -> Language {
        Language::JavaScript
    }

    fn parse(&self, path: &Path, content: &str) -> EngineResult<ParsedFile> {
        let start_time = std::time::Instant::now();
        
        debug!("Parsing JavaScript file: {}", path.display());
        
        let mut parser = self.parser.lock().unwrap();
        let tree = parser.parse(content, None).ok_or_else(|| {
            EngineError::ParseError {
                file: path.display().to_string(),
                message: "Failed to parse JavaScript file".to_string(),
            }
        })?;

        let mut cursor = tree.walk();
        let (nodes, relationships) = self.extract_nodes(&mut cursor, content, &path.display().to_string());

        let parse_time_ms = start_time.elapsed().as_millis() as u64;
        
        debug!(
            "Parsed {} nodes and {} relationships in {}ms",
            nodes.len(),
            relationships.len(),
            parse_time_ms
        );

        let metrics = FileMetrics {
            lines_of_code: content.lines().count(),
            complexity: 0, // TODO: Calculate complexity
            functions: nodes.iter().filter(|n| matches!(n.node_type, NodeType::Function)).count(),
            classes: nodes.iter().filter(|n| matches!(n.node_type, NodeType::Class)).count(),
            imports: nodes.iter().filter(|n| matches!(n.node_type, NodeType::Import)).count(),
            exports: nodes.iter().filter(|n| matches!(n.node_type, NodeType::Export)).count(),
        };

        Ok(ParsedFile {
            path: path.to_path_buf(),
            language: Language::JavaScript,
            nodes,
            relationships,
            metrics,
        })
    }
}