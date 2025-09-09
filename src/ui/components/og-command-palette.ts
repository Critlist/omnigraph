import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { eventBus } from '../../state/events';

interface Command {
  id: string;
  label: string;
  shortcut?: string;
  category: string;
  action: () => void;
}

@customElement('og-command-palette')
export class OGCommandPalette extends LitElement {
  @property({ type: Boolean }) open = false;
  @state() private searchTerm = '';
  @state() private selectedIndex = 0;
  @state() private commands: Command[] = [];
  
  static styles = css`
    :host {
      position: fixed;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      z-index: 2000;
      pointer-events: none;
    }

    .overlay {
      position: absolute;
      top: 0;
      left: 0;
      right: 0;
      bottom: 0;
      background: rgba(0, 0, 0, 0.5);
      backdrop-filter: blur(4px);
      opacity: 0;
      transition: opacity 0.2s;
      pointer-events: none;
    }

    .overlay.open {
      opacity: 1;
      pointer-events: auto;
    }

    .palette {
      position: absolute;
      top: 20%;
      left: 50%;
      transform: translateX(-50%) scale(0.95);
      width: 600px;
      max-width: 90vw;
      max-height: 400px;
      background: var(--panel-bg, rgba(30, 30, 40, 0.98));
      backdrop-filter: blur(20px);
      border: 1px solid var(--panel-border, rgba(255, 255, 255, 0.2));
      border-radius: 12px;
      box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
      opacity: 0;
      transition: all 0.2s;
      pointer-events: none;
      display: flex;
      flex-direction: column;
    }

    .palette.open {
      opacity: 1;
      transform: translateX(-50%) scale(1);
      pointer-events: auto;
    }

    .search-container {
      padding: 16px;
      border-bottom: 1px solid var(--panel-border, rgba(255, 255, 255, 0.1));
    }

    .search-input {
      width: 100%;
      padding: 12px 16px;
      background: rgba(255, 255, 255, 0.05);
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 8px;
      color: white;
      font-size: 16px;
      outline: none;
      transition: all 0.2s;
    }

    .search-input:focus {
      background: rgba(255, 255, 255, 0.08);
      border-color: var(--focus-color, #3498db);
    }

    .search-input::placeholder {
      color: rgba(255, 255, 255, 0.4);
    }

    .commands-list {
      flex: 1;
      overflow-y: auto;
      padding: 8px;
    }

    .category {
      padding: 8px 12px;
      font-size: 11px;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: rgba(255, 255, 255, 0.4);
      font-weight: 600;
    }

    .command {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 10px 16px;
      margin: 2px 0;
      background: transparent;
      border: none;
      border-radius: 6px;
      color: rgba(255, 255, 255, 0.9);
      cursor: pointer;
      transition: all 0.15s;
      width: 100%;
      text-align: left;
    }

    .command:hover {
      background: rgba(255, 255, 255, 0.08);
    }

    .command.selected {
      background: var(--focus-color, #3498db);
      color: white;
    }

    .command-label {
      flex: 1;
      font-size: 14px;
    }

    .command-shortcut {
      font-size: 12px;
      padding: 2px 6px;
      background: rgba(255, 255, 255, 0.1);
      border-radius: 4px;
      font-family: monospace;
      color: rgba(255, 255, 255, 0.6);
    }

    .command.selected .command-shortcut {
      background: rgba(255, 255, 255, 0.2);
      color: white;
    }

    .footer {
      padding: 12px 16px;
      border-top: 1px solid var(--panel-border, rgba(255, 255, 255, 0.1));
      display: flex;
      gap: 16px;
      font-size: 12px;
      color: rgba(255, 255, 255, 0.5);
    }

    .footer-hint {
      display: flex;
      align-items: center;
      gap: 4px;
    }

    .key {
      padding: 2px 4px;
      background: rgba(255, 255, 255, 0.1);
      border-radius: 3px;
      font-family: monospace;
      font-size: 11px;
    }

    /* Scrollbar styling */
    .commands-list::-webkit-scrollbar {
      width: 6px;
    }

    .commands-list::-webkit-scrollbar-track {
      background: rgba(255, 255, 255, 0.05);
      border-radius: 3px;
    }

    .commands-list::-webkit-scrollbar-thumb {
      background: rgba(255, 255, 255, 0.2);
      border-radius: 3px;
    }

    .commands-list::-webkit-scrollbar-thumb:hover {
      background: rgba(255, 255, 255, 0.3);
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    this.initializeCommands();
    document.addEventListener('keydown', this.handleGlobalKeydown);
  }

  disconnectedCallback() {
    super.disconnectedCallback();
    document.removeEventListener('keydown', this.handleGlobalKeydown);
  }

  private initializeCommands() {
    this.commands = [
      // Panel Commands
      {
        id: 'toggle-left-panel',
        label: 'Toggle Left Panel (Explorer)',
        shortcut: 'Ctrl+B',
        category: 'Panels',
        action: () => {
          eventBus.emit('panel:toggle', { panelId: 'file-tree' });
          this.close();
        }
      },
      {
        id: 'toggle-right-panel',
        label: 'Toggle Right Panel (Properties)',
        shortcut: 'Ctrl+\\',
        category: 'Panels',
        action: () => {
          eventBus.emit('panel:toggle', { panelId: 'properties' });
          this.close();
        }
      },
      {
        id: 'toggle-bottom-panel',
        label: 'Toggle Bottom Panel (Terminal)',
        shortcut: 'Ctrl+J',
        category: 'Panels',
        action: () => {
          eventBus.emit('panel:toggle', { panelId: 'terminal' });
          this.close();
        }
      },
      // Layout Commands
      {
        id: 'layout-explore',
        label: 'Switch to Explore Layout',
        shortcut: 'Ctrl+1',
        category: 'Layouts',
        action: () => {
          eventBus.emit('layout:switch', { preset: 'explore' });
          this.close();
        }
      },
      {
        id: 'layout-inspect',
        label: 'Switch to Inspect Layout',
        shortcut: 'Ctrl+2',
        category: 'Layouts',
        action: () => {
          eventBus.emit('layout:switch', { preset: 'inspect' });
          this.close();
        }
      },
      {
        id: 'layout-debug',
        label: 'Switch to Debug Layout',
        shortcut: 'Ctrl+3',
        category: 'Layouts',
        action: () => {
          eventBus.emit('layout:switch', { preset: 'debug' });
          this.close();
        }
      },
      // View Commands
      {
        id: 'toggle-all-panels',
        label: 'Toggle All Panels',
        shortcut: 'Ctrl+Shift+P',
        category: 'View',
        action: () => {
          eventBus.emit('panels:toggle-all', {});
          this.close();
        }
      },
      {
        id: 'reset-view',
        label: 'Reset 3D View',
        shortcut: 'R',
        category: 'View',
        action: () => {
          eventBus.emit('graph:reset-view', {});
          this.close();
        }
      },
      {
        id: 'fullscreen',
        label: 'Toggle Fullscreen',
        shortcut: 'F11',
        category: 'View',
        action: () => {
          if (!document.fullscreenElement) {
            document.documentElement.requestFullscreen();
          } else {
            document.exitFullscreen();
          }
          this.close();
        }
      },
      // Actions
      {
        id: 'parse-codebase',
        label: 'Parse Codebase',
        shortcut: 'Ctrl+O',
        category: 'Actions',
        action: () => {
          document.getElementById('parse-btn')?.click();
          this.close();
        }
      },
      {
        id: 'generate-graph',
        label: 'Generate Graph',
        shortcut: 'Ctrl+G',
        category: 'Actions',
        action: () => {
          document.getElementById('generate-btn')?.click();
          this.close();
        }
      },
      {
        id: 'help',
        label: 'Show Keyboard Shortcuts',
        shortcut: '?',
        category: 'Help',
        action: () => {
          this.showHelp();
          this.close();
        }
      }
    ];
  }

  private handleGlobalKeydown = (e: KeyboardEvent) => {
    // Open palette with Ctrl+Shift+P or Ctrl+K
    if ((e.ctrlKey && e.shiftKey && e.key === 'P') || (e.ctrlKey && e.key === 'k')) {
      e.preventDefault();
      this.toggle();
    }

    // Handle navigation when open
    if (this.open) {
      if (e.key === 'Escape') {
        e.preventDefault();
        this.close();
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        this.navigateDown();
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        this.navigateUp();
      } else if (e.key === 'Enter') {
        e.preventDefault();
        this.executeSelected();
      }
    }
  };

  private toggle() {
    this.open = !this.open;
    if (this.open) {
      this.searchTerm = '';
      this.selectedIndex = 0;
      // Focus search input after render
      this.updateComplete.then(() => {
        const input = this.shadowRoot?.querySelector('.search-input') as HTMLInputElement;
        input?.focus();
      });
    }
  }

  private close() {
    this.open = false;
  }

  private navigateDown() {
    const filtered = this.getFilteredCommands();
    this.selectedIndex = Math.min(this.selectedIndex + 1, filtered.length - 1);
    this.scrollToSelected();
  }

  private navigateUp() {
    this.selectedIndex = Math.max(this.selectedIndex - 1, 0);
    this.scrollToSelected();
  }

  private scrollToSelected() {
    this.updateComplete.then(() => {
      const selected = this.shadowRoot?.querySelector('.command.selected');
      selected?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
    });
  }

  private executeSelected() {
    const filtered = this.getFilteredCommands();
    const command = filtered[this.selectedIndex];
    if (command) {
      command.action();
    }
  }

  private getFilteredCommands(): Command[] {
    if (!this.searchTerm) return this.commands;
    
    const term = this.searchTerm.toLowerCase();
    return this.commands.filter(cmd => 
      cmd.label.toLowerCase().includes(term) ||
      cmd.category.toLowerCase().includes(term) ||
      cmd.shortcut?.toLowerCase().includes(term)
    );
  }

  private groupCommandsByCategory(): Map<string, Command[]> {
    const grouped = new Map<string, Command[]>();
    const filtered = this.getFilteredCommands();
    
    filtered.forEach(cmd => {
      const group = grouped.get(cmd.category) || [];
      group.push(cmd);
      grouped.set(cmd.category, group);
    });
    
    return grouped;
  }

  private handleSearch(e: Event) {
    this.searchTerm = (e.target as HTMLInputElement).value;
    this.selectedIndex = 0;
  }

  private showHelp() {
    // Will implement a help dialog
    console.log('Show help');
  }

  render() {
    const grouped = this.groupCommandsByCategory();
    let currentIndex = 0;

    return html`
      <div class="overlay ${this.open ? 'open' : ''}" @click=${this.close}>
        <div class="palette ${this.open ? 'open' : ''}" @click=${(e: Event) => e.stopPropagation()}>
          <div class="search-container">
            <input
              type="text"
              class="search-input"
              placeholder="Search commands..."
              .value=${this.searchTerm}
              @input=${this.handleSearch}
            />
          </div>
          
          <div class="commands-list">
            ${Array.from(grouped.entries()).map(([category, commands]) => html`
              <div class="category">${category}</div>
              ${commands.map(cmd => {
                const isSelected = currentIndex++ === this.selectedIndex;
                return html`
                  <button
                    class="command ${isSelected ? 'selected' : ''}"
                    @click=${() => cmd.action()}
                    @mouseenter=${() => this.selectedIndex = currentIndex - 1}
                  >
                    <span class="command-label">${cmd.label}</span>
                    ${cmd.shortcut ? html`
                      <span class="command-shortcut">${cmd.shortcut}</span>
                    ` : ''}
                  </button>
                `;
              })}
            `)}
          </div>
          
          <div class="footer">
            <div class="footer-hint">
              <span class="key">↑↓</span> Navigate
            </div>
            <div class="footer-hint">
              <span class="key">Enter</span> Execute
            </div>
            <div class="footer-hint">
              <span class="key">Esc</span> Close
            </div>
          </div>
        </div>
      </div>
    `;
  }
}