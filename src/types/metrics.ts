/**
 * TypeScript interfaces for metrics data from Rust backend
 * Matches the structures in og-types crate
 */

export interface RawMetrics {
  pagerankImports: number;
  pagerankCalls?: number;
  indegree: number;
  outdegree: number;
  kCore: number;
  clustering: number;
  betweenness: number;
  churn: number;
  complexity: number;
  owners: number;
  coverage: number;
}

export interface NormalizedMetrics {
  pagerankImports: number;
  pagerankCalls?: number;
  indegree: number;
  kCore: number;
  clustering: number;
  betweenness: number;
  churn: number;
  complexity: number;
  owners: number;
  coverage: number;
}

export interface UINodeMetricsV1 {
  path: string;
  name: string;
  nodeType: string;
  community: number;
  importance: number;
  risk: number;
  chokepoint: number;
  payoff: number;
  raw: RawMetrics;
  normalized: NormalizedMetrics;
  version?: number;
}

export interface AnalysisSummary {
  totalNodes: number;
  totalEdges: number;
  numCommunities: number;
  modularity: number;
  avgComplexity: number;
  highRiskCount: number;
  circularDependencies: number;
}

export interface AnalyzedGraph {
  graphData: GraphData;
  metrics: UINodeMetricsV1[];
  summary: AnalysisSummary;
}

export interface GraphNode {
  id: string;
  name: string;
  type: string;
  filePath?: string;
  nodeType?: string;
  fileType?: string;
  // Add metrics reference
  metrics?: UINodeMetricsV1;
}

export interface GraphLink {
  source: string;
  target: string;
  type?: string;
  linkType?: string;
  value?: number;
  strength?: number;
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

// Helper function to find metrics for a node
export function findMetricsForNode(
  nodeId: string,
  metrics: UINodeMetricsV1[]
): UINodeMetricsV1 | undefined {
  // Node ID might be the path or a variation of it
  return metrics.find(m => 
    m.path === nodeId || 
    m.path.endsWith(nodeId) ||
    nodeId.endsWith(m.path)
  );
}

// Color palette for communities
export const COMMUNITY_COLORS = [
  '#3498db', // Blue
  '#e74c3c', // Red
  '#2ecc71', // Green
  '#f39c12', // Orange
  '#9b59b6', // Purple
  '#1abc9c', // Turquoise
  '#34495e', // Dark Gray
  '#f1c40f', // Yellow
  '#e67e22', // Carrot
  '#95a5a6', // Light Gray
];

export function getCommunityColor(communityId: number): string {
  return COMMUNITY_COLORS[communityId % COMMUNITY_COLORS.length];
}

// Scale functions for visual encoding
export function scaleNodeSize(importance: number): number {
  // Scale from 0.5x to 2x based on importance (0-1)
  return 0.5 + importance * 1.5;
}

export function shouldShowRiskIndicator(risk: number): boolean {
  return risk > 0.7;
}

export function shouldShowChokepointGlow(chokepoint: number): boolean {
  return chokepoint > 0.7;
}