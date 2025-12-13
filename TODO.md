# TODO

## 🟡 IN PROGRESS - C Parser & Graph Building Issues

### Priority 0: Fix Import Relationships & Graph Connectivity

#### Issues Identified (Dec 9, 2024)

##### C Parser Import Resolution
- ✅ Fixed file ID generation to use consistent hashing
- ✅ Added proper path resolution for relative includes  
- ✅ Resolved include paths now match actual parsed file IDs
- ✅ Added `pnpm tauri:dev:log` for logging to omnigraph.log file

##### Remaining Issues
- ⚠️ ~30 files (out of 79) appear as disconnected nodes despite having includes
- ⚠️ 9 import relationships created by C parser don't make it to LOD system  
- ⚠️ Graph builder drops relationships when target nodes don't exist (system headers)
- ⚠️ Some import relationships lost between parser output and graph builder input

#### Root Causes Found

1. **Graph Builder Filtering** (builder.rs:86-87): Only adds relationships if BOTH nodes exist in node_map
2. **System Headers**: Imports to fcntl.h, unistd.h etc. dropped because those files aren't parsed
3. **Data Flow Issue**: Some relationships created by parser don't reach LOD system
4. **Multiple Codebases**: /src/ and /docs/historical/original-source/ create separate clusters

**Status**: Edges ARE showing for ~60% of files. Main `hack.h` cluster connects properly. Issue is with remaining disconnected nodes.

**Next Steps**:
- [ ] Investigate why certain import relationships don't flow from parser → graph builder → LOD
- [ ] Consider creating placeholder nodes for external/system includes
- [ ] Fix data flow issue causing ~30% of files to appear disconnected
- [ ] Add option to filter out system includes or handle them differently

### Priority 1: Performance & UI Issues

#### Analytics Engine Performance
- ✅ Commented out Louvain community detection (O(n²) complexity) for faster debugging
- ✅ Fixed betweenness centrality with sampling (was O(n³))
- ✅ Added panic catching and error recovery
- ✅ Cleaned up debug console logs

#### Remaining Performance Issues  
- [ ] Progress bar disappears/reappears between 80-100%
- [ ] Add UI checkboxes for selecting which analyses to run
- [ ] Create backend configuration for selective analysis
- [ ] Add progressive loading - show graph immediately, add analytics as they complete
- [ ] Implement smart defaults - auto-disable expensive analyses for large graphs

## 🚨 CRITICAL PATH TO MVP - Wire Analytics to UI

### Priority 1: Connect Metrics to Frontend (THIS WEEK)

#### 1.1 Pass Analytics Data to Frontend ⚡

- [ ] Ensure `analyze_with_metrics` returns full metric data to frontend
- [ ] Add metrics to GraphNode structure in frontend
- [ ] Create TypeScript interfaces for metric data
- [ ] Test data flow from Rust → Tauri → Frontend

#### 1.2 Micro HUD Display ⚡

- [ ] Create floating HUD component (top 2-3 metrics)
- [ ] Display: node name, importance score, risk score
- [ ] Add smooth transitions and hover effects
- [ ] Position HUD near cursor or fixed corner
- [ ] Wire HUD to selected node data

#### 1.3 Visual Encodings on Graph ⚡

- [ ] Node size = importance metric (scale 0.5x to 2x)
- [ ] Node color = community ID (distinct colors)
- [ ] Add risk indicator (red border if risk > 0.7)
- [ ] Chokepoint glow effect (if chokepoint > 0.7)
- [ ] Update node rendering in graph3d.ts

### Priority 2: Properties Panel & Controls (NEXT WEEK)

#### 2.1 Properties Panel

- [ ] Create collapsible side panel component
- [ ] Display all composite metrics (importance, risk, chokepoint, payoff)
- [ ] Show raw metrics breakdown in table format
- [ ] Add normalized values with progress bars
- [ ] Include community assignment with color indicator
- [ ] Add file path for code navigation

#### 2.2 Controls & Filtering

- [ ] Add metric selector dropdown (importance/risk/chokepoint/payoff)
- [ ] Implement "Top N" nodes filter slider
- [ ] Add threshold controls for highlighting
- [ ] Create "Recompute Metrics" button
- [ ] Add export button for metrics JSON

### Priority 3: Neo4j Integration (OPTIONAL but valuable)

- [ ] Implement Neo4j client in `og-db` crate using `neo4rs`
- [ ] Create batch node/relationship ingestion
- [ ] Add Cypher query templates for GDS algorithms
- [ ] Implement graph projections for analysis
- [ ] Persist computed metrics back to Neo4j
- [ ] Enable advanced graph queries and traversals

## Future Enhancements (Post-MVP)

### Code Quality Integration

- [ ] Add git history analysis for churn metrics
- [ ] Implement cyclomatic complexity from AST
- [ ] Add code ownership detection
- [ ] Integrate test coverage data

### Import Resolution Improvements

- [ ] Fix destructured import detection in JavaScript
- [ ] Add dynamic import/require() support
- [ ] Support webpack aliases and tsconfig paths

### Performance & Developer Experience

- [ ] Add memory-mapped file reading for large files
- [ ] Implement incremental parsing for file changes
- [ ] Add `.omnigraphignore` file support
- [ ] Create configuration file support
- [ ] Add crash reporting with redacted paths

### Additional Language Support

- [ ] Implement Rust parser
- [ ] Add Go parser
- [ ] Support Java/C#
- [ ] Complete C/C++ parser

### Advanced Analytics

- [ ] Implement bridge detection for critical connectors
- [ ] Add articulation point detection
- [ ] Create influence propagation analysis
- [ ] Add technical debt scoring

## 🤖 AI Integration Phase (Post-MVP)

### Phase 1: Data Pipeline & Infrastructure (Weeks 1-2)

#### Graph Feature Extraction

- [ ] Add feature extraction to Rust engine (pagerank, centrality, complexity vectors)
- [ ] Create training data export (graphs → JSON/Parquet format)
- [ ] Implement graph embeddings (node2vec or graph2vec)
- [ ] Set up Python inference sidecar with Tauri IPC bridge
- [ ] Create caching layer for embeddings in SQLite

#### Model Infrastructure

- [ ] Set up Hugging Face integration for model storage
- [ ] Implement model versioning and rollback system
- [ ] Create inference pipeline with batching support
- [ ] Add telemetry for model performance tracking

### Phase 2: Pattern Detection Agent (Weeks 3-4)

#### Training & Fine-tuning

- [ ] Fine-tune CodeT5-small or CodeBERT on graph query patterns
- [ ] Create labeled dataset of antipatterns (circular deps, god objects, dead code)
- [ ] Implement Graph Neural Network (GNN) using PyTorch Geometric
- [ ] Train pattern detector on 100-1000 manually labeled examples
- [ ] Use LoRA/QLoRA for efficient fine-tuning (reduce GPU needs by 90%)

#### Integration

- [ ] Expose pattern detection via Tauri command
- [ ] Create UI overlay for detected patterns
- [ ] Add confidence scores and explanations
- [ ] Implement user feedback collection for online learning

### Phase 3: Risk Analysis Agent (Weeks 5-6)

#### Risk Scoring Model

- [ ] Combine PageRank with cyclomatic complexity metrics
- [ ] Train XGBoost classifier for tech debt prediction
- [ ] Implement SHAP values for explainable risk scores
- [ ] Create risk heatmap overlay on 3D visualization
- [ ] Add historical change frequency analysis

#### Dependency Impact Predictor

- [ ] Analyze historical git changes with graph structure
- [ ] Train model to predict ripple effects of changes
- [ ] Create "blast radius" visualization
- [ ] Implement change impact scoring

### Phase 4: Smart Navigation Assistant (Weeks 7-8)

#### Natural Language Interface

- [ ] Implement graph query agent ("What depends on auth.rs?")
- [ ] Add conversational navigation ("Show me the most complex modules")
- [ ] Create code explanation generator using graph context
- [ ] Implement semantic code search across graph

#### Architecture Guardian

- [ ] Learn intended architecture from examples
- [ ] Detect layer violations and circular dependencies
- [ ] Suggest refactoring opportunities based on patterns
- [ ] Create architectural fitness functions

### Phase 5: Production Features (Weeks 9-10)

#### Performance Optimization

- [ ] Implement gradient checkpointing for memory efficiency
- [ ] Add mixed precision training (fp16)
- [ ] Create batch processing for large codebases
- [ ] Implement real-time incremental analysis

#### Developer Experience

- [ ] Add VS Code extension integration
- [ ] Create CI/CD integration for automated analysis
- [ ] Implement team dashboards for tech debt tracking
- [ ] Add export to SARIF format for tool integration

### Model & Library Stack

#### Code Understanding Models

- [ ] Integrate CodeT5+ (Salesforce) for code-to-text
- [ ] Add GraphCodeBERT (Microsoft) for structure analysis
- [ ] Implement UniXcoder for cross-language support
- [ ] Experiment with StarCoder for larger contexts

#### Graph Analysis Libraries

- [ ] Set up PyTorch Geometric for GNN implementation
- [ ] Add DGL as alternative GNN framework
- [ ] Integrate NetworkX for quick prototyping
- [ ] Use sentence-transformers for code embeddings

### Minimal MVP Prototype (2 weeks)

#### "Smart Graph Navigator" Quick Win

- [ ] Create `get_node_features` Tauri command returning feature vectors
- [ ] Implement `get_subgraph_embedding` for pattern matching
- [ ] Build Python GraphAgent class with CodeBERT encoder
- [ ] Add antipattern detection with pre-trained model
- [ ] Create simple UI for showing AI insights on hover
- [ ] Implement "Find Similar Structures" feature
- [ ] Add basic risk explanation with SHAP values

### Success Metrics for AI Integration

- [ ] Pattern detection accuracy > 85% on test set
- [ ] Risk prediction correlates with actual bug density (>0.7 correlation)
- [ ] Query response time < 500ms for graph traversal
- [ ] User feedback rating > 4/5 for AI suggestions
- [ ] 50% reduction in time to identify tech debt hotspots

### Testing

- [ ] Create golden fixture tests with known codebases
- [ ] Add snapshot tests for metric calculations
- [ ] Implement property-based tests
- [ ] Add performance benchmarks

## ✅ Completed Features

### Core Rust Engine & Analytics

- ✅ Complete Cargo workspace with 7 crates (types, parser, db, analytics, services, utils, graph)
- ✅ JavaScript, TypeScript, Python, C parsers with tree-sitter
- ✅ Parallel batch parsing with Rayon
- ✅ Import resolution system for edge creation
- ✅ **15+ graph algorithms implemented:**
  - PageRank, betweenness, closeness, eigenvector centrality
  - K-core decomposition, clustering coefficient
  - Louvain community detection
  - Coupling/cohesion metrics
  - Risk analysis and chokepoint detection
- ✅ Normalization with percentile ranges
- ✅ Composite metrics (importance, chokepoint, risk, payoff)
- ✅ Caching layer with DashMap
- ✅ Change impact analysis with propagation

### Tauri Desktop Application

- ✅ Migration from VS Code extension to Tauri v2
- ✅ 3D force-directed graph visualization (Three.js)
- ✅ Real-time progress indicators
- ✅ File system access and tree view
- ✅ Responsive canvas with resize handling
- ✅ Reset functionality
- ✅ Async command handlers with proper Send/Sync

## 📊 Current Status

**Analytics Engine**: 100% complete - All algorithms implemented and tested
**Parser System**: 90% complete - Minor import resolution issues
**Frontend Integration**: 10% complete - Metrics computed but not displayed
**Database Layer**: 5% complete - Structure exists, implementation pending

## 🎯 Success Criteria for MVP

- [ ] Metrics visible in HUD when selecting nodes
- [ ] Node visual encoding reflects metrics (size, color, glow)
- [ ] Properties panel shows full metric breakdown
- [ ] Can filter/sort nodes by any metric
- [ ] Export metrics to JSON for analysis
