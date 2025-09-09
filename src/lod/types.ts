// LOD Types matching Rust backend

export enum LodLevel {
  L1Packages = "l1Packages",
  L2Files = "l2Files", 
  L4Functions = "l4Functions"
}

export interface GraphNode {
  id: string;
  label: string;
  type: string;
  parentId?: string;
  filePath?: string;
  hasChildren: boolean;
  expanded: boolean;
  level: number;
}

export interface GraphEdge {
  id: string;
  source: string;
  target: string;
  kind: string;
  weight?: number;
  bundled: boolean;
}

export interface GraphPayload {
  nodes: GraphNode[];
  edges: GraphEdge[];
  lodLevel: LodLevel;
}

export interface GraphDelta {
  addNodes: GraphNode[];
  addEdges: GraphEdge[];
  removeNodeIds: string[];
  removeEdgeIds: string[];
  restoreBundles: GraphEdge[];
}