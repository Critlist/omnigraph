use og_types::{
    AstNode, Language, NodeType, ParseError, ParsedFile, Relationship, RelationshipType,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tree_sitter::{Node, Parser as TSParser, TreeCursor};

use crate::import_resolver::ImportResolver;
use crate::parser_trait::Parser;
use og_types::EngineResult;

pub struct TypeScriptParser {
    parser: Mutex<TSParser>,
    base_path: PathBuf,
}

impl TypeScriptParser {
    pub fn new() -> Self {
        Self::with_base_path(PathBuf::from("."))
    }
    
    pub fn with_base_path(base_path: PathBuf) -> Self {
        let mut parser = TSParser::new();
        parser
            .set_language(tree_sitter_typescript::language_typescript())
            .expect("Error loading TypeScript grammar");
        Self {
            parser: Mutex::new(parser),
            base_path,
        }
    }

    fn extract_nodes(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
    ) -> EngineResult<ParsedFile> {
        let mut nodes = Vec::new();
        let mut relationships = Vec::new();
        let mut import_map = HashMap::new();
        let mut export_map = HashMap::new();
        let mut complexity = 1;

        self.walk_tree(
            cursor,
            source,
            file_path,
            &mut nodes,
            &mut relationships,
            &mut import_map,
            &mut export_map,
            &mut complexity,
            None,
        )?;

        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            language: Language::TypeScript,
            nodes,
            relationships,
            metrics: Default::default(),
        })
    }

    fn walk_tree(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        export_map: &mut HashMap<String, String>,
        complexity: &mut usize,
        parent_id: Option<String>,
    ) -> EngineResult<()> {
        let node = cursor.node();
        let node_type = node.kind();

        match node_type {
            "program" => {
                // Root module node
                let file_id = format!("file:{}", file_path.display());
                let file_name = file_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                nodes.push(AstNode {
                    id: file_id.clone(),
                    name: file_name.to_string(),
                    node_type: NodeType::File,
                    start_line: 0,
                    end_line: source.lines().count(),
                    children: vec![],
                });

                if cursor.goto_first_child() {
                    loop {
                        self.walk_tree(
                            cursor,
                            source,
                            file_path,
                            nodes,
                            relationships,
                            import_map,
                            export_map,
                            complexity,
                            Some(file_id.clone()),
                        )?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
            "import_statement" => {
                self.process_import(node, source, &parent_id, relationships, import_map, file_path)?;
            }
            "export_statement" => {
                self.process_export(
                    cursor,
                    source,
                    file_path,
                    nodes,
                    relationships,
                    import_map,
                    export_map,
                    complexity,
                    &parent_id,
                )?;
            }
            "class_declaration" => {
                self.process_class(
                    cursor,
                    source,
                    file_path,
                    nodes,
                    relationships,
                    import_map,
                    export_map,
                    complexity,
                    &parent_id,
                )?;
            }
            "interface_declaration" => {
                self.process_interface(
                    cursor,
                    source,
                    file_path,
                    nodes,
                    relationships,
                    import_map,
                    export_map,
                    complexity,
                    &parent_id,
                )?;
            }
            "type_alias_declaration" => {
                self.process_type_alias(node, source, nodes, &parent_id, relationships)?;
            }
            "enum_declaration" => {
                self.process_enum(node, source, nodes, &parent_id, relationships)?;
            }
            "function_declaration" | "function_expression" | "arrow_function" => {
                self.process_function(
                    cursor,
                    source,
                    file_path,
                    nodes,
                    relationships,
                    import_map,
                    export_map,
                    complexity,
                    &parent_id,
                )?;
            }
            "variable_declaration" => {
                self.process_variable(
                    cursor,
                    source,
                    file_path,
                    nodes,
                    relationships,
                    import_map,
                    export_map,
                    complexity,
                    &parent_id,
                )?;
            }
            "if_statement" | "while_statement" | "for_statement" | "do_statement" => {
                *complexity += 1;
                if cursor.goto_first_child() {
                    loop {
                        self.walk_tree(
                            cursor,
                            source,
                            file_path,
                            nodes,
                            relationships,
                            import_map,
                            export_map,
                            complexity,
                            parent_id.clone(),
                        )?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
            _ => {
                // Recursively process children
                if cursor.goto_first_child() {
                    loop {
                        self.walk_tree(
                            cursor,
                            source,
                            file_path,
                            nodes,
                            relationships,
                            import_map,
                            export_map,
                            complexity,
                            parent_id.clone(),
                        )?;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
        }

        Ok(())
    }

    fn process_import(
        &self,
        node: Node,
        source: &str,
        parent_id: &Option<String>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        file_path: &Path,
    ) -> EngineResult<()> {
        if let Some(parent) = parent_id {
            // Find the import source
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    if cursor.node().kind() == "string" {
                        let import_path = cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?;
                        let import_path = import_path.trim_matches(|c| c == '"' || c == '\'');
                        
                        // Resolve the import path
                        let resolver = ImportResolver::new(self.base_path.clone());
                        if let Some(resolved_path) = resolver.resolve_import(import_path, file_path) {
                            relationships.push(Relationship {
                                source: parent.clone(),
                                target: resolved_path.clone(),
                                relationship_type: RelationshipType::Imports,
                            });
                            
                            // Store import mapping for later reference resolution
                            import_map.insert(import_path.to_string(), resolved_path);
                        } else {
                            // If we can't resolve it, store the original path (for external imports)
                            import_map.insert(import_path.to_string(), format!("external:{}", import_path));
                        }
                        break;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    fn process_export(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        export_map: &mut HashMap<String, String>,
        complexity: &mut usize,
        parent_id: &Option<String>,
    ) -> EngineResult<()> {
        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                match node.kind() {
                    "class_declaration" => {
                        self.process_class(
                            cursor,
                            source,
                            file_path,
                            nodes,
                            relationships,
                            import_map,
                            export_map,
                            complexity,
                            parent_id,
                        )?;
                    }
                    "function_declaration" => {
                        self.process_function(
                            cursor,
                            source,
                            file_path,
                            nodes,
                            relationships,
                            import_map,
                            export_map,
                            complexity,
                            parent_id,
                        )?;
                    }
                    "interface_declaration" => {
                        self.process_interface(
                            cursor,
                            source,
                            file_path,
                            nodes,
                            relationships,
                            import_map,
                            export_map,
                            complexity,
                            parent_id,
                        )?;
                    }
                    "type_alias_declaration" => {
                        self.process_type_alias(node, source, nodes, parent_id, relationships)?;
                    }
                    _ => {}
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
        Ok(())
    }

    fn process_class(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        export_map: &mut HashMap<String, String>,
        complexity: &mut usize,
        parent_id: &Option<String>,
    ) -> EngineResult<()> {
        let node = cursor.node();
        let mut class_name = "AnonymousClass".to_string();
        let mut extends_class = None;
        let mut implements_interfaces = Vec::new();

        // Extract class name and inheritance info
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "type_identifier" | "identifier" => {
                        if class_name == "AnonymousClass" {
                            class_name = child.utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string();
                        }
                    }
                    "extends_clause" => {
                        if let Some(extends_node) = self.find_child_by_type(&child, "identifier") {
                            extends_class = Some(extends_node.utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string());
                        }
                    }
                    "implements_clause" => {
                        let mut impl_cursor = child.walk();
                        if impl_cursor.goto_first_child() {
                            loop {
                                if impl_cursor.node().kind() == "type_identifier" {
                                    let interface_name = impl_cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?;
                                    implements_interfaces.push(interface_name.to_string());
                                }
                                if !impl_cursor.goto_next_sibling() {
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }

        let class_id = format!("class:{}:{}", file_path.display(), class_name);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;

        nodes.push(AstNode {
            id: class_id.clone(),
            name: class_name.clone(),
            node_type: NodeType::Class,
            start_line: line_start,
            end_line: line_end,
            children: vec![],
        });

        // Add relationships
        if let Some(parent) = parent_id {
            relationships.push(Relationship {
                source: parent.clone(),
                target: class_id.clone(),
                relationship_type: RelationshipType::Contains,
            });
        }

        if let Some(base_class) = extends_class {
            relationships.push(Relationship {
                source: class_id.clone(),
                target: format!("class:{}:{}", file_path.display(), base_class),
                relationship_type: RelationshipType::Extends,
            });
        }

        for interface in implements_interfaces {
            relationships.push(Relationship {
                source: class_id.clone(),
                target: format!("interface:{}:{}", file_path.display(), interface),
                relationship_type: RelationshipType::Implements,
            });
        }

        // Process class body
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "class_body" {
                    let mut body_cursor = cursor.node().walk();
                    if body_cursor.goto_first_child() {
                        loop {
                            self.walk_tree(
                                &mut body_cursor,
                                source,
                                file_path,
                                nodes,
                                relationships,
                                import_map,
                                export_map,
                                complexity,
                                Some(class_id.clone()),
                            )?;
                            if !body_cursor.goto_next_sibling() {
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
            cursor.goto_parent();
        }

        Ok(())
    }

    fn process_interface(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        export_map: &mut HashMap<String, String>,
        complexity: &mut usize,
        parent_id: &Option<String>,
    ) -> EngineResult<()> {
        let node = cursor.node();
        let mut interface_name = "AnonymousInterface".to_string();
        let mut extends_interfaces = Vec::new();

        // Extract interface name and extends info
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "type_identifier" | "identifier" => {
                        if interface_name == "AnonymousInterface" {
                            interface_name = child.utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string();
                        }
                    }
                    "extends_type_clause" => {
                        let mut ext_cursor = child.walk();
                        if ext_cursor.goto_first_child() {
                            loop {
                                if ext_cursor.node().kind() == "type_identifier" {
                                    let parent_interface = ext_cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?;
                                    extends_interfaces.push(parent_interface.to_string());
                                }
                                if !ext_cursor.goto_next_sibling() {
                                    break;
                                }
                            }
                        }
                    }
                    _ => {}
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }

        let interface_id = format!("interface:{}:{}", file_path.display(), interface_name);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;

        nodes.push(AstNode {
            id: interface_id.clone(),
            name: interface_name.clone(),
            node_type: NodeType::Interface,
            start_line: line_start,
            end_line: line_end,
            children: vec![],
        });

        // Add relationships
        if let Some(parent) = parent_id {
            relationships.push(Relationship {
                source: parent.clone(),
                target: interface_id.clone(),
                relationship_type: RelationshipType::Contains,
            });
        }

        for parent_interface in extends_interfaces {
            relationships.push(Relationship {
                source: interface_id.clone(),
                target: format!("interface:{}:{}", file_path.display(), parent_interface),
                relationship_type: RelationshipType::Extends,
            });
        }

        Ok(())
    }

    fn process_type_alias(
        &self,
        node: Node,
        source: &str,
        nodes: &mut Vec<AstNode>,
        parent_id: &Option<String>,
        relationships: &mut Vec<Relationship>,
    ) -> EngineResult<()> {
        let mut type_name = "AnonymousType".to_string();
        
        // Extract type alias name
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "type_identifier" || cursor.node().kind() == "identifier" {
                    type_name = cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string();
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        let type_id = format!("type:{}", type_name);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;

        nodes.push(AstNode {
            id: type_id.clone(),
            name: type_name,
            node_type: NodeType::TypeAlias,
            start_line: line_start,
            end_line: line_end,
            children: vec![],
        });

        if let Some(parent) = parent_id {
            relationships.push(Relationship {
                source: parent.clone(),
                target: type_id,
                relationship_type: RelationshipType::Contains,
            });
        }

        Ok(())
    }

    fn process_enum(
        &self,
        node: Node,
        source: &str,
        nodes: &mut Vec<AstNode>,
        parent_id: &Option<String>,
        relationships: &mut Vec<Relationship>,
    ) -> EngineResult<()> {
        let mut enum_name = "AnonymousEnum".to_string();
        
        // Extract enum name
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "identifier" {
                    enum_name = cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string();
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }

        let enum_id = format!("enum:{}", enum_name);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;

        nodes.push(AstNode {
            id: enum_id.clone(),
            name: enum_name,
            node_type: NodeType::Enum,
            start_line: line_start,
            end_line: line_end,
            children: vec![],
        });

        if let Some(parent) = parent_id {
            relationships.push(Relationship {
                source: parent.clone(),
                target: enum_id,
                relationship_type: RelationshipType::Contains,
            });
        }

        Ok(())
    }

    fn process_function(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        export_map: &mut HashMap<String, String>,
        complexity: &mut usize,
        parent_id: &Option<String>,
    ) -> EngineResult<()> {
        let node = cursor.node();
        let mut func_name = "AnonymousFunction".to_string();

        // First try to extract function name directly
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "identifier" {
                    func_name = cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string();
                    break;
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
        
        // If still anonymous, try to infer from context
        if func_name == "AnonymousFunction" {
            if let Some(inferred_name) = self.infer_function_name(node, source) {
                func_name = inferred_name;
            }
        }

        let func_id = format!("function:{}:{}", file_path.display(), func_name);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;

        nodes.push(AstNode {
            id: func_id.clone(),
            name: func_name.clone(),
            node_type: NodeType::Function,
            start_line: line_start,
            end_line: line_end,
            children: vec![],
        });

        if let Some(parent) = parent_id {
            relationships.push(Relationship {
                source: parent.clone(),
                target: func_id.clone(),
                relationship_type: RelationshipType::Contains,
            });
        }

        // Process function body for complexity
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "statement_block" {
                    let mut body_cursor = cursor.node().walk();
                    if body_cursor.goto_first_child() {
                        loop {
                            self.walk_tree(
                                &mut body_cursor,
                                source,
                                file_path,
                                nodes,
                                relationships,
                                import_map,
                                export_map,
                                complexity,
                                Some(func_id.clone()),
                            )?;
                            if !body_cursor.goto_next_sibling() {
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
            cursor.goto_parent();
        }

        Ok(())
    }

    fn process_variable(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        export_map: &mut HashMap<String, String>,
        complexity: &mut usize,
        parent_id: &Option<String>,
    ) -> EngineResult<()> {
        // Process variable declarators
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "variable_declarator" {
                    let declarator_node = cursor.node();
                    
                    // Get variable name first
                    let var_name = declarator_node.child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                    
                    let mut var_cursor = declarator_node.walk();
                    if var_cursor.goto_first_child() {
                        loop {
                            let child = var_cursor.node();
                            // Check if the variable is initialized with a function or class
                            if child.kind() == "arrow_function" || child.kind() == "function_expression" {
                                // The function processor will try to infer the name from context
                                // but we can also pass the variable name if we have it
                                self.process_function(
                                    &mut var_cursor,
                                    source,
                                    file_path,
                                    nodes,
                                    relationships,
                                    import_map,
                                    export_map,
                                    complexity,
                                    parent_id,
                                )?;
                            }
                            if !var_cursor.goto_next_sibling() {
                                break;
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
        Ok(())
    }

    fn find_child_by_type<'a>(&self, node: &'a Node, type_name: &str) -> Option<Node<'a>> {
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == type_name {
                    return Some(cursor.node());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }
    
    fn infer_function_name(&self, node: Node, source: &str) -> Option<String> {
        if let Some(parent) = node.parent() {
            match parent.kind() {
                // Variable declarations: const myFunc = () => {}
                "variable_declarator" | "lexical_binding" => {
                    return parent.child_by_field_name("name")
                        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                        .map(|s| s.to_string());
                }
                // Assignments: obj.method = function() {}
                "assignment_expression" => {
                    if let Some(left) = parent.child_by_field_name("left") {
                        // Handle member expressions
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
                "property_signature" | "method_signature" | "pair" | "method_definition" | "property" => {
                    // Get the key/name of the property
                    if let Some(key) = parent.child_by_field_name("key") {
                        return key.utf8_text(source.as_bytes()).ok()
                            .map(|s| s.trim_matches('"').trim_matches('\'').trim_matches('`').to_string());
                    }
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
                                // Check for event handlers
                                if method_name == "addEventListener" || method_name == "on" {
                                    // Try to get the event name from the first argument
                                    if let Some(args) = parent.child_by_field_name("arguments") {
                                        let mut cursor = args.walk();
                                        if cursor.goto_first_child() {
                                            loop {
                                                if cursor.node().kind() == "string" {
                                                    if let Ok(event_name) = cursor.node().utf8_text(source.as_bytes()) {
                                                        let clean_name = event_name.trim_matches('"').trim_matches('\'');
                                                        return Some(format!("{}_handler", clean_name));
                                                    }
                                                }
                                                if !cursor.goto_next_sibling() {
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                return Some(format!("{}_callback", method_name));
                            }
                        }
                    }
                }
                // Export statements
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
                        if matches!(grandparent.kind(), "function_declaration" | "method_definition" | "method_signature") {
                            if let Some(name) = grandparent.child_by_field_name("name") {
                                if let Ok(parent_name) = name.utf8_text(source.as_bytes()) {
                                    return Some(format!("{}_returned", parent_name));
                                }
                            }
                        }
                    }
                }
                // JSX/TSX props
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
                    "variable_declarator" | "lexical_binding" => {
                        return grandparent.child_by_field_name("name")
                            .and_then(|n| n.utf8_text(source.as_bytes()).ok())
                            .map(|s| s.to_string());
                    }
                    "pair" | "property" | "property_signature" => {
                        if let Some(key) = grandparent.child_by_field_name("key") {
                            return key.utf8_text(source.as_bytes()).ok()
                                .map(|s| s.trim_matches('"').trim_matches('\'').to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        
        // Check for IIFE
        if let Some(parent) = node.parent() {
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
}

impl Parser for TypeScriptParser {
    fn supported_extensions(&self) -> &[&str] {
        &[".ts", ".tsx"]
    }

    fn language(&self) -> Language {
        Language::TypeScript
    }

    fn parse(&self, path: &Path, content: &str) -> EngineResult<ParsedFile> {
        let tree = self
            .parser
            .lock()
            .unwrap()
            .parse(content, None)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse TypeScript file".to_string()))?;

        let mut cursor = tree.root_node().walk();
        self.extract_nodes(&mut cursor, content, path)
    }
}