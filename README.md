# Omnigraph: 3D Codebase Visualization

A powerful desktop application for visualizing code dependencies and relationships as interactive 3D force-directed graphs. Built with Rust, Tauri, and Three.js to escape the limitations of VS Code's restrictive webview environment.

![Status](https://img.shields.io/badge/status-alpha-orange)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20macos%20%7C%20windows-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## 🚀 Overview

Omnigraph transforms your codebase into an interactive 3D visualization, helping you understand complex dependencies, identify architectural patterns, and navigate large projects with ease. Originally conceived as a VS Code extension, it has evolved into a standalone desktop application powered by Rust and Tauri for maximum performance and freedom.

## ✨ Features

- **🎨 Interactive 3D Visualization**: Navigate your codebase in a beautiful force-directed 3D graph
- **⚡ Rust-Powered Parsing**: Lightning-fast AST parsing with tree-sitter for JavaScript, TypeScript, and Python
- **📊 Real-time Progress**: Visual feedback during parsing with percentage completion
- **🔍 Smart Filtering**: Automatically handles unresolved imports and external dependencies
- **📐 Responsive Canvas**: Automatic resizing and optimal rendering on any screen size
- **🔄 Hot Reset**: Quick reset functionality for testing and debugging
- **🎯 Node Details**: Click nodes to see file paths, metrics, and relationships

## 📦 Installation

### Prerequisites

- [Node.js](https://nodejs.org/) (v16 or higher)
- [pnpm](https://pnpm.io/) (recommended) or npm
- [Rust](https://rustup.rs/) (latest stable)
- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites)

### Quick Start

```bash
# Clone the repository
git clone https://github.com/yourusername/omnigraph.git
cd omnigraph

# Install dependencies
pnpm install

# Run in development mode
pnpm tauri:dev

# Build for production
pnpm tauri:build
```

### Linux/Wayland Users

If you're using Wayland (especially with Hyprland), the app includes compatibility flags in the `pnpm tauri:dev` command. These are automatically applied.

## 🎮 Usage

1. **Launch the Application**: Run `pnpm tauri:dev` or use the built executable
2. **Parse a Codebase**: Click "📁 Parse Codebase" and select your project directory
3. **Generate Graph**: Click "🎨 Generate Graph" to visualize the parsed data
4. **Explore**: 
   - Rotate: Left-click and drag
   - Zoom: Scroll wheel
   - Pan: Right-click and drag
   - Node Info: Click on any node to focus
5. **Reset**: Click "🗑️ Reset App" to clear and start fresh

## 🏗️ Architecture

### Technology Stack

- **Backend**: Rust with Tauri framework
- **Parser**: Tree-sitter for language-agnostic AST parsing
- **Graph Engine**: Petgraph for graph algorithms
- **Frontend**: TypeScript with Three.js
- **3D Rendering**: 3d-force-graph for WebGL visualization
- **Build System**: Vite for frontend, Cargo for Rust

### Supported Languages

- ✅ JavaScript (.js, .jsx)
- ✅ TypeScript (.ts, .tsx)
- ✅ Python (.py)
- 🔜 Rust (.rs) - Coming soon
- 🔜 Go (.go) - Planned
- 🔜 Java (.java) - Planned

## 🛠️ Development

### Project Structure

```
omnigraph/
├── src-tauri/          # Rust backend
│   ├── src/
│   │   ├── engine/     # Core parsing and graph engine
│   │   └── lib.rs      # Tauri commands
│   └── Cargo.toml
├── src/                # Frontend
│   ├── main.ts         # Application entry
│   └── visualization/  # 3D graph components
├── package.json
└── README.md
```

### Key Commands

```bash
# Development
pnpm tauri:dev         # Run with hot-reload
pnpm test              # Run tests
pnpm lint              # Lint code

# Building
cargo tauri build               # Production build
cargo tauri build --debug       # Debug build

# Platform-specific builds
cargo tauri build --target x86_64-pc-windows-msvc   # Windows
cargo tauri build --target x86_64-apple-darwin      # macOS Intel
cargo tauri build --target aarch64-apple-darwin     # macOS M1/M2
cargo tauri build --target x86_64-unknown-linux-gnu # Linux
```

### Adding Language Support

1. Add tree-sitter grammar to `Cargo.toml`
2. Create parser module in `src-tauri/src/engine/parser/`
3. Implement the `Parser` trait
4. Register in `ParserManager`

See `CLAUDE.md` for detailed development guidelines.

## 🚧 Roadmap

### Current Status: Alpha

- [x] Core Rust parsing engine
- [x] Tree-sitter integration
- [x] 3D force-directed graph
- [x] Progress indicators
- [x] Reset functionality
- [x] Responsive canvas

### Next Up

- [ ] Import path resolution
- [ ] Neo4j integration for persistence
- [ ] Graph state save/load
- [ ] VS Code extension wrapper
- [ ] More language parsers
- [ ] Graph analytics (PageRank, centrality)
- [ ] Export to various formats (JSON, GraphML)

### Future Vision

- [ ] AI-powered code insights
- [ ] Team collaboration features
- [ ] Git history visualization
- [ ] Performance profiling overlay
- [ ] Custom graph layouts
- [ ] Plugin system

## 🐛 Known Issues

- Import statements to external packages create orphaned edges (filtered out)
- Large codebases (>10,000 files) may take time to parse
- Wayland requires X11 compatibility mode (handled automatically)

See `TODO.md` for complete issue tracking.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- [Tauri](https://tauri.app/) - For the amazing desktop framework
- [Tree-sitter](https://tree-sitter.github.io/) - For robust language parsing
- [Three.js](https://threejs.org/) - For 3D graphics
- [3d-force-graph](https://github.com/vasturiano/3d-force-graph) - For the force-directed layout

## 📬 Contact

Project Link: [https://github.com/yourusername/omnigraph](https://github.com/yourusername/omnigraph)

---

**Built with ❤️ to escape the tyranny of VS Code's CSP restrictions**