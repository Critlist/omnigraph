use std::path::PathBuf;
use og_parser::ParserEngine;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("og_parser=debug")
        .init();
    
    // Create a simple JavaScript test file content
    let js_content = r#"
import { foo } from './foo';
import React from 'react';

export function testFunction(param1, param2) {
    console.log('Test function');
    return param1 + param2;
}

export class TestClass {
    constructor() {
        this.property = 'test';
    }
    
    method1() {
        return 'method1';
    }
}

const myVariable = 42;
export default myVariable;
"#;

    // Create parser engine
    let parser = ParserEngine::new();
    
    // Test JavaScript parsing
    println!("\n=== Testing JavaScript Parser ===");
    let js_path = PathBuf::from("test.js");
    match parser.parse_file(&js_path, js_content) {
        Ok(parsed) => {
            println!("✅ Successfully parsed JavaScript file");
            println!("  - Nodes found: {}", parsed.nodes.len());
            println!("  - Relationships found: {}", parsed.relationships.len());
            
            println!("\n  Nodes:");
            for node in &parsed.nodes {
                println!("    - {} ({:?}): {}", node.name, node.node_type, node.id);
            }
            
            println!("\n  Relationships:");
            for rel in &parsed.relationships {
                println!("    - {} --{:?}--> {}", rel.source, rel.relationship_type, rel.target);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse JavaScript: {}", e);
        }
    }
    
    // Test TypeScript parsing
    let ts_content = r#"
import { Component } from 'react';
import type { Props } from './types';

interface TestInterface {
    prop1: string;
    prop2: number;
}

type TestType = string | number;

export class MyComponent extends Component<Props> {
    private state: TestInterface = {
        prop1: 'test',
        prop2: 42
    };
    
    public render(): JSX.Element {
        return <div>Test</div>;
    }
}

export function helperFunction<T>(input: T): T {
    return input;
}
"#;
    
    println!("\n=== Testing TypeScript Parser ===");
    let ts_path = PathBuf::from("test.ts");
    match parser.parse_file(&ts_path, ts_content) {
        Ok(parsed) => {
            println!("✅ Successfully parsed TypeScript file");
            println!("  - Nodes found: {}", parsed.nodes.len());
            println!("  - Relationships found: {}", parsed.relationships.len());
            
            println!("\n  Nodes:");
            for node in &parsed.nodes {
                println!("    - {} ({:?}): {}", node.name, node.node_type, node.id);
            }
            
            println!("\n  Relationships:");
            for rel in &parsed.relationships {
                println!("    - {} --{:?}--> {}", rel.source, rel.relationship_type, rel.target);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse TypeScript: {}", e);
        }
    }
    
    // Test Python parsing
    let py_content = r#"
import os
from typing import List, Dict
from .module import MyClass

class TestClass:
    """Test class docstring"""
    
    def __init__(self):
        self.property = "test"
    
    def method1(self) -> str:
        return "method1"
    
    @staticmethod
    def static_method():
        return "static"

def test_function(param1: int, param2: str) -> Dict[str, int]:
    """Test function docstring"""
    result = {}
    for i in range(param1):
        result[param2 + str(i)] = i
    return result

MY_CONSTANT = 42
"#;
    
    println!("\n=== Testing Python Parser ===");
    let py_path = PathBuf::from("test.py");
    match parser.parse_file(&py_path, py_content) {
        Ok(parsed) => {
            println!("✅ Successfully parsed Python file");
            println!("  - Nodes found: {}", parsed.nodes.len());
            println!("  - Relationships found: {}", parsed.relationships.len());
            
            println!("\n  Nodes:");
            for node in &parsed.nodes {
                println!("    - {} ({:?}): {}", node.name, node.node_type, node.id);
            }
            
            println!("\n  Relationships:");
            for rel in &parsed.relationships {
                println!("    - {} --{:?}--> {}", rel.source, rel.relationship_type, rel.target);
            }
        }
        Err(e) => {
            println!("❌ Failed to parse Python: {}", e);
        }
    }
    
    // Test supported extensions
    println!("\n=== Supported Extensions ===");
    let extensions = parser.supported_extensions();
    println!("Extensions: {:?}", extensions);
}