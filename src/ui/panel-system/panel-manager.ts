import { panelsState, type PanelState, type DockArea } from '../../state/panels';
import { eventBus } from '../../state/events';
import '../components/og-panel';
import '../components/og-float-panel';
import '../components/og-file-tree';
import '../components/og-properties-panel';
import type { OGPanel } from '../components/og-panel';
import type { OGFloatPanel } from '../components/og-float-panel';

export interface PanelConfig {
  id: string;
  title: string;
  component?: () => HTMLElement;
  defaultArea?: DockArea;
  canFloat?: boolean;
  canClose?: boolean;
  canCollapse?: boolean;
}

export class PanelManager {
  private registry = new Map<string, PanelConfig>();
  private mounted = new Map<string, HTMLElement>();
  private nextZIndex = 100;

  constructor() {
    this.setupEventListeners();
    this.initializePanels();
  }

  private setupEventListeners() {
    eventBus.on('panel:docked', ({ panelId, area }) => {
      this.dock(panelId, area);
    });

    eventBus.on('panel:floated', ({ panelId }) => {
      this.float(panelId);
    });

    eventBus.on('panel:closed', ({ panelId }) => {
      this.close(panelId);
    });

    eventBus.on('panel:detached', ({ panelId }) => {
      this.float(panelId);
    });

    eventBus.on('panel:collapsed', ({ panelId, collapsed }) => {
      const state = panelsState.get()[panelId];
      if (state) {
        panelsState.setKey(panelId, { ...state, collapsed });
      }
    });

    eventBus.on('panel:resized', ({ panelId, size }) => {
      const state = panelsState.get()[panelId];
      if (state) {
        panelsState.setKey(panelId, { ...state, size });
      }
    });

    eventBus.on('panel:drag', ({ panelId, pos }) => {
      const state = panelsState.get()[panelId];
      if (state && state.mode === 'floating') {
        panelsState.setKey(panelId, { ...state, pos });
      }
    });
  }

  private initializePanels() {
    // Register default panels
    this.register({
      id: 'file-tree',
      title: 'Explorer',
      defaultArea: 'left',
      component: () => this.createFileTreePanel()
    });

    this.register({
      id: 'properties',
      title: 'Properties',
      defaultArea: 'right',
      component: () => this.createPropertiesPanel()
    });

    this.register({
      id: 'terminal',
      title: 'Output',
      defaultArea: 'bottom',
      component: () => this.createTerminalPanel()
    });
  }

  register(config: PanelConfig) {
    this.registry.set(config.id, config);
  }

  mount(panelId: string, container: HTMLElement | null = null) {
    const config = this.registry.get(panelId);
    if (!config) {
      console.warn(`Panel ${panelId} not registered`);
      return;
    }

    const state = panelsState.get()[panelId];
    if (!state) {
      console.warn(`Panel ${panelId} has no state`);
      return;
    }

    // Unmount existing instance if any
    this.unmount(panelId);

    let panelElement: HTMLElement;

    if (state.mode === 'floating') {
      panelElement = this.createFloatingPanel(config, state);
      const floatingLayer = document.getElementById('floating-layer');
      if (floatingLayer) {
        floatingLayer.appendChild(panelElement);
      }
    } else if (state.mode === 'docked') {
      panelElement = this.createDockedPanel(config, state);
      const targetContainer = container || this.getDockedContainer(state.area);
      if (targetContainer) {
        targetContainer.innerHTML = '';
        targetContainer.appendChild(panelElement);
        targetContainer.style.display = state.visible ? '' : 'none';
        
        // Show splitter
        const splitter = this.getSplitter(state.area);
        if (splitter) {
          splitter.style.display = state.visible ? '' : 'none';
        }
      }
    } else {
      return;
    }

    this.mounted.set(panelId, panelElement);
    
    // Add panel content
    if (config.component) {
      const content = config.component();
      const contentSlot = panelElement.querySelector('slot') || panelElement;
      if (contentSlot instanceof HTMLSlotElement) {
        contentSlot.assignedElements().forEach(el => el.remove());
      }
      panelElement.appendChild(content);
    }

    eventBus.emit('panel:mounted', { panelId });
  }

  unmount(panelId: string) {
    const element = this.mounted.get(panelId);
    if (element) {
      element.remove();
      this.mounted.delete(panelId);
    }
  }

  dock(panelId: string, area: DockArea) {
    const state = panelsState.get()[panelId];
    if (!state) return;

    panelsState.setKey(panelId, { 
      ...state, 
      mode: 'docked', 
      area,
      visible: true 
    });

    this.mount(panelId);
  }

  float(panelId: string, position?: { x: number; y: number }) {
    const state = panelsState.get()[panelId];
    if (!state) return;

    // Calculate center position if not provided
    if (!position) {
      const viewport = {
        w: window.innerWidth,
        h: window.innerHeight
      };
      position = {
        x: (viewport.w - (state.size?.w || 400)) / 2,
        y: (viewport.h - (state.size?.h || 300)) / 2
      };
    }

    panelsState.setKey(panelId, { 
      ...state, 
      mode: 'floating', 
      pos: position,
      z: this.getNextZIndex(),
      visible: true
    });

    this.mount(panelId);
  }

  close(panelId: string) {
    const state = panelsState.get()[panelId];
    if (!state) return;

    panelsState.setKey(panelId, { ...state, visible: false });
    this.unmount(panelId);

    // Hide docked container and splitter
    if (state.mode === 'docked') {
      const container = this.getDockedContainer(state.area);
      if (container) {
        container.style.display = 'none';
      }
      const splitter = this.getSplitter(state.area);
      if (splitter) {
        splitter.style.display = 'none';
      }
    }
  }

  toggle(panelId: string) {
    const state = panelsState.get()[panelId];
    if (!state) return;

    if (state.visible) {
      this.close(panelId);
    } else {
      if (state.mode === 'docked') {
        this.dock(panelId, state.area!);
      } else {
        this.float(panelId);
      }
    }
  }

  private createDockedPanel(config: PanelConfig, state: PanelState): HTMLElement {
    const panel = document.createElement('og-panel') as OGPanel;
    panel.title = config.title;
    panel.panelId = config.id;
    panel.canFloat = config.canFloat ?? true;
    panel.canClose = config.canClose ?? true;
    panel.canCollapse = config.canCollapse ?? true;
    return panel;
  }

  private createFloatingPanel(config: PanelConfig, state: PanelState): HTMLElement {
    const panel = document.createElement('og-float-panel') as OGFloatPanel;
    panel.title = config.title;
    panel.panelId = config.id;
    panel.position = state.pos || { x: 100, y: 100 };
    panel.size = state.size || { w: 400, h: 300 };
    panel.zIndex = state.z || this.getNextZIndex();
    return panel;
  }

  private getDockedContainer(area?: DockArea): HTMLElement | null {
    if (!area) return null;
    return document.querySelector(`.og-panel-${area}`);
  }

  private getSplitter(area?: DockArea): HTMLElement | null {
    if (!area) return null;
    const splitterClass = area === 'bottom' 
      ? '.og-splitter-h-bottom' 
      : `.og-splitter-v-${area}`;
    return document.querySelector(splitterClass);
  }

  private getNextZIndex(): number {
    const panels = Object.values(panelsState.get());
    const maxZ = Math.max(100, ...panels.map(p => p.z || 100));
    return maxZ + 1;
  }

  // Panel content creators
  private createFileTreePanel(): HTMLElement {
    const fileTree = document.createElement('og-file-tree');
    return fileTree;
  }

  private createPropertiesPanel(): HTMLElement {
    const propertiesPanel = document.createElement('og-properties-panel');
    return propertiesPanel;
  }

  private createTerminalPanel(): HTMLElement {
    const content = document.createElement('div');
    content.innerHTML = `
      <div style="padding: 8px; font-family: monospace; font-size: 12px; color: #2ecc71;">
        <div>🚀 Omnigraph initialized</div>
        <div>📊 Graph engine ready</div>
        <div>✅ Parsing complete: 42 files processed</div>
        <div>📈 Generated 156 nodes and 234 edges</div>
        <div style="margin-top: 8px; color: rgba(255,255,255,0.5);">
          > Ready for input...
        </div>
      </div>
    `;
    return content;
  }
}