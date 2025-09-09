use og_types::{
    AstNode, Language, NodeType, ParseError, ParsedFile, Relationship, RelationshipType,
};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Mutex;
use tree_sitter::{Node, Parser as TSParser, TreeCursor};

use crate::parser_trait::Parser;
use og_types::EngineResult;

pub struct PythonParser {
    parser: Mutex<TSParser>,
}

impl PythonParser {
    pub fn new() -> Self {
        let mut parser = TSParser::new();
        parser
            .set_language(tree_sitter_python::language())
            .expect("Error loading Python grammar");
        Self { parser: Mutex::new(parser) }
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
        let mut complexity = 1;

        self.walk_tree(
            cursor,
            source,
            file_path,
            &mut nodes,
            &mut relationships,
            &mut import_map,
            &mut complexity,
            None,
        )?;

        Ok(ParsedFile {
            path: file_path.to_path_buf(),
            language: Language::Python,
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
        complexity: &mut usize,
        parent_id: Option<String>,
    ) -> EngineResult<()> {
        let node = cursor.node();
        let node_type = node.kind();

        match node_type {
            "module" => {
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
            "import_statement" | "import_from_statement" => {
                self.process_import(node, source, &parent_id, relationships, import_map)?;
            }
            "class_definition" => {
                self.process_class(
                    cursor,
                    source,
                    file_path,
                    nodes,
                    relationships,
                    import_map,
                    complexity,
                    &parent_id,
                )?;
            }
            "function_definition" => {
                self.process_function(
                    cursor,
                    source,
                    file_path,
                    nodes,
                    relationships,
                    import_map,
                    complexity,
                    &parent_id,
                    false,
                )?;
            }
            "decorated_definition" => {
                // Handle decorated functions and classes
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        match child.kind() {
                            "function_definition" => {
                                self.process_function(
                                    cursor,
                                    source,
                                    file_path,
                                    nodes,
                                    relationships,
                                    import_map,
                                    complexity,
                                    &parent_id,
                                    true,
                                )?;
                            }
                            "class_definition" => {
                                self.process_class(
                                    cursor,
                                    source,
                                    file_path,
                                    nodes,
                                    relationships,
                                    import_map,
                                    complexity,
                                    &parent_id,
                                )?;
                            }
                            _ => {}
                        }
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                }
            }
            "if_statement" | "while_statement" | "for_statement" => {
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
            "try_statement" => {
                // Each except clause adds to complexity
                *complexity += 1;
                if cursor.goto_first_child() {
                    loop {
                        let child = cursor.node();
                        if child.kind() == "except_clause" {
                            *complexity += 1;
                        }
                        self.walk_tree(
                            cursor,
                            source,
                            file_path,
                            nodes,
                            relationships,
                            import_map,
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
    ) -> EngineResult<()> {
        if let Some(parent) = parent_id {
            let mut cursor = node.walk();
            
            // Handle different import types
            match node.kind() {
                "import_statement" => {
                    // import module1, module2
                    if cursor.goto_first_child() {
                        loop {
                            if cursor.node().kind() == "dotted_name" || cursor.node().kind() == "aliased_import" {
                                let module_name = if cursor.node().kind() == "aliased_import" {
                                    // Get the original module name, not the alias
                                    let mut alias_cursor = cursor.node().walk();
                                    if alias_cursor.goto_first_child() {
                                        if alias_cursor.node().kind() == "dotted_name" {
                                            alias_cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?
                                        } else {
                                            cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?
                                        }
                                    } else {
                                        cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?
                                    }
                                } else {
                                    cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?
                                };
                                
                                let module_path = module_name.replace('.', "/");
                                relationships.push(Relationship {
                                    source: parent.clone(),
                                    target: format!("module:{}", module_path),
                                    relationship_type: RelationshipType::Imports,
                                });
                                import_map.insert(module_name.to_string(), parent.clone());
                            }
                            if !cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }
                }
                "import_from_statement" => {
                    // from module import name1, name2
                    let mut module_name = None;
                    let mut imported_names = Vec::new();
                    
                    if cursor.goto_first_child() {
                        loop {
                            match cursor.node().kind() {
                                "dotted_name" | "relative_import" => {
                                    if module_name.is_none() {
                                        module_name = Some(cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string());
                                    }
                                }
                                "import" => {
                                    // Skip the "import" keyword
                                }
                                "identifier" | "aliased_import" => {
                                    imported_names.push(cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string());
                                }
                                _ => {}
                            }
                            if !cursor.goto_next_sibling() {
                                break;
                            }
                        }
                    }
                    
                    if let Some(module) = module_name {
                        let module_path = module.replace('.', "/");
                        relationships.push(Relationship {
                            source: parent.clone(),
                            target: format!("module:{}", module_path),
                            relationship_type: RelationshipType::Imports,
                        });
                        import_map.insert(module, parent.clone());
                    }
                }
                _ => {}
            }
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
        complexity: &mut usize,
        parent_id: &Option<String>,
    ) -> EngineResult<()> {
        let node = cursor.node();
        let mut class_name = "AnonymousClass".to_string();
        let mut base_classes = Vec::new();

        // Extract class name and base classes
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "identifier" => {
                        if class_name == "AnonymousClass" {
                            class_name = child.utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string();
                        }
                    }
                    "argument_list" => {
                        // Base classes in parentheses
                        let mut arg_cursor = child.walk();
                        if arg_cursor.goto_first_child() {
                            loop {
                                if arg_cursor.node().kind() == "identifier" || arg_cursor.node().kind() == "attribute" {
                                    let base_class = arg_cursor.node().utf8_text(source.as_bytes()).map_err(ParseError::from)?;
                                    base_classes.push(base_class.to_string());
                                }
                                if !arg_cursor.goto_next_sibling() {
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

        for base_class in base_classes {
            relationships.push(Relationship {
                source: class_id.clone(),
                target: format!("class:{}:{}", file_path.display(), base_class),
                relationship_type: RelationshipType::Extends,
            });
        }

        // Process class body
        if cursor.goto_first_child() {
            loop {
                if cursor.node().kind() == "block" {
                    let mut block_cursor = cursor.node().walk();
                    if block_cursor.goto_first_child() {
                        loop {
                            self.walk_tree(
                                &mut block_cursor,
                                source,
                                file_path,
                                nodes,
                                relationships,
                                import_map,
                                complexity,
                                Some(class_id.clone()),
                            )?;
                            if !block_cursor.goto_next_sibling() {
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

    fn process_function(
        &self,
        cursor: &mut TreeCursor,
        source: &str,
        file_path: &Path,
        nodes: &mut Vec<AstNode>,
        relationships: &mut Vec<Relationship>,
        import_map: &mut HashMap<String, String>,
        complexity: &mut usize,
        parent_id: &Option<String>,
        is_decorated: bool,
    ) -> EngineResult<()> {
        let node = cursor.node();
        let mut func_name = "AnonymousFunction".to_string();
        let mut is_async = false;

        // Extract function name
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "identifier" => {
                        if func_name == "AnonymousFunction" {
                            func_name = child.utf8_text(source.as_bytes()).map_err(ParseError::from)?.to_string();
                        }
                    }
                    "async" => {
                        is_async = true;
                    }
                    _ => {}
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }

        // Determine node type based on name and parent
        let node_type = if func_name == "__init__" || parent_id.as_ref().map_or(false, |id| id.starts_with("class:")) {
            NodeType::Method
        } else {
            NodeType::Function
        };

        let func_id = format!("function:{}:{}", file_path.display(), func_name);
        let line_start = node.start_position().row + 1;
        let line_end = node.end_position().row + 1;

        nodes.push(AstNode {
            id: func_id.clone(),
            name: func_name.clone(),
            node_type,
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
                if cursor.node().kind() == "block" {
                    let mut block_cursor = cursor.node().walk();
                    if block_cursor.goto_first_child() {
                        loop {
                            self.walk_tree(
                                &mut block_cursor,
                                source,
                                file_path,
                                nodes,
                                relationships,
                                import_map,
                                complexity,
                                Some(func_id.clone()),
                            )?;
                            if !block_cursor.goto_next_sibling() {
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
}

impl Parser for PythonParser {
    fn supported_extensions(&self) -> &[&str] {
        &[".py", ".pyi"]
    }

    fn language(&self) -> Language {
        Language::Python
    }

    fn parse(&self, path: &Path, content: &str) -> EngineResult<ParsedFile> {
        let tree = self
            .parser
            .lock()
            .unwrap()
            .parse(content, None)
            .ok_or_else(|| ParseError::ParseFailed("Failed to parse Python file".to_string()))?;

        let mut cursor = tree.root_node().walk();
        self.extract_nodes(&mut cursor, content, path)
    }
}