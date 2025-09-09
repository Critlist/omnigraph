import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { eventBus } from '../../state/events';
import { panelsState } from '../../state/panels';

@customElement('og-menu-bar')
export class OGMenuBar extends LitElement {
  @state() private activeDropdown: string | null = null;
  @state() private panelStates = panelsState.get();

  static styles = css`
    :host {
      display: block;
      background: rgba(20, 20, 30, 0.95);
      backdrop-filter: blur(10px);
      border-bottom: 1px solid rgba(255, 255, 255, 0.1);
      user-select: none;
      z-index: 1100;
      position: relative;
    }

    .menu-container {
      display: flex;
      align-items: center;
      height: 36px;
      padding: 0 12px;
      gap: 8px;
    }

    .menu-item {
      position: relative;
    }

    .menu-button {
      padding: 6px 12px;
      background: transparent;
      border: none;
      color: rgba(255, 255, 255, 0.8);
      cursor: pointer;
      border-radius: 4px;
      font-size: 13px;
      transition: all 0.2s;
      display: flex;
      align-items: center;
      gap: 4px;
      height: 28px;
    }

    .menu-button:hover,
    .menu-button.active {
      background: rgba(255, 255, 255, 0.1);
      color: white;
    }

    .dropdown {
      position: absolute;
      top: 100%;
      left: 0;
      margin-top: 4px;
      background: rgba(30, 30, 40, 0.98);
      backdrop-filter: blur(20px);
      border: 1px solid rgba(255, 255, 255, 0.15);
      border-radius: 8px;
      box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
      min-width: 220px;
      opacity: 0;
      transform: translateY(-10px);
      pointer-events: none;
      transition: all 0.2s;
      z-index: 1200;
    }

    .dropdown.open {
      opacity: 1;
      transform: translateY(0);
      pointer-events: auto;
    }

    .dropdown-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 8px 12px;
      background: none;
      border: none;
      color: rgba(255, 255, 255, 0.9);
      cursor: pointer;
      transition: all 0.15s;
      width: 100%;
      text-align: left;
      font-size: 13px;
    }

    .dropdown-item:hover {
      background: rgba(255, 255, 255, 0.08);
      color: white;
    }

    .dropdown-item:first-child {
      border-radius: 8px 8px 0 0;
    }

    .dropdown-item:last-child {
      border-radius: 0 0 8px 8px;
    }

    .dropdown-divider {
      height: 1px;
      background: rgba(255, 255, 255, 0.1);
      margin: 4px 0;
    }

    .dropdown-label {
      flex: 1;
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .dropdown-shortcut {
      font-size: 11px;
      color: rgba(255, 255, 255, 0.5);
      font-family: monospace;
      padding: 2px 4px;
      background: rgba(255, 255, 255, 0.05);
      border-radius: 3px;
    }

    .checkbox {
      width: 16px;
      height: 16px;
      border: 1px solid rgba(255, 255, 255, 0.3);
      border-radius: 3px;
      margin-right: 8px;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: all 0.2s;
    }

    .checkbox.checked {
      background: var(--focus-color, #3498db);
      border-color: var(--focus-color, #3498db);
    }

    .checkbox.checked::after {
      content: '✓';
      color: white;
      font-size: 12px;
    }

    .separator {
      flex: 1;
    }

    .status-text {
      font-size: 12px;
      color: rgba(255, 255, 255, 0.5);
      padding: 0 8px;
    }

    .icon-button {
      width: 28px;
      height: 28px;
      padding: 0;
      background: transparent;
      border: none;
      color: rgba(255, 255, 255, 0.7);
      cursor: pointer;
      border-radius: 4px;
      display: flex;
      align-items: center;
      justify-content: center;
      transition: all 0.2s;
      font-size: 16px;
    }

    .icon-button:hover {
      background: rgba(255, 255, 255, 0.1);
      color: white;
    }

    .icon-button[title]::after {
      content: attr(title);
      position: absolute;
      bottom: -24px;
      left: 50%;
      transform: translateX(-50%);
      background: rgba(0, 0, 0, 0.8);
      color: white;
      padding: 4px 8px;
      border-radius: 4px;
      font-size: 11px;
      white-space: nowrap;
      opacity: 0;
      pointer-events: none;
      transition: opacity 0.2s;
    }

    .icon-button:hover::after {
      opacity: 1;
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    
    // Subscribe to panel state changes
    panelsState.subscribe((state) => {
      this.panelStates = state;
    });

    // Close dropdown on outside click
    document.addEventListener('click', this.handleOutsideClick);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    document.removeEventListener('click', this.handleOutsideClick);
  }

  private handleOutsideClick = (e: MouseEvent) => {
    if (!this.contains(e.target as Node)) {
      this.activeDropdown = null;
    }
  };

  private toggleDropdown(name: string) {
    this.activeDropdown = this.activeDropdown === name ? null : name;
  }

  private togglePanel(panelId: string) {
    eventBus.emit('panel:toggle', { panelId });
  }

  private switchLayout(preset: string) {
    eventBus.emit('layout:switch', { preset });
    this.activeDropdown = null;
  }

  private openCommandPalette() {
    const palette = document.querySelector('og-command-palette') as any;
    if (palette) {
      palette.open = true;
    }
  }

  render() {
    return html`
      <div class="menu-container">
        <!-- View Menu -->
        <div class="menu-item">
          <button 
            class="menu-button ${this.activeDropdown === 'view' ? 'active' : ''}"
            @click=${() => this.toggleDropdown('view')}
          >
            View
          </button>
          <div class="dropdown ${this.activeDropdown === 'view' ? 'open' : ''}">
            <button class="dropdown-item" @click=${() => this.togglePanel('file-tree')}>
              <span class="dropdown-label">
                <span class="checkbox ${this.panelStates['file-tree']?.visible ? 'checked' : ''}"></span>
                Explorer
              </span>
              <span class="dropdown-shortcut">Ctrl+B</span>
            </button>
            <button class="dropdown-item" @click=${() => this.togglePanel('properties')}>
              <span class="dropdown-label">
                <span class="checkbox ${this.panelStates['properties']?.visible ? 'checked' : ''}"></span>
                Properties
              </span>
              <span class="dropdown-shortcut">Ctrl+\\</span>
            </button>
            <button class="dropdown-item" @click=${() => this.togglePanel('terminal')}>
              <span class="dropdown-label">
                <span class="checkbox ${this.panelStates['terminal']?.visible ? 'checked' : ''}"></span>
                Terminal
              </span>
              <span class="dropdown-shortcut">Ctrl+J</span>
            </button>
            <div class="dropdown-divider"></div>
            <button class="dropdown-item" @click=${() => eventBus.emit('panels:toggle-all', {})}>
              <span class="dropdown-label">Toggle All Panels</span>
              <span class="dropdown-shortcut">Ctrl+Shift+P</span>
            </button>
            <div class="dropdown-divider"></div>
            <button class="dropdown-item" @click=${() => document.documentElement.requestFullscreen()}>
              <span class="dropdown-label">Fullscreen</span>
              <span class="dropdown-shortcut">F11</span>
            </button>
          </div>
        </div>

        <!-- Layout Menu -->
        <div class="menu-item">
          <button 
            class="menu-button ${this.activeDropdown === 'layout' ? 'active' : ''}"
            @click=${() => this.toggleDropdown('layout')}
          >
            Layout
          </button>
          <div class="dropdown ${this.activeDropdown === 'layout' ? 'open' : ''}">
            <button class="dropdown-item" @click=${() => this.switchLayout('explore')}>
              <span class="dropdown-label">🔍 Explore</span>
              <span class="dropdown-shortcut">Ctrl+1</span>
            </button>
            <button class="dropdown-item" @click=${() => this.switchLayout('inspect')}>
              <span class="dropdown-label">🔬 Inspect</span>
              <span class="dropdown-shortcut">Ctrl+2</span>
            </button>
            <button class="dropdown-item" @click=${() => this.switchLayout('debug')}>
              <span class="dropdown-label">🐛 Debug</span>
              <span class="dropdown-shortcut">Ctrl+3</span>
            </button>
          </div>
        </div>

        <!-- Actions Menu -->
        <div class="menu-item">
          <button 
            class="menu-button ${this.activeDropdown === 'actions' ? 'active' : ''}"
            @click=${() => this.toggleDropdown('actions')}
          >
            Actions
          </button>
          <div class="dropdown ${this.activeDropdown === 'actions' ? 'open' : ''}">
            <button class="dropdown-item" @click=${() => document.getElementById('parse-btn')?.click()}>
              <span class="dropdown-label">📁 Parse Codebase</span>
              <span class="dropdown-shortcut">Ctrl+O</span>
            </button>
            <button class="dropdown-item" @click=${() => document.getElementById('generate-btn')?.click()}>
              <span class="dropdown-label">🎨 Generate Graph</span>
              <span class="dropdown-shortcut">Ctrl+G</span>
            </button>
            <button class="dropdown-item" @click=${() => document.getElementById('connect-btn')?.click()}>
              <span class="dropdown-label">🔌 Connect Neo4j</span>
            </button>
            <div class="dropdown-divider"></div>
            <button class="dropdown-item" @click=${() => document.getElementById('reset-btn')?.click()}>
              <span class="dropdown-label">🔄 Reset View</span>
              <span class="dropdown-shortcut">R</span>
            </button>
            <button class="dropdown-item" @click=${() => document.getElementById('reset-app-btn')?.click()}>
              <span class="dropdown-label">🗑️ Reset App</span>
            </button>
          </div>
        </div>

        <!-- Help Menu -->
        <div class="menu-item">
          <button 
            class="menu-button ${this.activeDropdown === 'help' ? 'active' : ''}"
            @click=${() => this.toggleDropdown('help')}
          >
            Help
          </button>
          <div class="dropdown ${this.activeDropdown === 'help' ? 'open' : ''}">
            <button class="dropdown-item" @click=${this.openCommandPalette}>
              <span class="dropdown-label">Command Palette</span>
              <span class="dropdown-shortcut">Ctrl+K</span>
            </button>
            <button class="dropdown-item" @click=${() => this.showKeyboardShortcuts()}>
              <span class="dropdown-label">Keyboard Shortcuts</span>
              <span class="dropdown-shortcut">?</span>
            </button>
          </div>
        </div>

        <div class="separator"></div>

        <!-- Quick Actions -->
        <button 
          class="icon-button" 
          title="Command Palette (Ctrl+K)"
          @click=${this.openCommandPalette}
        >
          🔍
        </button>
        
        <button 
          class="icon-button" 
          title="Toggle Panels"
          @click=${() => eventBus.emit('panels:toggle-all', {})}
        >
          📊
        </button>

        <span class="status-text">Press Ctrl+K for commands</span>
      </div>
    `;
  }

  private showKeyboardShortcuts() {
    // Create and show a help dialog
    console.log('Show keyboard shortcuts');
    this.activeDropdown = null;
  }
}