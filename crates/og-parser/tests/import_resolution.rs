#[cfg(test)]
mod tests {
    use og_parser::typescript::TypeScriptParser;
    use og_parser::javascript::JavaScriptParser;
    use og_parser::Parser;
    use std::path::PathBuf;
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_typescript_import_resolution() {
        // Create temp directory with test files
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        
        // Create main.ts with imports
        let main_path = base_path.join("main.ts");
        let mut main_file = fs::File::create(&main_path).unwrap();
        writeln!(main_file, r#"
import {{ helper }} from './utils/helper';
import {{ Component }} from './components/Button';
import axios from 'axios';
import config from './config';

export function main() {{
    helper();
}}
"#).unwrap();

        // Create utils/helper.ts
        fs::create_dir_all(base_path.join("utils")).unwrap();
        let helper_path = base_path.join("utils/helper.ts");
        let mut helper_file = fs::File::create(&helper_path).unwrap();
        writeln!(helper_file, r#"
export function helper() {{
    console.log('Helper function');
}}
"#).unwrap();

        // Create components/Button.tsx
        fs::create_dir_all(base_path.join("components")).unwrap();
        let button_path = base_path.join("components/Button.tsx");
        let mut button_file = fs::File::create(&button_path).unwrap();
        writeln!(button_file, r#"
export const Component = () => {{
    return '<button>Click me</button>';
}}
"#).unwrap();

        // Create config/index.ts
        fs::create_dir_all(base_path.join("config")).unwrap();
        let config_path = base_path.join("config/index.ts");
        let mut config_file = fs::File::create(&config_path).unwrap();
        writeln!(config_file, r#"
export default {{
    apiUrl: 'https://api.example.com'
}};
"#).unwrap();

        // Parse main.ts
        let parser = TypeScriptParser::with_base_path(base_path.clone());
        let main_content = fs::read_to_string(&main_path).unwrap();
        let result = parser.parse(&main_path, &main_content).unwrap();
        
        // Check relationships - should have resolved imports
        let imports = result.relationships.iter()
            .filter(|r| matches!(r.relationship_type, og_types::RelationshipType::Imports))
            .collect::<Vec<_>>();
        
        println!("Found {} import relationships", imports.len());
        for import in &imports {
            println!("  {} -> {}", import.source, import.target);
        }
        
        // Should resolve local imports to actual file paths
        assert!(imports.iter().any(|r| r.target.contains("utils/helper.ts")));
        assert!(imports.iter().any(|r| r.target.contains("components/Button.tsx")));
        assert!(imports.iter().any(|r| r.target.contains("config/index.ts")));
        
        // External import should not be resolved to a file
        let has_axios = imports.iter().any(|r| r.target.contains("axios"));
        assert!(!has_axios || imports.iter().any(|r| r.target.starts_with("external:")));
    }
    
    #[test]
    fn test_javascript_import_resolution() {
        // Create temp directory with test files
        let temp_dir = TempDir::new().unwrap();
        let base_path = temp_dir.path().to_path_buf();
        
        // Create main.js with imports
        let main_path = base_path.join("main.js");
        let mut main_file = fs::File::create(&main_path).unwrap();
        writeln!(main_file, r#"
import {{ helper }} from './utils/helper.js';
import Component from './components/Button';
import config from './config';

export function main() {{
    helper();
}}
"#).unwrap();

        // Create utils/helper.js
        fs::create_dir_all(base_path.join("utils")).unwrap();
        let helper_path = base_path.join("utils/helper.js");
        let mut helper_file = fs::File::create(&helper_path).unwrap();
        writeln!(helper_file, r#"
export function helper() {{
    console.log('Helper function');
}}
"#).unwrap();

        // Create components/Button.jsx
        fs::create_dir_all(base_path.join("components")).unwrap();
        let button_path = base_path.join("components/Button.jsx");
        let mut button_file = fs::File::create(&button_path).unwrap();
        writeln!(button_file, r#"
export default function Button() {{
    return '<button>Click me</button>';
}}
"#).unwrap();

        // Create config/index.js
        fs::create_dir_all(base_path.join("config")).unwrap();
        let config_path = base_path.join("config/index.js");
        let mut config_file = fs::File::create(&config_path).unwrap();
        writeln!(config_file, r#"
module.exports = {{
    apiUrl: 'https://api.example.com'
}};
"#).unwrap();

        // Parse main.js
        let parser = JavaScriptParser::with_base_path(base_path.clone());
        let main_content = fs::read_to_string(&main_path).unwrap();
        let result = parser.parse(&main_path, &main_content).unwrap();
        
        // Check relationships - should have resolved imports
        let imports = result.relationships.iter()
            .filter(|r| matches!(r.relationship_type, og_types::RelationshipType::Imports))
            .collect::<Vec<_>>();
        
        println!("Found {} import relationships", imports.len());
        for import in &imports {
            println!("  {} -> {}", import.source, import.target);
        }
        
        // Should resolve local imports to actual file paths
        assert!(imports.iter().any(|r| r.target.contains("utils/helper.js")), 
                "Should find utils/helper.js import");
        assert!(imports.iter().any(|r| r.target.contains("components/Button.jsx")),
                "Should find components/Button.jsx import");
        assert!(imports.iter().any(|r| r.target.contains("config/index.js")),
                "Should find config/index.js import");
    }
}