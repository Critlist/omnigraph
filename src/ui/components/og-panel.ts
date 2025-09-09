import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { eventBus } from '../../state/events';

@customElement('og-panel')
export class OGPanel extends LitElement {
  @property() title = '';
  @property({ attribute: 'panel-id' }) panelId = '';
  @property({ type: Boolean }) collapsed = false;
  @property({ type: Boolean }) canFloat = true;
  @property({ type: Boolean }) canClose = true;
  @property({ type: Boolean }) canCollapse = true;
  
  @state() private isResizing = false;

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      background: var(--panel-bg, rgba(30, 30, 40, 0.95));
      backdrop-filter: blur(10px);
      border: 1px solid var(--panel-border, rgba(255, 255, 255, 0.1));
      border-radius: 8px;
      overflow: hidden;
    }

    .panel-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 12px;
      background: var(--panel-header-bg, rgba(40, 40, 50, 0.8));
      border-bottom: 1px solid var(--panel-border, rgba(255, 255, 255, 0.1));
      user-select: none;
      cursor: default;
    }

    .panel-title {
      font-size: 14px;
      font-weight: 500;
      color: var(--panel-title-color, #fff);
      flex: 1;
    }

    .panel-controls {
      display: flex;
      gap: 4px;
    }

    .panel-btn {
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

    .panel-btn:hover {
      background: var(--panel-control-hover-bg, rgba(255, 255, 255, 0.1));
      color: var(--panel-control-hover-color, #fff);
    }

    .panel-btn:focus {
      outline: 2px solid var(--focus-color, #3498db);
      outline-offset: -2px;
    }

    .panel-content {
      flex: 1;
      overflow: auto;
      padding: 12px;
    }

    .panel-content[hidden] {
      display: none;
    }

    .chevron {
      transition: transform 0.3s ease;
    }

    .chevron.collapsed {
      transform: rotate(-90deg);
    }

    svg {
      width: 16px;
      height: 16px;
      fill: currentColor;
    }
  `;

  render() {
    return html`
      <div class="panel-header">
        <span class="panel-title">${this.title}</span>
        <div class="panel-controls">
          ${this.canCollapse ? html`
            <button 
              @click=${this.handleCollapse} 
              aria-label="Collapse"
              aria-expanded=${!this.collapsed}
              class="panel-btn"
            >
              <svg class="chevron ${this.collapsed ? 'collapsed' : ''}" viewBox="0 0 24 24">
                <path d="M7 10l5 5 5-5z"/>
              </svg>
            </button>
          ` : ''}
          ${this.canFloat ? html`
            <button 
              @click=${this.handleDetach} 
              aria-label="Detach"
              class="panel-btn"
            >
              <svg viewBox="0 0 24 24">
                <path d="M19 13H13V19H11V13H5V11H11V5H13V11H19V13Z"/>
              </svg>
            </button>
          ` : ''}
          ${this.canClose ? html`
            <button 
              @click=${this.handleClose} 
              aria-label="Close"
              class="panel-btn"
            >
              <svg viewBox="0 0 24 24">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          ` : ''}
        </div>
      </div>
      <div class="panel-content" ?hidden=${this.collapsed}>
        <slot></slot>
      </div>
    `;
  }

  private handleCollapse() {
    this.collapsed = !this.collapsed;
    eventBus.emit('panel:collapsed', { 
      panelId: this.panelId,
      collapsed: this.collapsed
    });
  }

  private handleDetach() {
    eventBus.emit('panel:detached', { 
      panelId: this.panelId
    });
  }

  private handleClose() {
    eventBus.emit('panel:closed', { 
      panelId: this.panelId
    });
  }

  connectedCallback() {
    super.connectedCallback();
    this.setAttribute('role', 'complementary');
    this.setAttribute('aria-label', this.title || 'Panel');
  }
}