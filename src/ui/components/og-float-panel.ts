import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { eventBus } from '../../state/events';

@customElement('og-float-panel')
export class OGFloatPanel extends LitElement {
  @property({ attribute: 'panel-id' }) panelId = '';
  @property({ type: Object }) position = { x: 100, y: 100 };
  @property({ type: Object }) size = { w: 400, h: 300 };
  @property() title = '';
  @property({ type: Number }) zIndex = 100;
  
  @state() private dragState: { startX: number; startY: number; startPosX: number; startPosY: number } | null = null;
  @state() private resizeState: { startX: number; startY: number; startW: number; startH: number; edge: string } | null = null;

  static styles = css`
    :host {
      position: absolute;
      display: flex;
      flex-direction: column;
      background: var(--panel-bg, rgba(30, 30, 40, 0.98));
      backdrop-filter: blur(15px);
      border: 1px solid var(--panel-border, rgba(255, 255, 255, 0.15));
      border-radius: 8px;
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
      overflow: hidden;
      min-width: 200px;
      min-height: 150px;
    }

    .float-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 10px 12px;
      background: var(--panel-header-bg, rgba(40, 40, 50, 0.9));
      border-bottom: 1px solid var(--panel-border, rgba(255, 255, 255, 0.1));
      cursor: move;
      user-select: none;
    }

    .float-title {
      font-size: 14px;
      font-weight: 500;
      color: var(--panel-title-color, #fff);
      flex: 1;
    }

    .float-controls {
      display: flex;
      gap: 4px;
    }

    .float-btn {
      width: 24px;
      height: 24px;
      padding: 0;
      background: transparent;
      border: none;
      color: var(--panel-control-color, rgba(255, 255, 255, 0.6));
      cursor: pointer;
      border-radius: 4px;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: all 0.2s;
    }

    .float-btn:hover {
      background: var(--panel-control-hover-bg, rgba(255, 255, 255, 0.1));
      color: var(--panel-control-hover-color, #fff);
    }

    .float-content {
      flex: 1;
      overflow: auto;
      padding: 12px;
    }

    /* Resize handles */
    .resize-handle {
      position: absolute;
      background: transparent;
    }

    .resize-n {
      top: 0;
      left: 10px;
      right: 10px;
      height: 5px;
      cursor: ns-resize;
    }

    .resize-s {
      bottom: 0;
      left: 10px;
      right: 10px;
      height: 5px;
      cursor: ns-resize;
    }

    .resize-e {
      top: 10px;
      bottom: 10px;
      right: 0;
      width: 5px;
      cursor: ew-resize;
    }

    .resize-w {
      top: 10px;
      bottom: 10px;
      left: 0;
      width: 5px;
      cursor: ew-resize;
    }

    .resize-ne {
      top: 0;
      right: 0;
      width: 10px;
      height: 10px;
      cursor: nesw-resize;
    }

    .resize-nw {
      top: 0;
      left: 0;
      width: 10px;
      height: 10px;
      cursor: nwse-resize;
    }

    .resize-se {
      bottom: 0;
      right: 0;
      width: 10px;
      height: 10px;
      cursor: nwse-resize;
    }

    .resize-sw {
      bottom: 0;
      left: 0;
      width: 10px;
      height: 10px;
      cursor: nesw-resize;
    }

    svg {
      width: 16px;
      height: 16px;
      fill: currentColor;
    }
  `;

  render() {
    return html`
      <div class="float-header" @pointerdown=${this.handleDragStart}>
        <span class="float-title">${this.title}</span>
        <div class="float-controls">
          <button @click=${this.handleDock} aria-label="Dock" class="float-btn">
            <svg viewBox="0 0 24 24">
              <path d="M5 16L3 5l5 5-3 6zm14 0l2-11-5 5 3 6z"/>
            </svg>
          </button>
          <button @click=${this.handleClose} aria-label="Close" class="float-btn">
            <svg viewBox="0 0 24 24">
              <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
            </svg>
          </button>
        </div>
      </div>
      <div class="float-content">
        <slot></slot>
      </div>
      <!-- Resize handles -->
      <div class="resize-handle resize-n" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 'n')}></div>
      <div class="resize-handle resize-s" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 's')}></div>
      <div class="resize-handle resize-e" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 'e')}></div>
      <div class="resize-handle resize-w" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 'w')}></div>
      <div class="resize-handle resize-ne" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 'ne')}></div>
      <div class="resize-handle resize-nw" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 'nw')}></div>
      <div class="resize-handle resize-se" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 'se')}></div>
      <div class="resize-handle resize-sw" @pointerdown=${(e: PointerEvent) => this.handleResizeStart(e, 'sw')}></div>
    `;
  }

  connectedCallback() {
    super.connectedCallback();
    this.updatePosition();
    this.updateSize();
    this.style.zIndex = String(this.zIndex);
    
    // Event listeners
    this.addEventListener('pointermove', this.handlePointerMove);
    this.addEventListener('pointerup', this.handlePointerUp);
    this.addEventListener('pointercancel', this.handlePointerUp);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    this.removeEventListener('pointermove', this.handlePointerMove);
    this.removeEventListener('pointerup', this.handlePointerUp);
    this.removeEventListener('pointercancel', this.handlePointerUp);
  }

  private handleDragStart(e: PointerEvent) {
    if ((e.target as HTMLElement).closest('.float-controls')) {
      return; // Don't drag if clicking controls
    }
    
    e.preventDefault();
    this.setPointerCapture(e.pointerId);
    
    this.dragState = {
      startX: e.clientX,
      startY: e.clientY,
      startPosX: this.position.x,
      startPosY: this.position.y
    };
    
    // Bring to front
    eventBus.emit('panel:floated', { panelId: this.panelId });
  }

  private handleResizeStart(e: PointerEvent, edge: string) {
    e.preventDefault();
    e.stopPropagation();
    this.setPointerCapture(e.pointerId);
    
    this.resizeState = {
      startX: e.clientX,
      startY: e.clientY,
      startW: this.size.w,
      startH: this.size.h,
      edge
    };
  }

  private handlePointerMove = (e: PointerEvent) => {
    if (this.dragState) {
      const deltaX = e.clientX - this.dragState.startX;
      const deltaY = e.clientY - this.dragState.startY;
      
      this.position = {
        x: this.dragState.startPosX + deltaX,
        y: this.dragState.startPosY + deltaY
      };
      
      this.updatePosition();
      eventBus.emit('panel:drag', { panelId: this.panelId, pos: this.position });
    } else if (this.resizeState) {
      const deltaX = e.clientX - this.resizeState.startX;
      const deltaY = e.clientY - this.resizeState.startY;
      const edge = this.resizeState.edge;
      
      let newW = this.resizeState.startW;
      let newH = this.resizeState.startH;
      let newX = this.position.x;
      let newY = this.position.y;
      
      if (edge.includes('e')) newW += deltaX;
      if (edge.includes('w')) {
        newW -= deltaX;
        newX += deltaX;
      }
      if (edge.includes('s')) newH += deltaY;
      if (edge.includes('n')) {
        newH -= deltaY;
        newY += deltaY;
      }
      
      // Apply minimum size constraints
      newW = Math.max(200, newW);
      newH = Math.max(150, newH);
      
      this.size = { w: newW, h: newH };
      this.position = { x: newX, y: newY };
      
      this.updateSize();
      this.updatePosition();
      
      eventBus.emit('panel:resized', { panelId: this.panelId, size: this.size });
    }
  };

  private handlePointerUp = (e: PointerEvent) => {
    if (this.dragState || this.resizeState) {
      this.releasePointerCapture(e.pointerId);
      this.dragState = null;
      this.resizeState = null;
    }
  };

  private updatePosition() {
    this.style.transform = `translate3d(${this.position.x}px, ${this.position.y}px, 0)`;
  }

  private updateSize() {
    this.style.width = `${this.size.w}px`;
    this.style.height = `${this.size.h}px`;
  }

  private handleDock() {
    eventBus.emit('panel:docked', { panelId: this.panelId, area: 'left' });
  }

  private handleClose() {
    eventBus.emit('panel:closed', { panelId: this.panelId });
  }
}