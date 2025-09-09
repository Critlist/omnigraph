import { GraphNode, GraphEdge, GraphPayload, GraphDelta, LodLevel } from './types';

export class LodState {
  private currentLod: LodLevel = LodLevel.L2Files;
  private expandedNodes: Set<string> = new Set();
  private nodeIndex: Map<string, GraphNode> = new Map();
  private edgeIndex: Map<string, GraphEdge> = new Map();
  private nodes: GraphNode[] = [];
  private edges: GraphEdge[] = [];

  getCurrentLod(): LodLevel {
    return this.currentLod;
  }

  setLod(lod: LodLevel) {
    this.currentLod = lod;
    this.expandedNodes.clear();
  }

  getNodes(): GraphNode[] {
    return this.nodes;
  }

  getEdges(): GraphEdge[] {
    return this.edges;
  }

  isExpanded(nodeId: string): boolean {
    return this.expandedNodes.has(nodeId);
  }

  resetToGraph(payload: GraphPayload) {
    console.log('[LOD] Resetting to graph at level:', payload.lodLevel);
    
    this.currentLod = payload.lodLevel;
    this.nodes = [...payload.nodes];
    this.edges = [...payload.edges];
    
    // Rebuild indices
    this.nodeIndex.clear();
    this.edgeIndex.clear();
    
    for (const node of this.nodes) {
      this.nodeIndex.set(node.id, node);
      if (node.expanded) {
        this.expandedNodes.add(node.id);
      }
    }
    
    for (const edge of this.edges) {
      this.edgeIndex.set(edge.id, edge);
    }
  }

  applyDelta(delta: GraphDelta) {
    console.log('[LOD] Applying delta:', delta);
    
    // Remove nodes
    for (const nodeId of delta.removeNodeIds) {
      const index = this.nodes.findIndex(n => n.id === nodeId);
      if (index >= 0) {
        this.nodes.splice(index, 1);
        this.nodeIndex.delete(nodeId);
        this.expandedNodes.delete(nodeId);
      }
    }
    
    // Remove edges
    for (const edgeId of delta.removeEdgeIds) {
      const index = this.edges.findIndex(e => e.id === edgeId);
      if (index >= 0) {
        this.edges.splice(index, 1);
        this.edgeIndex.delete(edgeId);
      }
    }
    
    // Add new nodes
    for (const node of delta.addNodes) {
      if (!this.nodeIndex.has(node.id)) {
        this.nodes.push(node);
        this.nodeIndex.set(node.id, node);
        if (node.expanded) {
          this.expandedNodes.add(node.id);
        }
      }
    }
    
    // Add new edges
    for (const edge of delta.addEdges) {
      if (!this.edgeIndex.has(edge.id)) {
        this.edges.push(edge);
        this.edgeIndex.set(edge.id, edge);
      }
    }
    
    // Restore bundled edges
    for (const edge of delta.restoreBundles) {
      if (!this.edgeIndex.has(edge.id)) {
        this.edges.push(edge);
        this.edgeIndex.set(edge.id, edge);
      }
    }
  }

  expandNode(nodeId: string) {
    this.expandedNodes.add(nodeId);
    const node = this.nodeIndex.get(nodeId);
    if (node) {
      node.expanded = true;
    }
  }

  collapseNode(nodeId: string) {
    this.expandedNodes.delete(nodeId);
    const node = this.nodeIndex.get(nodeId);
    if (node) {
      node.expanded = false;
    }
  }

  // Convert to format expected by 3d-force-graph
  toGraphData() {
    return {
      nodes: this.nodes.map(n => ({
        id: n.id,
        name: n.label,
        type: n.type,
        val: n.level === 1 ? 10 : n.level === 2 ? 5 : 2,
        color: this.getNodeColor(n.type),
        hasChildren: n.hasChildren,
        expanded: n.expanded
      })),
      links: this.edges.map(e => ({
        source: e.source,
        target: e.target,
        type: e.kind,
        value: e.weight || 1,
        bundled: e.bundled
      }))
    };
  }

  private getNodeColor(type: string): string {
    switch(type) {
      case 'package': return '#ff6b6b';
      case 'file': return '#4ecdc4';
      case 'function': return '#45b7d1';
      case 'class': return '#96ceb4';
      default: return '#95a5a6';
    }
  }
}

// Global instance
export const lodState = new LodState();