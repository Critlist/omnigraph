/**
 * Clean 3D Graph Visualization - NO CSP, NO NONCE, JUST FREEDOM!
 */

// @ts-ignore - TypeScript doesn't recognize the default export properly
import ForceGraph3D from '3d-force-graph';
import * as THREE from 'three';

import { UINodeMetricsV1 } from '../types/metrics';

export interface GraphNode {
    id: string;
    name: string;
    type: string; // Now accepts any node type
    nodeType?: string; // Alternative type field
    filePath?: string;
    size?: number; // Weight/importance of the node
    color?: string;
    group?: number;
    connections?: number; // Number of connections
    metrics?: UINodeMetricsV1; // Attached metrics data
}

export interface GraphLink {
    source: string;
    target: string;
    type?: string; // Made optional to match backend
    linkType?: string; // Alternative from backend
    strength?: number;
    value?: number; // Alternative from backend
}

export interface GraphData {
    nodes: GraphNode[];
    links: GraphLink[];
    stats?: {
        fileCount: number;
        nodeCount: number;
        linkCount: number;
    };
}

export class Graph3DVisualization {
    private graph: any;
    private container: HTMLElement;
    private selectedNode: GraphNode | null = null;
    private resizeHandler: (() => void) | null = null;
    private resizeObserver: ResizeObserver | null = null;

    constructor(containerId: string) {
        const container = document.getElementById(containerId);
        if (!container) {
            throw new Error(`Container with id ${containerId} not found`);
        }
        this.container = container;
        this.initializeGraph();
        this.setupResizeListener();
    }

    private setupResizeListener(): void {
        let resizeTimeout: any;
        
        this.resizeHandler = () => {
            // Debounce resize events
            clearTimeout(resizeTimeout);
            resizeTimeout = setTimeout(() => {
                this.handleResize();
            }, 250);
        };
        
        window.addEventListener('resize', this.resizeHandler);
        
        // Also observe container size changes
        this.resizeObserver = new ResizeObserver(() => {
            this.resizeHandler!();
        });
        this.resizeObserver.observe(this.container);
    }
    
    private handleResize(): void {
        if (this.graph) {
            // Get the new container dimensions
            const width = this.container.clientWidth;
            const height = this.container.clientHeight;
            
            // Update the graph dimensions
            this.graph
                .width(width)
                .height(height);
            
            // Force re-render
            this.graph.refresh();
            
            console.log(`📐 Canvas resized to ${width}x${height}`);
        }
    }
    
    private initializeGraph(): void {
        // Get initial container dimensions
        const width = this.container.clientWidth;
        const height = this.container.clientHeight;
        
        // Create the 3D force graph - NO RESTRICTIONS!
        // @ts-ignore - ForceGraph3D callable issue
        this.graph = (ForceGraph3D as any)()(this.container)
            .width(width)
            .height(height)
            .backgroundColor('#0a0a0a')
            .showNavInfo(false)
            .linkOpacity(0.5)
            .linkWidth(1)
            .linkDirectionalParticles(2)
            .linkDirectionalParticleSpeed(0.005)
            .nodeLabel('name')
            .nodeAutoColorBy('type')
            .nodeThreeObject((node: any) => {
                // All nodes are spheres, size based on weight
                const size = this.getNodeSize(node);
                const geometry = new THREE.SphereGeometry(size, 32, 32);
                const material = new THREE.MeshLambertMaterial({
                    color: this.getNodeColor(node),
                    transparent: true,
                    opacity: 0.9,
                });
                return new THREE.Mesh(geometry, material);
            })
            .onNodeClick(this.handleNodeClick.bind(this))
            .onNodeHover(this.handleNodeHover.bind(this));

        // Add lights - because we can!
        const scene = this.graph.scene();
        scene.add(new THREE.AmbientLight(0xffffff, 0.6));
        scene.add(new THREE.DirectionalLight(0xffffff, 0.6));

        // Add grid helper - why not?
        const gridHelper = new THREE.GridHelper(1000, 20, 0x444444, 0x222222);
        scene.add(gridHelper);
    }

    private getNodeSize(node: any): number {
        // Base size for each node type
        const baseSizes: Record<string, number> = {
            file: 8,
            module: 7,
            class: 6,
            function: 4,
            interface: 5,
            type_alias: 3,
            enum: 4,
            variable: 3,
            import: 2,
            export: 2,
            method: 3,
            property: 2
        };
        
        // Get base size or default
        let baseSize = baseSizes[node.type] || 3;
        
        // Calculate weight multiplier based on connections
        // The graph library adds these properties after data is loaded
        let weight = 1;
        
        // If node has explicit size property, use it as a multiplier
        if (node.size && node.size > 0) {
            weight = Math.sqrt(node.size / 10); // Square root for better visual scaling
        }
        
        // If node has connection data (added by force-graph), use it
        if (node.__degree) {
            // __degree is the number of connections
            weight = Math.max(weight, 1 + Math.log(node.__degree + 1) * 0.3);
        }
        
        // Return final size with min/max bounds
        return Math.min(Math.max(baseSize * weight, 2), 20);
    }

    private getNodeColor(node: any): string {
        const colors: Record<string, string> = {
            file: '#4A90E2',
            module: '#7B68EE',
            class: '#50C878',
            function: '#FFB347',
            method: '#FFA07A',
            variable: '#87CEEB',
            import: '#DDA0DD',
            export: '#F0E68C',
            interface: '#98D8C8',
            property: '#F7DC6F',
            type_alias: '#BB8FCE',
            enum: '#85C1E2',
        };
        return colors[node.type] || '#7ED321';
    }

    private handleNodeClick(node: any): void {
        this.selectedNode = node;

        // Zoom to node
        const distance = 100;
        const distRatio = 1 + distance / Math.hypot(node.x, node.y, node.z);

        this.graph.cameraPosition(
            { x: node.x * distRatio, y: node.y * distRatio, z: node.z * distRatio },
            node,
            1000
        );

        // Emit event for file opening (Tauri will handle this)
        if (node.filePath) {
            window.dispatchEvent(new CustomEvent('open-file', {
                detail: { path: node.filePath }
            }));
        }
    }

    private handleNodeHover(node: any): void {
        this.container.style.cursor = node ? 'pointer' : 'default';
    }

    public loadData(data: GraphData): void {
        // Just load the data - no validation, no CSP, no bullshit
        this.graph.graphData(data);
    }

    public updateData(data: GraphData): void {
        // Hot reload data
        this.graph.graphData(data);
    }

    public highlightPath(nodeIds: string[]): void {
        // Highlight a path through the graph
        const highlightNodes = new Set(nodeIds);

        this.graph.nodeColor((node: any) =>
            highlightNodes.has(node.id) ? '#FF0000' : this.getNodeColor(node)
        );
    }

    public resetView(): void {
        this.graph.cameraPosition(
            { x: 0, y: 0, z: 300 },
            { x: 0, y: 0, z: 0 },
            1000
        );
    }

    public dispose(): void {
        // Clean up event listeners
        if (this.resizeHandler) {
            window.removeEventListener('resize', this.resizeHandler);
            this.resizeHandler = null;
        }
        
        if (this.resizeObserver) {
            this.resizeObserver.disconnect();
            this.resizeObserver = null;
        }
        
        // Clean up the graph
        if (this.graph) {
            this.graph._destructor();
            this.graph = null;
        }
        
        console.log("🧹 Graph visualization disposed and cleaned up");
    }
}