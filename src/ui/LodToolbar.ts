import { invoke } from '@tauri-apps/api/core';
import { LodLevel, GraphPayload } from '../lod/types';
import { lodState } from '../lod/state';

export class LodToolbar {
  private container: HTMLElement;
  private onGraphUpdate: (data: any) => void;

  constructor(onGraphUpdate: (data: any) => void) {
    this.onGraphUpdate = onGraphUpdate;
    this.container = this.createToolbar();
    this.attachToDOM();
  }

  private createToolbar(): HTMLElement {
    const toolbar = document.createElement('div');
    toolbar.id = 'lod-toolbar';
    toolbar.className = 'lod-toolbar';
    toolbar.innerHTML = `
      <div class="lod-buttons">
        <button class="lod-btn" data-lod="${LodLevel.L1Packages}" title="Package View">
          📦 Packages
        </button>
        <button class="lod-btn active" data-lod="${LodLevel.L2Files}" title="File View">
          📄 Files
        </button>
        <button class="lod-btn" data-lod="${LodLevel.L4Functions}" title="Function View">
          🔧 Functions
        </button>
      </div>
      <div class="lod-info">
        <span id="lod-node-count">0 nodes</span>
        <span id="lod-edge-count">0 edges</span>
      </div>
    `;

    // Add styles
    this.addStyles();

    // Add event listeners
    toolbar.querySelectorAll('.lod-btn').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const target = e.target as HTMLElement;
        const lod = target.dataset.lod as LodLevel;
        if (lod) {
          this.switchLod(lod);
        }
      });
    });

    return toolbar;
  }

  private addStyles() {
    if (document.getElementById('lod-toolbar-styles')) return;

    const style = document.createElement('style');
    style.id = 'lod-toolbar-styles';
    style.textContent = `
      .lod-toolbar {
        position: fixed;
        top: 70px;
        left: 20px;
        background: rgba(30, 30, 40, 0.95);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 8px;
        padding: 10px;
        display: flex;
        flex-direction: column;
        gap: 10px;
        z-index: 1000;
        backdrop-filter: blur(10px);
      }

      .lod-buttons {
        display: flex;
        gap: 5px;
      }

      .lod-btn {
        padding: 8px 12px;
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 4px;
        color: #a0a0a0;
        cursor: pointer;
        transition: all 0.3s ease;
        font-size: 12px;
        display: flex;
        align-items: center;
        gap: 5px;
      }

      .lod-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
        border-color: rgba(255, 255, 255, 0.2);
      }

      .lod-btn.active {
        background: rgba(78, 205, 196, 0.2);
        color: #4ecdc4;
        border-color: #4ecdc4;
      }

      .lod-info {
        display: flex;
        justify-content: space-between;
        font-size: 11px;
        color: #666;
        padding: 5px;
      }

      .node-expand-btn {
        position: absolute;
        width: 20px;
        height: 20px;
        background: rgba(78, 205, 196, 0.9);
        border: 1px solid #4ecdc4;
        border-radius: 3px;
        color: white;
        font-size: 14px;
        font-weight: bold;
        cursor: pointer;
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1001;
        transition: all 0.2s ease;
      }

      .node-expand-btn:hover {
        background: #4ecdc4;
        transform: scale(1.1);
      }

      .node-expand-btn.collapse {
        background: rgba(255, 107, 107, 0.9);
        border-color: #ff6b6b;
      }

      .node-expand-btn.collapse:hover {
        background: #ff6b6b;
      }
    `;
    document.head.appendChild(style);
  }

  private attachToDOM() {
    // Wait for DOM to be ready
    if (document.body) {
      document.body.appendChild(this.container);
    } else {
      document.addEventListener('DOMContentLoaded', () => {
        document.body.appendChild(this.container);
      });
    }
  }

  private async switchLod(lod: LodLevel) {
    console.log('[LOD] Switching to level:', lod);
    
    // Update UI
    this.container.querySelectorAll('.lod-btn').forEach(btn => {
      btn.classList.remove('active');
      if (btn.getAttribute('data-lod') === lod) {
        btn.classList.add('active');
      }
    });

    try {
      // Call backend
      const payload = await invoke<GraphPayload>('get_graph_at_lod', { lod });
      
      // Update state
      lodState.setLod(lod);
      lodState.resetToGraph(payload);
      
      // Update graph visualization
      const graphData = lodState.toGraphData();
      this.onGraphUpdate(graphData);
      
      // Update info
      this.updateInfo(payload.nodes.length, payload.edges.length);
    } catch (error) {
      console.error('[LOD] Failed to switch level:', error);
    }
  }

  public updateInfo(nodeCount: number, edgeCount: number) {
    const nodeEl = document.getElementById('lod-node-count');
    const edgeEl = document.getElementById('lod-edge-count');
    
    if (nodeEl) nodeEl.textContent = `${nodeCount} nodes`;
    if (edgeEl) edgeEl.textContent = `${edgeCount} edges`;
  }

  public async expandNode(nodeId: string) {
    console.log('[LOD] Expanding node:', nodeId);
    
    // Determine target LOD based on current level
    let targetLod = lodState.getCurrentLod();
    const node = lodState.getNodes().find(n => n.id === nodeId);
    
    if (node) {
      if (node.level === 1) targetLod = LodLevel.L2Files;
      else if (node.level === 2) targetLod = LodLevel.L4Functions;
    }

    try {
      const delta = await invoke('expand_node', { 
        nodeId, 
        targetLod 
      });
      
      lodState.expandNode(nodeId);
      lodState.applyDelta(delta as any);
      
      const graphData = lodState.toGraphData();
      this.onGraphUpdate(graphData);
      
      // Update counts
      const nodes = lodState.getNodes();
      const edges = lodState.getEdges();
      this.updateInfo(nodes.length, edges.length);
    } catch (error) {
      console.error('[LOD] Failed to expand node:', error);
    }
  }

  public async collapseNode(nodeId: string) {
    console.log('[LOD] Collapsing node:', nodeId);
    
    try {
      const delta = await invoke('collapse_node', { nodeId });
      
      lodState.collapseNode(nodeId);
      lodState.applyDelta(delta as any);
      
      const graphData = lodState.toGraphData();
      this.onGraphUpdate(graphData);
      
      // Update counts
      const nodes = lodState.getNodes();
      const edges = lodState.getEdges();
      this.updateInfo(nodes.length, edges.length);
    } catch (error) {
      console.error('[LOD] Failed to collapse node:', error);
    }
  }
}

export default LodToolbar;