# Omnigraph: VS Code Extension to Tauri v2 Migration Plan

## Executive Summary

This document outlines the complete migration strategy for converting Omnigraph from a VS Code extension to a standalone Tauri v2 desktop application, with a planned VS Code wrapper for later integration.

## Current State Analysis

### VS Code Dependencies (12 files affected)

- **Extension Core**: `src/extension.ts` - Main activation and lifecycle
- **Managers**: Command, Progress, Status Bar managers tied to VS Code API
- **Renderer**: Webview provider and message handling
- **Configuration**: VS Code workspace settings integration
- **Security**: Credential storage via VS Code secrets API

### Core Engine Components (Preserve)

- **Parser Service**: Tree-sitter based AST parsing (JavaScript, Python)
- **Database Service**: Neo4j integration with query builders
- **Graph Service**: Graph generation and optimization pipeline
- **Analytics Service**: PageRank, centrality metrics, community detection
- **Visualization**: Three.js 3D rendering engine

### Build System

- Currently uses esbuild for VS Code extension bundling
- TypeScript with experimental decorators for Inversify
- Vitest for testing with TestContainers for integration tests

## Migration Phases

### Phase 0: Project Setup & Foundation (Week 1)

**Goal**: Establish Tauri project structure while maintaining existing code

#### Tasks

1. **Initialize Tauri v2 Project**

   ```bash
   npm create tauri-app@latest -- --beta
   # Choose: TypeScript, Vite, npm/pnpm
   ```

2. **Restructure Directories**

   ```
   omnigraph/
   ├── src-tauri/           # New: Rust backend
   ├── src/
   │   ├── engine/         # Move current src/ here
   │   ├── ui/             # New: Frontend components
   │   └── app/            # New: App initialization
   ├── public/             # Static assets
   └── index.html          # App entry point
   ```

3. **Update Build Configuration**
   - Add `vite.config.ts` for frontend bundling
   - Configure TypeScript paths for new structure
   - Update package.json scripts

4. **Create Abstraction Layer**
   - Interface for platform-specific features
   - Abstract file system, settings, and UI notifications

**Deliverables**:

- [ ] Tauri project initialized
- [ ] Code relocated to new structure
- [ ] Build scripts updated
- [ ] Abstraction interfaces defined

---

### Phase 1: Core Engine Isolation (Week 2)

**Goal**: Decouple core engine from VS Code dependencies

#### Tasks

1. **Remove VS Code Dependencies from Services**
   - Replace `vscode.window` calls with abstraction
   - Remove `vscode.workspace` configuration
   - Abstract logger to use console/file instead of VS Code output

2. **Create Platform Adapter Pattern**

   ```typescript
   interface IPlatformAdapter {
     showNotification(message: string): void;
     readFile(path: string): Promise<string>;
     writeFile(path: string, content: string): Promise<void>;
     getConfiguration<T>(key: string): T;
   }
   ```

3. **Refactor Service Container**
   - Remove VS Code specific bindings
   - Add platform adapter binding
   - Ensure services use adapter instead of direct VS Code calls

4. **Update Tests**
   - Mock platform adapter in tests
   - Ensure all tests pass with new abstractions

**Deliverables**:

- [ ] VS Code dependencies removed from core services
- [ ] Platform adapter implemented
- [ ] All unit tests passing
- [ ] Integration tests updated

---

### Phase 2: Tauri Backend Implementation (Week 3)

**Goal**: Implement Rust backend with Tauri commands

#### Tasks

1. **Core Tauri Commands**

   ```rust
   #[tauri::command]
   async fn parse_directory(path: String) -> Result<ParseResult, String>
   
   #[tauri::command]
   async fn connect_neo4j(config: Neo4jConfig) -> Result<bool, String>
   
   #[tauri::command]
   async fn generate_graph(options: GraphOptions) -> Result<GraphData, String>
   ```

2. **File System Operations**
   - Implement secure file access
   - Directory traversal with permissions
   - File watching for auto-refresh

3. **State Management**
   - Application state in Rust
   - Settings persistence
   - Session management

4. **IPC Bridge**
   - TypeScript to Rust command invocation
   - Event system for progress updates
   - Error handling and serialization

**Deliverables**:

- [ ] Tauri commands implemented
- [ ] File system operations working
- [ ] State management functional
- [ ] IPC communication established

---

### Phase 3: Frontend UI Development (Week 4)

**Goal**: Create native desktop UI experience

#### Tasks

1. **Window Management**
   - Main application window
   - Resizable panels
   - Native menus and shortcuts

2. **UI Components**
   - File explorer sidebar
   - Settings dialog
   - Status bar
   - Progress indicators

3. **3D Visualization Integration**
   - Port webview content to Tauri window
   - Ensure Three.js renders properly
   - Handle window resize events

4. **Theme System**
   - Light/dark theme support
   - System theme detection
   - Custom theme configuration

**Deliverables**:

- [ ] Main window functional
- [ ] UI components implemented
- [ ] 3D visualization working
- [ ] Theme system operational

---

### Phase 4: Feature Parity & Polish (Week 5)

**Goal**: Achieve feature parity with VS Code extension

#### Tasks

1. **Neo4j Integration**
   - Connection management UI
   - Connection testing
   - Error handling and recovery

2. **Graph Operations**
   - Generate graph from UI
   - Export functionality
   - Clear/refresh operations

3. **Performance Optimization**
   - Lazy loading for large codebases
   - Web Workers integration
   - Memory management

4. **User Experience**
   - Keyboard shortcuts
   - Context menus
   - Drag and drop support
   - Recent projects

**Deliverables**:

- [ ] All original features working
- [ ] Performance optimized
- [ ] UX improvements implemented
- [ ] Cross-platform testing complete

---

### Phase 5: Distribution & Packaging (Week 6)

**Goal**: Prepare for distribution

#### Tasks

1. **Auto-updater**
   - Implement Tauri updater
   - Update server configuration
   - Version management

2. **Installers**
   - Windows: MSI/NSIS installer
   - macOS: DMG with code signing
   - Linux: AppImage, deb, rpm

3. **CI/CD Pipeline**
   - GitHub Actions for builds
   - Automated testing
   - Release automation

4. **Documentation**
   - User guide
   - Installation instructions
   - Troubleshooting guide

**Deliverables**:

- [ ] Auto-updater functional
- [ ] Platform installers created
- [ ] CI/CD pipeline operational
- [ ] Documentation complete

---

### Phase 6: VS Code Extension Wrapper (Week 7-8)

**Goal**: Create lightweight VS Code extension that communicates with Tauri app

#### Tasks

1. **Extension Architecture**
   - Minimal VS Code extension
   - IPC/WebSocket communication
   - Process management

2. **Integration Points**
   - Command palette commands
   - Explorer context menu
   - Status bar integration

3. **Communication Protocol**
   - Define message protocol
   - Implement bidirectional communication
   - Handle app lifecycle

4. **Marketplace Preparation**
   - Extension packaging
   - Marketplace metadata
   - Publishing workflow

**Deliverables**:

- [ ] VS Code wrapper extension created
- [ ] Communication protocol working
- [ ] Extension published to marketplace
- [ ] Integration tested

## Technical Decisions

### Technology Stack

- **Desktop Framework**: Tauri v2 (Rust + WebView)
- **Frontend**: TypeScript + Vite + Three.js
- **Backend**: Rust with Tauri
- **Database**: Neo4j (external dependency)
- **Testing**: Vitest + Playwright (for E2E)
- **CI/CD**: GitHub Actions

### Architecture Patterns

- **Clean Architecture**: Separate business logic from framework
- **Dependency Injection**: Keep Inversify for service management
- **Platform Adapter**: Abstract platform-specific features
- **Command Pattern**: For Tauri IPC communication

### Data Flow

1. User Action → Tauri Frontend
2. Frontend → Tauri Command (IPC)
3. Tauri Command → TypeScript Engine (via adapter)
4. Engine → Neo4j Database
5. Results → Frontend → 3D Visualization

## Risk Mitigation

### High-Risk Areas

1. **Three.js Performance in WebView**
   - Mitigation: Test early, optimize rendering
   - Fallback: Consider native OpenGL if needed

2. **Neo4j Connectivity**
   - Mitigation: Robust error handling
   - Fallback: Embedded graph database option

3. **Cross-Platform Compatibility**
   - Mitigation: Regular testing on all platforms
   - Fallback: Platform-specific implementations

4. **File System Permissions**
   - Mitigation: Use Tauri's permission system
   - Fallback: Manual directory selection

## Success Metrics

### Phase Completion Criteria

- [ ] All tests passing (unit + integration)
- [ ] No VS Code dependencies in core engine
- [ ] Tauri app launches on all platforms
- [ ] Feature parity with original extension
- [ ] Performance benchmarks met
- [ ] Documentation complete

### Performance Targets

- App startup: < 2 seconds
- Graph generation: < 5 seconds for 1000 files
- Memory usage: < 500MB for typical project
- 3D rendering: 60 FPS for graphs < 500 nodes

## Timeline Summary

| Phase | Duration | Key Milestone |
|-------|----------|---------------|
| Phase 0 | Week 1 | Tauri project setup |
| Phase 1 | Week 2 | Engine isolated |
| Phase 2 | Week 3 | Backend functional |
| Phase 3 | Week 4 | UI complete |
| Phase 4 | Week 5 | Feature parity |
| Phase 5 | Week 6 | Ready for distribution |
| Phase 6 | Week 7-8 | VS Code wrapper |

**Total Duration**: 6-8 weeks for full migration

## Next Steps

1. **Immediate Actions**:
   - Set up Tauri v2 project
   - Create abstraction interfaces
   - Begin moving code to new structure

2. **Week 1 Goals**:
   - Complete Phase 0
   - Start Phase 1 refactoring
   - Set up CI/CD pipeline

3. **Communication**:
   - Weekly progress updates
   - Blocker identification
   - Architecture decision records

## Appendix: Command Mapping

| VS Code Command | Tauri Equivalent |
|-----------------|------------------|
| `omnigraph.showGraph` | Menu: View → Show Graph |
| `omnigraph.generateGraph` | Button/Menu: Generate Graph |
| `omnigraph.refreshGraph` | F5 or Refresh button |
| `omnigraph.exportGraph` | Menu: File → Export |
| `omnigraph.openSettings` | Menu: Edit → Preferences |
| `ctrl+shift+g` | Global shortcut: Cmd/Ctrl+G |

---

*This migration plan is a living document and will be updated as the migration progresses.*
