use og_types::{ParsedFile, Language, EngineResult};
use std::path::Path;

/// Trait for language-specific parsers
pub trait Parser: Send + Sync {
    /// Get supported file extensions
    fn supported_extensions(&self) -> &[&str];
    
    /// Get the language this parser handles
    fn language(&self) -> Language;
    
    /// Check if this parser can handle the given file
    fn can_parse(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            let ext_str = format!(".{}", ext.to_string_lossy());
            self.supported_extensions().iter().any(|&e| e == ext_str)
        } else {
            false
        }
    }
    
    /// Parse a single file
    fn parse(&self, path: &Path, content: &str) -> EngineResult<ParsedFile>;
}