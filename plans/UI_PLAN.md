# Omnigraph Hybrid UI — Implementation Plan

## 1) Overview

Omnigraph's hybrid UI model combines **Docked Panels** (persistent workspace tools in a CSS Grid layout), **Floating Panels** (detachable overlays for temporary inspections), and a **HUD Layer** (context-aware, auto-fading information overlay) to deliver a professional code visualization experience that maintains 60fps performance while providing VS Code-familiar power-user features. This architecture respects the primacy of the 3D canvas by using pointer-event fencing and transform-only animations, ensuring the graph visualization never reflows during UI operations.

```
┌─────────────────────────────────────────────┐
│                 APP CONTAINER                │
├─────────────────────────────────────────────┤
│  ┌─────────────────────────────────────┐    │
│  │      CANVAS LAYER (Three.js)        │    │
│  │         z-index: 0                  │    │
│  └─────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐    │
│  │    DOCKED LAYER (CSS Grid)          │    │
│  │  ┌────┬──────────────┬────┐        │    │
│  │  │Left│   Canvas      │Right│       │    │
│  │  ├────┴──────┬───────┴────┤        │    │
│  │  │   Bottom  │             │        │    │
│  │  └───────────┘   z-index: 10       │    │
│  └─────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐    │
│  │   FLOATING LAYER (Absolute)         │    │
│  │     [Panel] [Panel]  z-index: 100+  │    │
│  └─────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐    │
│  │      HUD LAYER (Fixed)              │    │
│  │  TL ─────────────── TR              │    │
│  │  │    [Center]      │  z-index:1000 │    │
│  │  BL ─────────────── BR              │    │
│  └─────────────────────────────────────┘    │
│  ┌─────────────────────────────────────┐    │
│  │   SNAP ZONES (Invisible guides)     │    │
│  └─────────────────────────────────────┘    │
└─────────────────────────────────────────────┘
```

## 2) Core Elements & Specs

### 2.1 **3D Canvas (Base Layer)**

**Role:** Primary interactive surface for graph visualization; maintains constant dimensions and never reflows during UI operations.

**Integration points:**

- Camera state changes → HUD density manager
- Selection events → Context card display
- Pointer events → Fenced from panel overlays

**API hooks (TypeScript signatures):**

```typescript
interface CanvasAPI {
  onCameraChange(callback: (state: CameraState) => void): () => void;
  onNodeSelect(callback: (node: GraphNode | null) => void): () => void;
  onNodeHover(callback: (node: GraphNode | null, pos: Vector2) => void): () => void;
  getViewport(): { width: number; height: number; dpr: number };
  projectToScreen(worldPos: Vector3): Vector2 | null;
}
```

### 2.2 **Docked Panels (Panel Layer / Grid)**

**DOM/CSS layout:** `og-docked-layer` uses CSS Grid with named areas (left/right/bottom), splitter divs for resizing.

**Splitter behavior:**

- Pointer capture on drag start
- Min/max constraints (200px min, 50% max)
- Smooth resize via `grid-template-columns` updates
- Debounced persistence to localStorage

**Panel chrome:**

- Header: title, collapse button (chevron), detach button (window icon), close (×)
- Keyboard navigation: F6 cycles panels, Escape closes active
- ARIA roles: `complementary` for sidebars, `region` for bottom

**Acceptance criteria:**

- ✓ Splitters resize smoothly without canvas reflow
- ✓ Panel sizes persist across sessions
- ✓ Collapse animation uses CSS transitions (300ms ease)
- ✓ Tab navigation works within panels
- ✓ Screen reader announces panel names

### 2.3 **Floating Panels (Overlay Layer)**

**Implementation:** Absolute-positioned `<og-float-panel>` custom elements, managed z-stack, drag via transform3d.

**Snap zones:**

- 9 zones: 8 edges (N/S/E/W/NE/NW/SE/SW) + center
- Magnetic threshold: 40px
- Visual preview: semi-transparent overlay
- Hysteresis: 10px deadzone to prevent jitter

**Persistence:** Position (x,y), size (w,h), z-index, visibility saved per layout preset.

**Tear-out:** Optional Tauri window spawn for dual-monitor workflows.

**Acceptance criteria:**

- ✓ Drag doesn't affect canvas framerate
- ✓ Snap zones show visual feedback on hover
- ✓ Z-order updates on click (bring to front)
- ✓ Escape key closes topmost panel
- ✓ Positions restore correctly after reload

### 2.4 **HUD (Heads-Up Display)**

**Elements:** Context cards (node details), breadcrumbs (navigation path), minimap (overview), metrics (FPS/nodes).

**Zoom-aware density:**

- Far (zoom 0-0.3): Show only minimap
- Medium (zoom 0.3-0.7): Add breadcrumbs, node counts
- Close (zoom 0.7-1.5): Show all elements
- Detail (zoom 1.5+): Full context cards

**Auto-fade:** 3s timeout for static elements, immediate fade on camera move.

**Occlusion:** 20px margin from panel edges, center-avoidance during rotation.

**Acceptance criteria:**

- ✓ HUD opacity responds to zoom level
- ✓ Context cards track node position smoothly
- ✓ Auto-fade timer resets on interaction
- ✓ HUD never overlaps interactive controls
- ✓ Metrics update at 10Hz max

### 2.5 **Snap Zones & Docking Heuristics**

**Zone definitions:**

- Edge zones: 100px wide bands along viewport edges
- Corner zones: 150×150px squares at corners
- Center zone: 200×200px center area

**Thresholds:**

- Enter threshold: 40px (start showing preview)
- Commit threshold: 20px (snap on release)
- Hysteresis band: 10px (prevent flutter)

**Visual feedback:**

- Preview: 50% opacity overlay of final position
- Active zone: 2px solid accent border
- Cursor: Changes to `move` during drag

**Conflict resolution:** Nearest zone wins; tie-break by priority (edges > corners > center).

**Acceptance criteria:**

- ✓ Snap preview appears at 40px distance
- ✓ Smooth animation to snap position (200ms ease-out)
- ✓ Multi-panel snapping doesn't overlap
- ✓ Keyboard users can snap via arrow keys
- ✓ Touch/pen input works identically

### 2.6 **Panel State & Layouts**

**State model (nanostores):**

```typescript
import { map, atom } from 'nanostores';

export const panelsState = map<Record<string, PanelState>>();
export const activeLayout = atom<LayoutPreset>();
export const hudElements = map<Record<string, HUDElement>>();
```

**Named presets:**

- **Explore:** Focus on navigation (file tree left, search floating)
- **Inspect:** Deep dive (file tree + properties docked)
- **Debug:** Full context (all panels + logs bottom)

**Save/restore:** JSON schema v1, auto-migrate on version change.

**Acceptance criteria:**

- ✓ Layout switch instant (<100ms)
- ✓ Custom layouts saveable with name
- ✓ Export produces valid JSON
- ✓ Import validates schema before apply
- ✓ Forward compatibility via version field

## 3) Technical Design

### 3.1 **DOM & Layers**

```html
<div id="app" class="og-app">
  <!-- Canvas Layer -->
  <div id="canvas-layer" class="og-canvas-layer">
    <div id="graph-container"></div>
  </div>
  
  <!-- Docked Layer -->
  <div id="docked-layer" class="og-docked-layer">
    <div class="og-panel-left" data-panel="file-tree">
      <og-panel title="Explorer"></og-panel>
    </div>
    <div class="og-splitter og-splitter-v-left"></div>
    
    <div class="og-panel-right" data-panel="properties">
      <og-panel title="Properties"></og-panel>
    </div>
    <div class="og-splitter og-splitter-v-right"></div>
    
    <div class="og-panel-bottom" data-panel="terminal">
      <og-panel title="Output"></og-panel>
    </div>
    <div class="og-splitter og-splitter-h-bottom"></div>
  </div>
  
  <!-- Floating Layer -->
  <div id="floating-layer" class="og-floating-layer"></div>
  
  <!-- HUD Layer -->
  <div id="hud-layer" class="og-hud-layer">
    <div class="og-hud-tl"></div>
    <div class="og-hud-tr"></div>
    <div class="og-hud-bl"></div>
    <div class="og-hud-br"></div>
    <div class="og-hud-center"></div>
  </div>
  
  <!-- Snap Zones -->
  <div id="snap-zones" class="og-snap-zones"></div>
</div>
```

### 3.2 **TypeScript Interfaces**

```typescript
export type PanelMode = 'docked' | 'floating' | 'window' | 'minimized';
export type DockArea = 'left' | 'right' | 'bottom';

export interface PanelState {
  id: string;
  mode: PanelMode;
  area?: DockArea;
  pos?: { x: number; y: number };
  size?: { w: number; h: number };
  z?: number;
  visible: boolean;
  opacity?: number;
  canFloat?: boolean;
  canTearOut?: boolean;
  min?: { w: number; h: number };
  max?: { w?: number; h?: number };
}

export interface HUDElement {
  id: string;
  kind: 'metric' | 'breadcrumb' | 'tooltip' | 'minimap' | 'context';
  quadrant: 'tl' | 'tr' | 'bl' | 'br' | 'center';
  priority: number;
  fadeMs: number;
  zoom: { min: number; max: number };
  autoHide: boolean;
}

export interface LayoutPreset {
  name: 'explore' | 'inspect' | 'debug' | 'custom';
  panels: Record<string, PanelState>;
  hud: HUDElement[];
  canvas?: { fov?: number; controls?: 'trackball' | 'orbit' };
  version: 1;
}
```

### 3.3 **Component Contracts (Lit)**

```typescript
// og-panel.ts
import { LitElement, html, css } from 'lit';
import { customElement, property } from 'lit/decorators.js';

@customElement('og-panel')
export class OGPanel extends LitElement {
  @property() title = '';
  @property({ type: Boolean }) collapsed = false;
  
  render() {
    return html`
      <div class="panel-header">
        <span class="panel-title">${this.title}</span>
        <div class="panel-controls">
          <button @click=${this.#handleCollapse} aria-label="Collapse">
            <svg><!-- chevron icon --></svg>
          </button>
          <button @click=${this.#handleDetach} aria-label="Detach">
            <svg><!-- window icon --></svg>
          </button>
          <button @click=${this.#handleClose} aria-label="Close">
            <svg><!-- × icon --></svg>
          </button>
        </div>
      </div>
      <div class="panel-content" ?hidden=${this.collapsed}>
        <slot></slot>
      </div>
    `;
  }
  
  #handleDetach() {
    this.dispatchEvent(new CustomEvent('og:detach', { 
      detail: { panelId: this.id },
      bubbles: true 
    }));
  }
}

// og-float-panel.ts
@customElement('og-float-panel')
export class OGFloatPanel extends LitElement {
  @property() panelId = '';
  @property({ type: Object }) position = { x: 100, y: 100 };
  
  connectedCallback() {
    super.connectedCallback();
    this.#setupDrag();
  }
  
  #setupDrag() {
    let dragState: { startX: number; startY: number } | null = null;
    
    this.addEventListener('pointerdown', (e) => {
      if ((e.target as Element).closest('.panel-header')) {
        dragState = { startX: e.clientX - this.position.x, startY: e.clientY - this.position.y };
        this.setPointerCapture(e.pointerId);
      }
    });
    
    this.addEventListener('pointermove', (e) => {
      if (dragState) {
        const newPos = { 
          x: e.clientX - dragState.startX, 
          y: e.clientY - dragState.startY 
        };
        this.position = newPos;
        this.style.transform = `translate3d(${newPos.x}px, ${newPos.y}px, 0)`;
        this.dispatchEvent(new CustomEvent('og:drag', { detail: newPos }));
      }
    });
  }
}

// og-hud-card.ts
@customElement('og-hud-card')
export class OGHudCard extends LitElement {
  @property() target: any = null;
  @property() variant: 'compact' | 'detailed' = 'compact';
  private fadeTimer?: number;
  
  updated() {
    this.#resetFade();
  }
  
  #resetFade() {
    clearTimeout(this.fadeTimer);
    this.style.opacity = '1';
    this.fadeTimer = window.setTimeout(() => {
      this.style.opacity = '0';
    }, 3000);
  }
}
```

### 3.4 **Managers & Services**

```typescript
// panel-manager.ts
export class PanelManager {
  private registry = new Map<string, PanelConfig>();
  private mounted = new Map<string, PanelInstance>();
  
  register(id: string, config: PanelConfig) {
    this.registry.set(id, config);
  }
  
  mount(panelId: string, container: HTMLElement) {
    const config = this.registry.get(panelId);
    if (!config) throw new Error(`Panel ${panelId} not registered`);
    
    const instance = config.factory();
    container.appendChild(instance);
    this.mounted.set(panelId, instance);
    
    eventBus.emit('panel:mounted', { panelId });
  }
  
  dock(panelId: string, area: DockArea) {
    const state = panelsState.get()[panelId];
    panelsState.setKey(panelId, { ...state, mode: 'docked', area });
    eventBus.emit('panel:docked', { panelId, area });
  }
  
  float(panelId: string, position?: { x: number; y: number }) {
    const state = panelsState.get()[panelId];
    panelsState.setKey(panelId, { 
      ...state, 
      mode: 'floating', 
      pos: position || { x: 100, y: 100 },
      z: this.getNextZIndex()
    });
    eventBus.emit('panel:floated', { panelId });
  }
  
  private getNextZIndex(): number {
    const panels = Object.values(panelsState.get());
    const maxZ = Math.max(...panels.map(p => p.z || 100));
    return maxZ + 1;
  }
}

// floating-panels.ts
export class FloatingPanelEngine {
  private dragState: Map<string, DragInfo> = new Map();
  private snapZones: SnapZone[] = [];
  private rafId?: number;
  
  startDrag(panelId: string, startPos: Vector2) {
    this.dragState.set(panelId, {
      startPos,
      currentPos: startPos,
      snapping: null
    });
    this.showSnapZones();
    this.startRAF();
  }
  
  updateDrag(panelId: string, pos: Vector2) {
    const state = this.dragState.get(panelId);
    if (!state) return;
    
    state.currentPos = pos;
    const snapTarget = this.detectSnap(pos);
    
    if (snapTarget !== state.snapping) {
      state.snapping = snapTarget;
      this.updateSnapPreview(snapTarget);
    }
  }
  
  private detectSnap(pos: Vector2): SnapZone | null {
    for (const zone of this.snapZones) {
      const dist = zone.getDistance(pos);
      if (dist < 40) return zone;
    }
    return null;
  }
  
  private startRAF() {
    const update = () => {
      for (const [panelId, state] of this.dragState) {
        const panel = document.getElementById(panelId);
        if (panel) {
          panel.style.transform = `translate3d(${state.currentPos.x}px, ${state.currentPos.y}px, 0)`;
        }
      }
      this.rafId = requestAnimationFrame(update);
    };
    update();
  }
}

// hud-manager.ts  
export class HUDManager {
  private elements = new Map<string, HUDElement>();
  private zoomLevel = 1;
  private fadeTimers = new Map<string, number>();
  
  updateDensity(zoom: number) {
    this.zoomLevel = zoom;
    
    for (const [id, element] of this.elements) {
      const visible = zoom >= element.zoom.min && zoom <= element.zoom.max;
      const container = this.getQuadrant(element.quadrant);
      const dom = container.querySelector(`[data-hud-id="${id}"]`);
      
      if (dom) {
        dom.style.display = visible ? 'block' : 'none';
        if (visible && element.autoHide) {
          this.scheduleFade(id, element.fadeMs);
        }
      }
    }
  }
  
  showContextCard(node: GraphNode, screenPos: Vector2) {
    const card = document.createElement('og-hud-card');
    card.target = node;
    card.variant = this.zoomLevel > 1.5 ? 'detailed' : 'compact';
    
    // Occlusion check
    const safePos = this.findSafePosition(screenPos);
    card.style.transform = `translate3d(${safePos.x}px, ${safePos.y}px, 0)`;
    
    this.getQuadrant('center').appendChild(card);
  }
  
  private findSafePosition(pos: Vector2): Vector2 {
    const panels = document.querySelectorAll('.og-panel');
    const margin = 20;
    
    for (const panel of panels) {
      const rect = panel.getBoundingClientRect();
      if (pos.x > rect.left - margin && pos.x < rect.right + margin &&
          pos.y > rect.top - margin && pos.y < rect.bottom + margin) {
        // Adjust position to avoid panel
        if (pos.x < rect.left + rect.width / 2) {
          pos.x = rect.left - margin;
        } else {
          pos.x = rect.right + margin;
        }
      }
    }
    return pos;
  }
}
```

**Event bus topics:**

```typescript
// Using mitt
import mitt from 'mitt';

type Events = {
  'panel:mounted': { panelId: string };
  'panel:docked': { panelId: string; area: DockArea };
  'panel:floated': { panelId: string };
  'panel:closed': { panelId: string };
  'hud:show': { element: HUDElement };
  'hud:hide': { elementId: string };
  'layout:switch': { preset: string };
  'layout:save': { name: string };
};

export const eventBus = mitt<Events>();
```

### 3.5 **Persistence**

```typescript
// persistence.ts
import { panelsState, activeLayout } from '../state/panels';

const STORAGE_KEY = 'og:layout:v1';
const DEBOUNCE_MS = 500;

let saveTimer: number;

panelsState.subscribe((panels) => {
  clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    const layout: LayoutPreset = {
      name: 'custom',
      panels,
      hud: Array.from(hudElements.get().values()),
      version: 1
    };
    
    if (window.__TAURI__) {
      // Save to Tauri app data dir
      invoke('save_layout', { layout });
    } else {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(layout));
    }
  }, DEBOUNCE_MS);
});

export async function loadLayout(): Promise<void> {
  let data: string | null = null;
  
  if (window.__TAURI__) {
    data = await invoke('load_layout');
  } else {
    data = localStorage.getItem(STORAGE_KEY);
  }
  
  if (data) {
    const layout = JSON.parse(data) as LayoutPreset;
    if (layout.version === 1) {
      panelsState.set(layout.panels);
      activeLayout.set(layout);
    } else {
      migrateLayout(layout);
    }
  }
}

export function exportLayout(): string {
  return JSON.stringify(activeLayout.get(), null, 2);
}

export function importLayout(json: string): void {
  const layout = JSON.parse(json) as LayoutPreset;
  validateLayout(layout);
  panelsState.set(layout.panels);
  activeLayout.set(layout);
}
```

### 3.6 **Performance Budget**

```typescript
// performance-monitor.ts
class PerformanceMonitor {
  private frameMs: number[] = [];
  private qualityMode: 'high' | 'medium' | 'low' = 'high';
  
  measureFrame(ms: number) {
    this.frameMs.push(ms);
    if (this.frameMs.length > 60) this.frameMs.shift();
    
    const avgMs = this.frameMs.reduce((a, b) => a + b) / this.frameMs.length;
    
    if (avgMs > 18 && this.qualityMode === 'high') {
      this.downgrade();
    } else if (avgMs < 14 && this.qualityMode !== 'high') {
      this.upgrade();
    }
  }
  
  private downgrade() {
    this.qualityMode = this.qualityMode === 'high' ? 'medium' : 'low';
    document.documentElement.dataset.quality = this.qualityMode;
    
    // CSS handles the visual changes
    // [data-quality="low"] .og-float-panel { backdrop-filter: none; }
  }
}
```

**Pointer event fencing:**

```typescript
// All panels stop propagation
document.querySelectorAll('.og-panel, .og-float-panel').forEach(panel => {
  panel.addEventListener('pointerdown', e => e.stopPropagation());
  panel.addEventListener('wheel', e => e.stopPropagation(), { passive: false });
});

// Canvas only gets events in clear areas
canvasLayer.addEventListener('pointerdown', (e) => {
  const target = document.elementFromPoint(e.clientX, e.clientY);
  if (target === canvasLayer || target?.closest('#graph-container')) {
    // Handle 3D interaction
  }
});
```

### 3.7 **Accessibility**

```typescript
// a11y-manager.ts
export class A11yManager {
  setupPanel(panel: HTMLElement) {
    panel.setAttribute('role', 'complementary');
    panel.setAttribute('aria-label', panel.dataset.panelTitle || 'Panel');
    
    // Focus trap
    const focusableElements = panel.querySelectorAll(
      'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );
    
    panel.addEventListener('keydown', (e) => {
      if (e.key === 'Tab') {
        const first = focusableElements[0] as HTMLElement;
        const last = focusableElements[focusableElements.length - 1] as HTMLElement;
        
        if (e.shiftKey && document.activeElement === first) {
          e.preventDefault();
          last.focus();
        } else if (!e.shiftKey && document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    });
  }
  
  announceHUD(message: string) {
    const liveRegion = document.getElementById('og-live-region');
    if (liveRegion) {
      liveRegion.textContent = message;
      setTimeout(() => liveRegion.textContent = '', 100);
    }
  }
}
```

## 4) UX Guardrails

**When to use each layer:**

| Content Type           | HUD | Floating Panel | Docked Panel |
| ---------------------- | --- | -------------- | ------------ |
| Node details on hover  | ✓   |                |              |
| File tree              |     |                | ✓            |
| Search results         |     | ✓              |              |
| Properties editor      |     |                | ✓            |
| Metrics/FPS            | ✓   |                |              |
| Terminal/logs          |     |                | ✓            |
| Temporary comparisons  |     | ✓              |              |
| Navigation breadcrumbs | ✓   |                |              |

**Occlusion rules:**

- HUD maintains 20px margin from all panel edges
- During camera rotation, HUD fades to 30% opacity
- Auto-minimize panels when coverage >60% of canvas
- Context cards reposition to avoid center during interaction

**Keyboard map:**

```
Ctrl+B          Toggle left panel
Ctrl+J          Toggle bottom panel  
Ctrl+Shift+E    Focus file tree
Ctrl+Shift+F    Open floating search
Ctrl+`          Toggle terminal
Ctrl+\          Toggle right panel
F6              Cycle panel focus
F11             Fullscreen canvas
Escape          Close active floating panel
Ctrl+1/2/3      Switch layout (Explore/Inspect/Debug)
Ctrl+Shift+P    Command palette
Alt+Z           Toggle HUD
```

## 5) Default Layout Presets

**Explore:**

- Left: File tree (300px, collapsible)
- Right: Hidden
- Bottom: Hidden  
- Floating: Search (top center, 600×400px)
- HUD: Minimap (BR), breadcrumbs (TL), metrics (TR)

**Inspect:**

- Left: File tree (250px)
- Right: Properties (350px)
- Bottom: Hidden
- Floating: None
- HUD: Context cards on hover, metrics (TR)

**Debug:**

- Left: File tree (200px)
- Right: Properties (300px)
- Bottom: Logs (200px height)
- Floating: Watch expressions (TR), breakpoints (TL)
- HUD: Full metrics, call stack breadcrumbs, minimap

## 6) File Map

```
src/
├─ state/
│  ├─ panels.ts           # nanostores for panels/layouts
│  └─ hud.ts              # HUD element state
├─ ui/
│  ├─ components/
│  │  ├─ og-panel.ts      # Docked panel component
│  │  ├─ og-float-panel.ts # Floating panel component
│  │  └─ og-hud-card.ts   # HUD card component
│  ├─ panel-system/
│  │  ├─ panel-manager.ts # Panel orchestration
│  │  ├─ docked-panels.ts # Grid layout manager
│  │  ├─ floating-panels.ts # Drag & drop engine
│  │  └─ snap-zones.ts    # Magnetic snap system
│  ├─ hud/
│  │  ├─ hud-manager.ts   # HUD orchestration
│  │  ├─ minimap.ts       # Graph overview
│  │  └─ breadcrumbs.ts   # Navigation path
│  ├─ layouts/
│  │  ├─ layout-manager.ts # Preset switching
│  │  ├─ explore.json     # Explore preset
│  │  ├─ inspect.json     # Inspect preset
│  │  └─ debug.json       # Debug preset
│  └─ styles/
│     ├─ panels.css       # Panel styling
│     ├─ hud.css          # HUD styles
│     └─ animations.css   # Transitions
├─ visualization/
│  └─ graph3d.ts          # Camera hooks, selection
├─ utils/
│  ├─ persistence.ts      # Save/load layouts
│  └─ geometry.ts         # Collision, projection
├─ main.ts                # Initialize UI system
└─ index.html             # DOM structure
```

## 7) Implementation Plan (Phases & DoD)

**Week 1: Docked grid + splitters + persistence**

- DoD: Resize/collapse works; sizes restored; a11y basics pass; no canvas reflow

**Week 2: Floating panels + snap zones + z-stack**

- DoD: Drag/resize/snap smooth; positions persist; keyboard close; 60fps maintained

**Week 3: HUD primitives (metrics, breadcrumbs, context)**

- DoD: Zoom-aware density works; auto-fade reliable; pointer fencing effective

**Week 4: Layout presets + export/import + a11y polish**

- DoD: Preset switch <100ms; JSON round-trip valid; screen reader navigation works

**Week 5: Perf tuning + quality throttle + minimap**

- DoD: Stable 60fps with 50k nodes; blur fallback engages at 18ms frame time

## 8) Risks & Mitigations

- **Snap heuristics feel twitchy** → Add 10px hysteresis band + smooth preview
- **GPU overdraw from effects** → Quality modes + auto-throttle by frame time
- **State drift (multi-window)** → Single source of truth in nanostores + IPC sync
- **A11y regressions** → Playwright + Axe automated testing in CI

## 9) Acceptance Checklist

- ✓ Canvas never resizes during panel/HUD operations
- ✓ All panel actions have keyboard equivalents and ARIA roles
- ✓ Layouts restore exactly; export/import deterministic
- ✓ HUD never obscures central aim during camera motion
- ✓ Sustained 60fps with 50k nodes/100k edges on mid-tier GPU
- ✓ Panel drag/resize uses transform only (no reflow)
- ✓ Memory stable over 1hr session (no leaks)
- ✓ Touch/pen input works identically to mouse
- ✓ Multi-monitor aware (panels stay in viewport)
- ✓ Theme follows system dark/light preference

## 10) Appendix

**CSS naming conventions:**

- Prefix: `og-*` (omnigraph)
- Modifiers: BEM-style (`og-panel--collapsed`)
- States: data attributes (`[data-docked="left"]`)

**Event bus topics:**

```
panel:mount      { panelId, container }
panel:unmount    { panelId }
panel:dock       { panelId, area }
panel:float      { panelId, position }
panel:resize     { panelId, size }
panel:close      { panelId }
hud:show         { element, quadrant }
hud:hide         { elementId }
hud:fade         { elementId, duration }
layout:switch    { preset }
layout:save      { name, data }
layout:load      { name }
```

**Example saved layout JSON:**

```json
{
  "name": "custom",
  "version": 1,
  "panels": {
    "file-tree": {
      "id": "file-tree",
      "mode": "docked",
      "area": "left",
      "size": { "w": 280, "h": null },
      "visible": true,
      "opacity": 1
    },
    "properties": {
      "id": "properties",
      "mode": "floating",
      "pos": { "x": 820, "y": 100 },
      "size": { "w": 400, "h": 600 },
      "z": 101,
      "visible": true
    }
  },
  "hud": [
    {
      "id": "minimap",
      "kind": "minimap",
      "quadrant": "br",
      "priority": 1,
      "fadeMs": 0,
      "zoom": { "min": 0, "max": 2 },
      "autoHide": false
    }
  ],
  "canvas": {
    "fov": 75,
    "controls": "orbit"
  }
}
```
