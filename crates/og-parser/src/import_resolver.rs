use std::path::{Path, PathBuf};
use std::fs;

/// Resolves import paths to actual file paths
pub struct ImportResolver {
    base_path: PathBuf,
}

impl ImportResolver {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }

    /// Resolve an import path relative to a source file
    pub fn resolve_import(&self, import_path: &str, source_file: &Path) -> Option<String> {
        // Skip node_modules and external packages
        if self.is_external_import(import_path) {
            return None;
        }

        let source_dir = source_file.parent()?;
        
        // Try to resolve the import path
        if let Some(resolved) = self.resolve_path(import_path, source_dir) {
            // Convert to a file ID format matching what we use for nodes
            return Some(format!("file:{}", resolved.display()));
        }

        None
    }

    /// Check if this is an external/node_modules import
    fn is_external_import(&self, import_path: &str) -> bool {
        // External imports don't start with './' or '../' or '/'
        !import_path.starts_with('.')
            && !import_path.starts_with('/')
            && !import_path.starts_with('~')
    }

    /// Try to resolve a path with various strategies
    fn resolve_path(&self, import_path: &str, source_dir: &Path) -> Option<PathBuf> {
        let base_path = if import_path.starts_with('/') {
            // Absolute import from project root
            self.base_path.clone()
        } else {
            // Relative import
            source_dir.to_path_buf()
        };

        // Clean up the import path
        let import_path = import_path.trim_start_matches("./");
        let candidate_base = base_path.join(import_path);

        // Try different resolution strategies
        let extensions = [".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs"];
        
        // Strategy 1: Exact match
        if candidate_base.exists() && candidate_base.is_file() {
            return Some(self.normalize_path(candidate_base));
        }

        // Strategy 2: Try with extensions
        for ext in &extensions {
            let with_ext = PathBuf::from(format!("{}{}", candidate_base.display(), ext));
            if with_ext.exists() && with_ext.is_file() {
                return Some(self.normalize_path(with_ext));
            }
        }

        // Strategy 3: Directory with index file
        if candidate_base.exists() && candidate_base.is_dir() {
            for ext in &extensions {
                let index_path = candidate_base.join(format!("index{}", ext));
                if index_path.exists() && index_path.is_file() {
                    return Some(self.normalize_path(index_path));
                }
            }
        }

        // Strategy 4: Check package.json for main field (for local packages)
        if candidate_base.exists() && candidate_base.is_dir() {
            let package_json = candidate_base.join("package.json");
            if package_json.exists() {
                if let Ok(contents) = fs::read_to_string(&package_json) {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                        if let Some(main) = json.get("main").and_then(|m| m.as_str()) {
                            let main_path = candidate_base.join(main);
                            if main_path.exists() {
                                return Some(self.normalize_path(main_path));
                            }
                            // Try with extensions
                            for ext in &extensions {
                                let with_ext = PathBuf::from(format!("{}{}", main_path.display(), ext));
                                if with_ext.exists() {
                                    return Some(self.normalize_path(with_ext));
                                }
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// Normalize path to be relative to base_path if possible
    fn normalize_path(&self, path: PathBuf) -> PathBuf {
        // Try to canonicalize the path
        let canonical = path.canonicalize().unwrap_or(path);
        
        // Try to make it relative to the base path
        if let Ok(relative) = canonical.strip_prefix(&self.base_path) {
            self.base_path.join(relative)
        } else {
            canonical
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_resolve_relative_import() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        
        // Create test file structure
        let src_dir = base_path.join("src");
        fs::create_dir(&src_dir).unwrap();
        
        let utils_dir = src_dir.join("utils");
        fs::create_dir(&utils_dir).unwrap();
        
        fs::write(utils_dir.join("helper.ts"), "export const helper = () => {};").unwrap();
        fs::write(src_dir.join("main.ts"), "import { helper } from './utils/helper';").unwrap();
        
        let resolver = ImportResolver::new(base_path.clone());
        let source_file = src_dir.join("main.ts");
        
        // Test relative import resolution
        let resolved = resolver.resolve_import("./utils/helper", &source_file);
        assert!(resolved.is_some());
        assert!(resolved.unwrap().contains("utils/helper.ts"));
    }

    #[test]
    fn test_resolve_index_file() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        
        // Create test file structure with index file
        let src_dir = base_path.join("src");
        fs::create_dir(&src_dir).unwrap();
        
        let utils_dir = src_dir.join("utils");
        fs::create_dir(&utils_dir).unwrap();
        
        fs::write(utils_dir.join("index.ts"), "export * from './helper';").unwrap();
        fs::write(src_dir.join("main.ts"), "import { helper } from './utils';").unwrap();
        
        let resolver = ImportResolver::new(base_path.clone());
        let source_file = src_dir.join("main.ts");
        
        // Test index file resolution
        let resolved = resolver.resolve_import("./utils", &source_file);
        assert!(resolved.is_some());
        assert!(resolved.unwrap().contains("utils/index.ts"));
    }

    #[test]
    fn test_skip_external_imports() {
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        
        let resolver = ImportResolver::new(base_path.clone());
        let source_file = base_path.join("src/main.ts");
        
        // External imports should return None
        assert_eq!(resolver.resolve_import("react", &source_file), None);
        assert_eq!(resolver.resolve_import("@types/node", &source_file), None);
        assert_eq!(resolver.resolve_import("lodash/debounce", &source_file), None);
    }
}