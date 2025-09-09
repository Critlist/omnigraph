/**
 * Omnigraph Tauri App - Main Entry Point
 * FREEDOM FROM VS CODE RESTRICTIONS!
 */

import { Graph3DVisualization, GraphData } from "./visualization/graph3d";
import "./style.css";
import "./ui/styles/panels.css";
import { PanelManager } from "./ui/panel-system/panel-manager";
import { SplitterManager } from "./ui/panel-system/splitter-manager";
import { panelsState, layoutPresets } from "./state/panels";
import { eventBus } from "./state/events";
import "./ui/components/og-command-palette";
import "./ui/components/og-menu-bar";
import { AnalyzedGraph, UINodeMetricsV1, findMetricsForNode } from "./types/metrics";

// Extend window interface for Tauri
declare global {
    interface Window {
        __TAURI__?: any;
    }
}

// Check if we're in Tauri or browser
const isTauri = window.__TAURI__ !== undefined;

// Mock Tauri API for browser development
let invoke: any;
let open: any;
let openDialog: any;

// Initialize Tauri or mock APIs
async function initializeAPIs() {
    if (isTauri) {
        console.log("🚀 Running in Tauri mode");
        // Dynamic imports for Tauri environment
        try {
            const tauriCore = await import("@tauri-apps/api/core");
            const tauriDialog = await import("@tauri-apps/plugin-dialog");
            
            invoke = tauriCore.invoke;
            open = async (path: string) => {
                // For now, just log - opener plugin might need different setup
                console.log("Open file:", path);
            };
            openDialog = tauriDialog.open;
            console.log("✅ Tauri APIs loaded");
        } catch (error) {
            console.error("❌ Failed to load Tauri APIs:", error);
        }
    } else {
    // Mock functions for browser
    console.log("🌐 Running in browser mode - Tauri API mocked");
    
    invoke = async (cmd: string, args?: any) => {
        console.log(`Mock invoke: ${cmd}`, args);
        
        // Return mock data based on command
        if (cmd === "parse_codebase") {
            return {
                fileCount: 10,
                nodeCount: 50,
                edgeCount: 75,
                languages: ["JavaScript", "Python"]
            };
        }
        if (cmd === "generate_graph") {
            // Return mock graph data
            return {
                nodes: [
                    { id: "1", name: "main.js", type: "file", filePath: "/src/main.js" },
                    { id: "2", name: "App", type: "class", filePath: "/src/main.js" },
                    { id: "3", name: "render", type: "function", filePath: "/src/main.js" },
                ],
                links: [
                    { source: "1", target: "2", type: "contains" },
                    { source: "2", target: "3", type: "contains" },
                ]
            };
        }
        return null;
    };
    
    open = async (path: string) => {
        console.log(`Mock open: ${path}`);
    };
    
    openDialog = async (options: any) => {
        console.log("Mock dialog:", options);
        return "/mock/path/to/project";
    };
    }
}

// Global instances
let graph3d: Graph3DVisualization | null = null;
let panelManager: PanelManager | null = null;
let splitterManager: SplitterManager | null = null;
let currentMetrics: UINodeMetricsV1[] = [];  // Store metrics globally

// Initialize the app
async function initializeApp() {
    console.log("🚀 Omnigraph starting up - NO CSP, NO NONCE!");
    
    // Initialize APIs first
    await initializeAPIs();
    
    // Test Tauri connection
    if (isTauri && invoke) {
        try {
            console.log("🔧 Testing Tauri connection...");
            // Try a simple command that should always work
            const testResult = await invoke("get_saved_graph");
            console.log("✅ Tauri connection working, test result:", testResult);
        } catch (testError) {
            console.error("❌ Tauri connection test failed:", testError);
            console.error("This might indicate a problem with the Tauri backend");
        }
    }
    
    // Set up progress event listener
    if (isTauri && window.__TAURI__) {
        try {
            const { listen } = await import("@tauri-apps/api/event");
            await listen("parse-progress", (event: any) => {
                const progress = event.payload;
                updateProgress(progress);
            });
        } catch (error) {
            console.error("Failed to set up event listener:", error);
        }
    }
    
    // Debug: Check if elements exist
    const app = document.getElementById("app");
    const controlPanel = document.getElementById("control-panel");
    const graphContainer = document.getElementById("graph-container");
    
    console.log("App element:", app);
    console.log("Control panel:", controlPanel);
    console.log("Graph container:", graphContainer);
    
    // Create the graph visualization
    if (graphContainer) {
        try {
            graph3d = new Graph3DVisualization("graph-container");
            console.log("✅ 3D Graph initialized");
        } catch (error) {
            console.error("❌ Failed to initialize 3D graph:", error);
        }
    } else {
        console.error("❌ Graph container not found!");
    }

    // Initialize UI systems
    panelManager = new PanelManager();
    splitterManager = new SplitterManager();
    console.log("✅ Panel system initialized");

    // Set up event handlers
    setupEventHandlers();
    setupUIEventHandlers();

    // Load initial data if available
    await loadInitialData();
}

function setupEventHandlers() {
    // File open handler - only in Tauri
    if (isTauri) {
        window.addEventListener("open-file", async (event: Event) => {
            const customEvent = event as CustomEvent;
            const { path } = customEvent.detail;
            console.log(`Opening file: ${path}`);
            await open(path);
        });
    }

    // Parse button
    const parseBtn = document.getElementById("parse-btn");
    if (parseBtn) {
        parseBtn.addEventListener("click", async () => {
            await parseCodebase();
        });
    }

    // Connect to Neo4j button
    const connectBtn = document.getElementById("connect-btn");
    if (connectBtn) {
        connectBtn.addEventListener("click", async () => {
            await connectToNeo4j();
        });
    }

    // Generate graph button
    const generateBtn = document.getElementById("generate-btn");
    if (generateBtn) {
        generateBtn.addEventListener("click", async () => {
            await generateGraph();
        });
    }

    // Reset view button
    const resetBtn = document.getElementById("reset-btn");
    if (resetBtn) {
        resetBtn.addEventListener("click", () => {
            graph3d?.resetView();
        });
    }

    // Reset app button
    const resetAppBtn = document.getElementById("reset-app-btn");
    if (resetAppBtn) {
        resetAppBtn.addEventListener("click", async () => {
            await resetApp();
        });
    }

    // Toggle panels button
    const togglePanelsBtn = document.getElementById("toggle-panels-btn");
    if (togglePanelsBtn) {
        togglePanelsBtn.addEventListener("click", () => {
            togglePanels();
        });
    }

    // Keyboard shortcuts
    document.addEventListener("keydown", (e) => {
        // Ctrl+B - Toggle left panel
        if (e.ctrlKey && e.key === "b") {
            e.preventDefault();
            panelManager?.toggle("file-tree");
        }
        // Ctrl+J - Toggle bottom panel
        if (e.ctrlKey && e.key === "j") {
            e.preventDefault();
            panelManager?.toggle("terminal");
        }
        // Ctrl+\ - Toggle right panel
        if (e.ctrlKey && e.key === "\\") {
            e.preventDefault();
            panelManager?.toggle("properties");
        }
        // Ctrl+1/2/3 - Switch layouts
        if (e.ctrlKey && e.key === "1") {
            e.preventDefault();
            switchLayout("explore");
        }
        if (e.ctrlKey && e.key === "2") {
            e.preventDefault();
            switchLayout("inspect");
        }
        if (e.ctrlKey && e.key === "3") {
            e.preventDefault();
            switchLayout("debug");
        }
    });
}

async function loadInitialData() {
    try {
        // Try to load saved graph data
        const data = await invoke("get_saved_graph") as GraphData;
        if (data && graph3d) {
            graph3d.loadData(data);
            console.log("✅ Loaded saved graph data");
        }
    } catch (error) {
        console.log("No saved graph data found");
    }
}

function updateProgress(progress: any) {
    const container = document.getElementById("progress-container");
    const bar = document.getElementById("progress-bar");
    const text = document.getElementById("progress-text");
    
    if (container && bar && text) {
        container.style.display = "block";
        bar.style.width = `${progress.percentage}%`;
        text.textContent = `${Math.round(progress.percentage)}% - ${progress.message}`;
        
        // Hide progress bar after completion
        if (progress.percentage >= 100) {
            setTimeout(() => {
                container.style.display = "none";
            }, 2000);
        }
    }
}

async function parseCodebase() {
    try {
        const status = document.getElementById("status");
        
        // Open directory selection dialog
        const selectedDir = await openDialog({
            directory: true,
            multiple: false,
            title: "Select Codebase Directory"
        });

        if (!selectedDir) {
            console.log("No directory selected");
            return;
        }

        // Reset progress bar
        const progressContainer = document.getElementById("progress-container");
        const progressBar = document.getElementById("progress-bar");
        const progressText = document.getElementById("progress-text");
        if (progressContainer && progressBar && progressText) {
            progressContainer.style.display = "block";
            progressBar.style.width = "0%";
            progressText.textContent = "0% - Starting...";
        }

        if (status) status.textContent = "Analyzing codebase with metrics...";

        console.log("📤 Invoking analyze_with_metrics with path:", selectedDir);
        
        let analyzedGraph: AnalyzedGraph;
        try {
            // Use analyze_with_metrics to get both graph and metrics
            const response = await invoke("analyze_with_metrics", {
                path: selectedDir
            });
            
            console.log("📥 Raw response from analyze_with_metrics:", response);
            analyzedGraph = response as AnalyzedGraph;
        } catch (invokeError) {
            console.error("❌ Tauri invoke error:", invokeError);
            // Try fallback to parse_codebase if analyze_with_metrics fails
            console.log("⚠️ Falling back to parse_codebase...");
            
            const parseResult = await invoke("parse_codebase", {
                path: selectedDir
            });
            console.log("📥 Parse result:", parseResult);
            
            // Generate graph separately
            const graphData = await invoke("generate_graph");
            console.log("📥 Graph data:", graphData);
            
            // Create a minimal AnalyzedGraph structure
            analyzedGraph = {
                graphData: graphData as GraphData,
                metrics: [],
                summary: {
                    totalNodes: (graphData as any)?.nodes?.length || 0,
                    totalEdges: (graphData as any)?.links?.length || 0,
                    numCommunities: 0,
                    modularity: 0,
                    avgComplexity: 0,
                    highRiskCount: 0,
                    circularDependencies: 0
                }
            };
        }

        // Validate the response
        if (!analyzedGraph || !analyzedGraph.graphData) {
            console.error("❌ Invalid response structure:", analyzedGraph);
            throw new Error("Invalid response from analyze_with_metrics");
        }

        // Store metrics globally (with fallback to empty array)
        currentMetrics = analyzedGraph.metrics || [];
        console.log(`✅ Received ${currentMetrics.length} node metrics`);
        
        // Log summary if available
        if (analyzedGraph.summary) {
            console.log("📊 Analysis Summary:", analyzedGraph.summary);
            console.log(`  Communities: ${analyzedGraph.summary.numCommunities || 0}`);
            console.log(`  High Risk Nodes: ${analyzedGraph.summary.highRiskCount || 0}`);
            if (typeof analyzedGraph.summary.avgComplexity === 'number') {
                console.log(`  Avg Complexity: ${analyzedGraph.summary.avgComplexity.toFixed(2)}`);
            }
        } else {
            console.log("⚠️ No analysis summary available");
        }

        if (status) {
            const nodeCount = analyzedGraph.summary?.totalNodes || 
                             analyzedGraph.graphData?.nodes?.length || 0;
            status.textContent = `Analyzed ${nodeCount} nodes with metrics`;
        }

        // Generate the graph data with metrics attached
        const graphData = analyzedGraph.graphData;
        
        // Attach metrics to nodes
        if (graphData?.nodes && currentMetrics.length > 0) {
            graphData.nodes.forEach(node => {
                const nodeMetrics = findMetricsForNode(node.id, currentMetrics);
                if (nodeMetrics) {
                    node.metrics = nodeMetrics;
                }
            });
            const nodesWithMetrics = graphData.nodes.filter(n => n.metrics).length;
            console.log(`✅ Attached metrics to ${nodesWithMetrics} nodes`);
        }
        
        // Send to graph visualization
        if (graph3d && graphData) {
            graph3d.loadData(graphData);
            console.log("✅ Graph loaded with metrics");
        }
        
        // Build file tree from graph nodes (existing functionality)
        const files = buildFileTreeFromGraph(graphData);
        if (files && files.length > 0) {
            eventBus.emit('files:updated', { files });
            console.log("📁 File tree updated with", files.length, "root items");
            
            // Automatically show the file tree panel if not visible
            const fileTreeState = panelsState.get()['file-tree'];
            if (!fileTreeState?.visible) {
                panelManager?.dock('file-tree', 'left');
            }
        }
        
        // Emit metrics update event for other components
        if (currentMetrics.length > 0 || analyzedGraph.summary) {
            eventBus.emit('metrics:updated', { 
                metrics: currentMetrics, 
                summary: analyzedGraph.summary || {}
            });
        }
        
    } catch (error) {
        console.error("❌ Failed to analyze codebase:", error);
        const status = document.getElementById("status");
        if (status) status.textContent = "Failed to analyze codebase";
    }
}

// Helper function to build file tree from graph data
function buildFileTreeFromGraph(graphData: GraphData): any[] {
    if (!graphData || !graphData.nodes) {
        return [];
    }
    
    // This is a simplified version - you might want to enhance this
    const fileNodes = graphData.nodes.filter(n => 
        n && (n.type === 'file' || n.nodeType === 'file')
    );
    
    return fileNodes.map(node => ({
        name: node.name || 'Unknown',
        path: node.filePath || node.id || '',
        type: 'file',
        children: null
    }));
}

async function connectToNeo4j() {
    try {
        const status = document.getElementById("status");
        if (status) status.textContent = "Connecting to Neo4j...";

        const connected = await invoke("connect_neo4j", {
            uri: "bolt://localhost:7687",
            username: "neo4j",
            password: "password" // TODO: Add proper config UI
        });

        if (status) status.textContent = connected ? "Connected to Neo4j" : "Failed to connect";
        console.log(connected ? "✅ Neo4j connected" : "❌ Neo4j connection failed");
    } catch (error) {
        console.error("❌ Failed to connect to Neo4j:", error);
    }
}

async function generateGraph() {
    try {
        const status = document.getElementById("status");
        if (status) status.textContent = "Generating graph...";

        const graphData = await invoke("generate_graph") as GraphData;
        
        // Debug log the received data
        console.log("📊 Received graph data:", graphData);
        
        if (graph3d && graphData) {
            graph3d.loadData(graphData);
            
            // Update status
            if (status) status.textContent = `Graph generated: ${graphData.nodes.length} nodes, ${graphData.links.length} links`;
            
            // Update counters
            const nodeCount = document.getElementById("node-count");
            const linkCount = document.getElementById("link-count");
            if (nodeCount) nodeCount.textContent = `Nodes: ${graphData.nodes.length}`;
            if (linkCount) linkCount.textContent = `Links: ${graphData.links.length}`;
            
            console.log("✅ Graph generated");
            console.log(`   Nodes: ${graphData.nodes.length}`);
            console.log(`   Links: ${graphData.links.length}`);
        }
    } catch (error) {
        console.error("❌ Failed to generate graph:", error);
        const status = document.getElementById("status");
        if (status) status.textContent = "Failed to generate graph";
    }
}

async function resetApp() {
    try {
        const status = document.getElementById("status");
        if (status) status.textContent = "Resetting app...";

        // Call reset command on backend
        await invoke("reset_app");
        
        // Clear the 3D graph
        if (graph3d) {
            graph3d.loadData({ nodes: [], links: [] });
        }
        
        // Reset UI elements
        const nodeCount = document.getElementById("node-count");
        const linkCount = document.getElementById("link-count");
        if (nodeCount) nodeCount.textContent = "Nodes: 0";
        if (linkCount) linkCount.textContent = "Links: 0";
        
        if (status) status.textContent = "App reset - Ready";
        console.log("✅ App reset successfully");
        
        // Reload the page to fully clear any stuck state
        setTimeout(() => {
            window.location.reload();
        }, 500);
    } catch (error) {
        console.error("❌ Failed to reset app:", error);
        const status = document.getElementById("status");
        if (status) status.textContent = "Failed to reset app";
    }
}

// Helper functions for panels
function togglePanels() {
    const currentPanels = panelsState.get();
    
    // Check if any panel is visible
    const anyVisible = Object.values(currentPanels).some(p => p.visible && p.mode === 'docked');
    
    if (anyVisible) {
        // Hide all panels
        Object.keys(currentPanels).forEach(id => {
            panelManager?.close(id);
        });
    } else {
        // Show default layout (explore)
        switchLayout('explore');
    }
}

function switchLayout(layoutName: 'explore' | 'inspect' | 'debug') {
    const layout = layoutPresets[layoutName];
    if (!layout) return;
    
    // Apply the layout
    Object.entries(layout.panels).forEach(([id, state]) => {
        panelsState.setKey(id, state);
        if (state.visible) {
            if (state.mode === 'docked' && state.area) {
                panelManager?.dock(id, state.area);
            } else if (state.mode === 'floating') {
                panelManager?.float(id, state.pos);
            }
        } else {
            panelManager?.close(id);
        }
    });
    
    // Update status
    const status = document.getElementById("status");
    if (status) {
        status.textContent = `Layout: ${layoutName}`;
        setTimeout(() => {
            status.textContent = "Ready";
        }, 2000);
    }
}

// Setup additional UI event handlers
function setupUIEventHandlers() {
    // Panel toggle event
    eventBus.on('panel:toggle', ({ panelId }) => {
        panelManager?.toggle(panelId);
    });
    
    // Layout switch event
    eventBus.on('layout:switch', ({ preset }) => {
        switchLayout(preset as 'explore' | 'inspect' | 'debug');
    });
    
    // Toggle all panels
    eventBus.on('panels:toggle-all', () => {
        togglePanels();
    });
    
    // Graph reset view
    eventBus.on('graph:reset-view', () => {
        graph3d?.resetView();
    });

    // Additional keyboard shortcuts
    document.addEventListener("keydown", (e) => {
        // Help shortcut (?)
        if (e.key === "?" && !e.ctrlKey && !e.altKey) {
            const palette = document.querySelector('og-command-palette') as any;
            if (palette) {
                palette.open = true;
            }
        }
        
        // Reset view (R)
        if (e.key === "r" && !e.ctrlKey && !e.altKey) {
            graph3d?.resetView();
        }
        
        // Parse codebase (Ctrl+O)
        if (e.ctrlKey && e.key === "o") {
            e.preventDefault();
            parseCodebase();
        }
        
        // Generate graph (Ctrl+G)
        if (e.ctrlKey && e.key === "g") {
            e.preventDefault();
            generateGraph();
        }
    });
}

// Start the app when DOM is ready
document.addEventListener("DOMContentLoaded", initializeApp);
