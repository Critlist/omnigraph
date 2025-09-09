# Core Engine Extraction Plan

## What We're Keeping (The Good Stuff)

### 1. Parser System ✅

**Location**: `src/parser/`

- Tree-sitter based parsing
- Multi-language support (JavaScript, Python)
- AST extraction logic
- **NO VS Code dependencies** - This is clean!

### 2. Neo4j Database Layer ✅

**Location**: `src/database/`

- Query builders
- Connection management
- Data ingestion pipeline
- **Minor cleanup needed**: Remove VS Code config readers

### 3. Graph Analytics ✅

**Location**: `src/analytics/`

- PageRank implementation
- Centrality metrics
- Community detection
- **Completely clean** - No VS Code deps

### 4. Graph Generation Pipeline ✅

**Location**: `src/graph/`

- Graph schema
- Optimization algorithms
- Pipeline architecture
- **Completely clean** - Pure business logic

### 5. Service Architecture (Modified) ⚠️

**Location**: `src/services/`, `src/container/`

- Keep Inversify DI pattern
- Keep service interfaces
- **Strip out**: VS Code lifecycle hooks

## What We're Throwing Away (The Nightmare Fuel)

### 1. VS Code Extension Crap 🗑️

- `src/extension.ts` - GONE
- `src/managers/` - All VS Code specific managers - GONE
- `src/renderer/` - Webview provider with CSP hell - GONE
- All command palette registration - GONE
- All VS Code API imports - GONE

### 2. Webview Security Theater 🗑️

- Content Security Policy (CSP) - GONE
- Nonce generation - GONE
- Message passing restrictions - GONE
- Webview URI schemes - GONE

### 3. VS Code Configuration 🗑️

- Workspace settings - GONE
- Extension configuration - GONE
- VS Code specific package.json cruft - GONE

## New Clean Architecture

```
omnigraph-tauri/
├── src-tauri/              # Rust backend (minimal)
│   ├── src/
│   │   └── main.rs        # Just window creation & IPC
│   └── tauri.conf.json
│
├── src/                    # TypeScript/JS
│   ├── engine/            # EXTRACTED CLEAN ENGINE
│   │   ├── parser/        # Tree-sitter parsing
│   │   ├── database/      # Neo4j operations
│   │   ├── graph/         # Graph algorithms
│   │   ├── analytics/     # Graph analytics
│   │   └── services/      # Service layer
│   │
│   ├── visualization/     # NEW UNRESTRICTED 3D
│   │   ├── scene.ts      # Three.js scene setup
│   │   ├── graph3d.ts    # Force-directed 3D graph
│   │   └── controls.ts   # Camera & interaction
│   │
│   └── main.ts           # Simple app entry
│
├── index.html            # Clean HTML, no CSP bullshit
└── package.json          # Simple deps, no VS Code

```

## Migration Steps

### Step 1: Set Up Clean Tauri Project

```bash
cd omnigraph-tauri
pnpm install
pnpm add three neo4j-driver inversify reflect-metadata
pnpm add -D @types/three typescript
```

### Step 2: Copy Core Engine

```bash
# Create engine structure
mkdir -p src/engine/{parser,database,graph,analytics,services}

# Copy clean components
cp -r ../src/parser/* src/engine/parser/
cp -r ../src/database/* src/engine/database/
cp -r ../src/graph/* src/engine/graph/
cp -r ../src/analytics/* src/engine/analytics/
```

### Step 3: Clean Up Imports

- Remove all `import * as vscode`
- Remove all VS Code type imports
- Replace VS Code logger with console.log or simple file logger
- Replace VS Code config with simple JSON config

### Step 4: Create Simple Visualization

```typescript
// No CSP, no nonce, no restrictions!
import * as THREE from 'three';
import { ForceGraph3D } from '3d-force-graph';

// Just fucking render it!
const graph = ForceGraph3D()
  (document.getElementById('graph'))
  .graphData(data)
  .nodeLabel('name')
  .onNodeClick(node => {
    // Direct file opening, no VS Code API needed
    window.electronAPI.openFile(node.filePath);
  });
```

### Step 5: Simple Tauri Commands

```rust
#[tauri::command]
async fn parse_codebase(path: String) -> Result<GraphData, String> {
    // Call TypeScript parser via Node.js binding
    Ok(parsed_data)
}

#[tauri::command]
async fn open_file(path: String) -> Result<(), String> {
    // Just open the damn file
    open::that(path)?;
    Ok(())
}
```

## Benefits of This Approach

1. **No CSP Restrictions** - Load any resource, use any library
2. **No Webview Message Passing** - Direct function calls
3. **No Extension Manifest** - Just a desktop app
4. **No VS Code API Limitations** - Full Node.js access
5. **Simple File Access** - No workspace restrictions
6. **Direct Neo4j Connection** - No proxy needed
7. **Unrestricted Three.js** - Use any feature, any library

## What We Lose (And Don't Care About)

- VS Code integration (can add back later with thin wrapper)
- Command palette (replace with native menus)
- VS Code themes (use system theme or custom)
- Extension marketplace (distribute as desktop app)

## Implementation Priority

1. **Week 1**: Extract core engine, strip VS Code deps
2. **Week 2**: Set up Tauri, create simple UI
3. **Week 3**: Rebuild 3D visualization without restrictions
4. **Week 4**: Polish and optimize

## The Joy of Freedom

No more:

- "Content Security Policy directive"
- "Refused to load script"
- "Nonce mismatch"
- "Webview context isolation"
- "Extension host restrictions"

Just:

- Load Three.js ✅
- Connect to Neo4j ✅
- Parse files ✅
- Render graphs ✅
- Ship it! 🚀
