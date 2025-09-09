use serde::{Deserialize, Serialize};

use std::path::PathBuf;

/// Parsed file representation
#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub language: Language,
    pub nodes: Vec<AstNode>,
    pub relationships: Vec<Relationship>,
    pub metrics: crate::metrics::FileMetrics,
}

/// AST node representation
#[derive(Debug, Clone)]
pub struct AstNode {
    pub id: String,
    pub node_type: NodeType,
    pub name: String,
    pub start_line: usize,
    pub end_line: usize,
    pub children: Vec<String>,
}

/// Node types in the AST
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeType {
    File,
    Module,
    Class,
    Interface,
    Function,
    Method,
    Variable,
    Property,
    Import,
    Export,
    TypeAlias,
    Enum,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::File => "file",
            NodeType::Module => "module",
            NodeType::Class => "class",
            NodeType::Interface => "interface",
            NodeType::Function => "function",
            NodeType::Method => "method",
            NodeType::Variable => "variable",
            NodeType::Property => "property",
            NodeType::Import => "import",
            NodeType::Export => "export",
            NodeType::TypeAlias => "type",
            NodeType::Enum => "enum",
        }
    }
}

/// Relationship between AST nodes
#[derive(Debug, Clone)]
pub struct Relationship {
    pub source: String,
    pub target: String,
    pub relationship_type: RelationshipType,
}

/// Types of relationships between nodes
#[derive(Debug, Clone, Copy)]
pub enum RelationshipType {
    Contains,
    Calls,
    Imports,
    Exports,
    Extends,
    Implements,
    References,
}

impl RelationshipType {
    pub fn as_str(&self) -> &'static str {
        match self {
            RelationshipType::Contains => "CONTAINS",
            RelationshipType::Calls => "CALLS",
            RelationshipType::Imports => "IMPORTS",
            RelationshipType::Exports => "EXPORTS",
            RelationshipType::Extends => "EXTENDS",
            RelationshipType::Implements => "IMPLEMENTS",
            RelationshipType::References => "REFERENCES",
        }
    }
}

/// Supported programming languages
#[derive(Debug, Clone, Copy)]
pub enum Language {
    JavaScript,
    TypeScript,
    Python,
    Rust,
    C,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Language::JavaScript => "javascript",
            Language::TypeScript => "typescript",
            Language::Python => "python",
            Language::Rust => "rust",
            Language::C => "c",
        }
    }

    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            Language::JavaScript => &[".js", ".mjs", ".cjs", ".jsx"],
            Language::TypeScript => &[".ts", ".tsx", ".mts", ".cts"],
            Language::Python => &[".py", ".pyi"],
            Language::Rust => &[".rs"],
            Language::C => &[".c", ".h"],
        }
    }
}

