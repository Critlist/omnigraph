use crate::Parser;
use og_types::{ParsedFile, EngineResult, EngineError};
use og_utils::ProgressReporter;
use rayon::prelude::*;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{instrument, info};

/// Main parser engine that orchestrates language-specific parsers
pub struct ParserEngine {
    parsers: Vec<Box<dyn Parser>>,
    base_path: PathBuf,
}

impl ParserEngine {
    /// Create a new parser engine with all available parsers
    pub fn new() -> Self {
        Self::with_base_path(PathBuf::from("."))
    }
    
    /// Create a new parser engine with a specific base path
    pub fn with_base_path(base_path: PathBuf) -> Self {
        let mut parsers: Vec<Box<dyn Parser>> = vec![];
        
        #[cfg(feature = "js")]
        {
            parsers.push(Box::new(crate::javascript::JavaScriptParser::with_base_path(base_path.clone())));
        }
        
        #[cfg(feature = "ts")]
        {
            parsers.push(Box::new(crate::typescript::TypeScriptParser::with_base_path(base_path.clone())));
        }
        
        #[cfg(feature = "python")]
        {
            parsers.push(Box::new(crate::python::PythonParser::new()));
        }
        
        #[cfg(feature = "c")]
        {
            parsers.push(Box::new(crate::c::CParser::with_base_path(base_path.clone())));
        }
        
        Self { parsers, base_path }
    }
    
    /// Parse a single file
    #[instrument(skip(self, content))]
    pub fn parse_file(&self, path: &Path, content: &str) -> EngineResult<ParsedFile> {
        for parser in &self.parsers {
            if parser.can_parse(path) {
                info!("Parsing {} with {} parser", path.display(), parser.language().as_str());
                return parser.parse(path, content);
            }
        }
        
        Err(EngineError::ParseError {
            file: path.display().to_string(),
            message: "No parser found for file type".to_string(),
        })
    }
    
    /// Parse multiple files in parallel
    #[instrument(skip(self, files, progress))]
    pub fn parse_batch<'a>(
        &self,
        files: Vec<(String, String)>, // (path, content)
        progress: Option<Arc<dyn ProgressReporter>>,
    ) -> Vec<EngineResult<ParsedFile>> {
        let total = files.len();
        
        files
            .into_par_iter()
            .enumerate()
            .map(|(idx, (path_str, content))| {
                let path = Path::new(&path_str);
                
                if let Some(ref reporter) = progress {
                    let percentage = ((idx + 1) as f32 / total as f32) * 100.0;
                    reporter.report(&format!("Parsing {}", path.display()), percentage);
                }
                
                self.parse_file(path, &content)
            })
            .collect()
    }
    
    /// Get supported extensions across all parsers
    pub fn supported_extensions(&self) -> Vec<&str> {
        self.parsers
            .iter()
            .flat_map(|p| p.supported_extensions().iter().copied())
            .collect()
    }
}

impl Default for ParserEngine {
    fn default() -> Self {
        Self::new()
    }
}