import { panelsState } from '../../state/panels';
import { eventBus } from '../../state/events';

export class SplitterManager {
  private splitters: Map<string, HTMLElement> = new Map();
  private dragState: {
    splitter: string;
    startX: number;
    startY: number;
    startSize: number;
    panelId: string;
  } | null = null;

  constructor() {
    this.initializeSplitters();
    this.setupEventListeners();
  }

  private initializeSplitters() {
    // Find all splitters in the DOM
    const leftSplitter = document.querySelector('.og-splitter-v-left') as HTMLElement;
    const rightSplitter = document.querySelector('.og-splitter-v-right') as HTMLElement;
    const bottomSplitter = document.querySelector('.og-splitter-h-bottom') as HTMLElement;

    if (leftSplitter) {
      this.splitters.set('left', leftSplitter);
      this.setupSplitter(leftSplitter, 'left', 'file-tree');
    }

    if (rightSplitter) {
      this.splitters.set('right', rightSplitter);
      this.setupSplitter(rightSplitter, 'right', 'properties');
    }

    if (bottomSplitter) {
      this.splitters.set('bottom', bottomSplitter);
      this.setupSplitter(bottomSplitter, 'bottom', 'terminal');
    }
  }

  private setupSplitter(splitter: HTMLElement, area: string, panelId: string) {
    splitter.addEventListener('pointerdown', (e) => {
      e.preventDefault();
      splitter.setPointerCapture(e.pointerId);
      splitter.classList.add('active');

      const state = panelsState.get()[panelId];
      if (!state || !state.size) return;

      this.dragState = {
        splitter: area,
        startX: e.clientX,
        startY: e.clientY,
        startSize: area === 'bottom' ? state.size.h : state.size.w,
        panelId
      };
    });

    splitter.addEventListener('pointermove', (e) => {
      if (!this.dragState || this.dragState.splitter !== area) return;

      const deltaX = e.clientX - this.dragState.startX;
      const deltaY = e.clientY - this.dragState.startY;
      
      let newSize: number;
      
      if (area === 'left') {
        newSize = this.dragState.startSize + deltaX;
      } else if (area === 'right') {
        newSize = this.dragState.startSize - deltaX;
      } else if (area === 'bottom') {
        newSize = this.dragState.startSize - deltaY;
      } else {
        return;
      }

      // Apply constraints
      const state = panelsState.get()[this.dragState.panelId];
      if (!state) return;

      const minSize = area === 'bottom' 
        ? (state.min?.h || 150) 
        : (state.min?.w || 200);
      
      const maxSize = area === 'bottom'
        ? (state.max?.h || 400)
        : (state.max?.w || 600);

      newSize = Math.max(minSize, Math.min(maxSize, newSize));

      // Update the panel size
      this.updatePanelSize(this.dragState.panelId, area, newSize);
    });

    splitter.addEventListener('pointerup', (e) => {
      if (this.dragState && this.dragState.splitter === area) {
        splitter.releasePointerCapture(e.pointerId);
        splitter.classList.remove('active');
        
        // Save the final size
        const state = panelsState.get()[this.dragState.panelId];
        if (state && state.size) {
          eventBus.emit('panel:resized', { 
            panelId: this.dragState.panelId, 
            size: state.size 
          });
        }
        
        this.dragState = null;
      }
    });

    // Handle pointer cancel (e.g., losing focus)
    splitter.addEventListener('pointercancel', (e) => {
      if (this.dragState && this.dragState.splitter === area) {
        splitter.releasePointerCapture(e.pointerId);
        splitter.classList.remove('active');
        this.dragState = null;
      }
    });
  }

  private updatePanelSize(panelId: string, area: string, size: number) {
    const state = panelsState.get()[panelId];
    if (!state || !state.size) return;

    // Update state
    const newSize = { ...state.size };
    if (area === 'bottom') {
      newSize.h = size;
    } else {
      newSize.w = size;
    }

    panelsState.setKey(panelId, { ...state, size: newSize });

    // Update CSS
    const panel = document.querySelector(`.og-panel-${area}`) as HTMLElement;
    if (panel) {
      if (area === 'bottom') {
        panel.style.height = `${size}px`;
        document.documentElement.style.setProperty(`--panel-${area}-height`, `${size}px`);
      } else {
        panel.style.width = `${size}px`;
        document.documentElement.style.setProperty(`--panel-${area}-width`, `${size}px`);
      }
    }
  }

  private setupEventListeners() {
    // Listen for panel visibility changes to show/hide splitters
    panelsState.subscribe((panels) => {
      Object.entries(panels).forEach(([id, state]) => {
        if (state.mode === 'docked' && state.area) {
          const splitter = this.splitters.get(state.area);
          if (splitter) {
            splitter.style.display = state.visible ? '' : 'none';
          }
        }
      });
    });

    // Add keyboard support for splitter movement
    document.addEventListener('keydown', (e) => {
      if (!this.dragState) return;

      const step = 10;
      let handled = false;

      switch (e.key) {
        case 'ArrowLeft':
          if (this.dragState.splitter === 'left' || this.dragState.splitter === 'right') {
            this.adjustSplitter(this.dragState.splitter, -step);
            handled = true;
          }
          break;
        case 'ArrowRight':
          if (this.dragState.splitter === 'left' || this.dragState.splitter === 'right') {
            this.adjustSplitter(this.dragState.splitter, step);
            handled = true;
          }
          break;
        case 'ArrowUp':
          if (this.dragState.splitter === 'bottom') {
            this.adjustSplitter(this.dragState.splitter, step);
            handled = true;
          }
          break;
        case 'ArrowDown':
          if (this.dragState.splitter === 'bottom') {
            this.adjustSplitter(this.dragState.splitter, -step);
            handled = true;
          }
          break;
        case 'Escape':
          this.cancelDrag();
          handled = true;
          break;
      }

      if (handled) {
        e.preventDefault();
      }
    });
  }

  private adjustSplitter(area: string, delta: number) {
    if (!this.dragState) return;

    const state = panelsState.get()[this.dragState.panelId];
    if (!state || !state.size) return;

    let newSize: number;
    if (area === 'bottom') {
      newSize = state.size.h + delta;
    } else {
      newSize = state.size.w + delta;
    }

    this.updatePanelSize(this.dragState.panelId, area, newSize);
  }

  private cancelDrag() {
    if (this.dragState) {
      const splitter = this.splitters.get(this.dragState.splitter);
      if (splitter) {
        splitter.classList.remove('active');
      }
      this.dragState = null;
    }
  }

  // Public methods
  showSplitter(area: string) {
    const splitter = this.splitters.get(area);
    if (splitter) {
      splitter.style.display = '';
    }
  }

  hideSplitter(area: string) {
    const splitter = this.splitters.get(area);
    if (splitter) {
      splitter.style.display = 'none';
    }
  }
}