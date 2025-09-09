import { LitElement, html, css } from 'lit';
import { customElement, state } from 'lit/decorators.js';
import { eventBus } from '../../state/events';

interface FileInfo {
  path: string;
  name: string;
  type: 'file' | 'folder';
  extension?: string;
  size?: number;
  lines?: number;
  complexity?: number;
  imports?: string[];
  exports?: string[];
  classes?: string[];
  functions?: string[];
}

@customElement('og-properties-panel')
export class OGPropertiesPanel extends LitElement {
  @state() private selectedFile: FileInfo | null = null;
  @state() private activeTab: 'info' | 'dependencies' | 'metrics' = 'info';

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      font-family: system-ui, -apple-system, sans-serif;
      font-size: 13px;
      color: var(--text-color, rgba(255, 255, 255, 0.9));
    }

    .tabs {
      display: flex;
      border-bottom: 1px solid rgba(255, 255, 255, 0.1);
      flex-shrink: 0;
    }

    .tab {
      padding: 8px 16px;
      background: transparent;
      border: none;
      border-bottom: 2px solid transparent;
      color: rgba(255, 255, 255, 0.6);
      cursor: pointer;
      font-size: 12px;
      transition: all 0.2s;
    }

    .tab:hover {
      color: rgba(255, 255, 255, 0.8);
      background: rgba(255, 255, 255, 0.05);
    }

    .tab.active {
      color: white;
      border-bottom-color: var(--focus-color, #3498db);
    }

    .content {
      flex: 1;
      overflow-y: auto;
      padding: 16px;
    }

    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      height: 100%;
      color: rgba(255, 255, 255, 0.4);
      text-align: center;
    }

    .empty-icon {
      font-size: 48px;
      margin-bottom: 12px;
      opacity: 0.3;
    }

    .property-group {
      margin-bottom: 20px;
    }

    .property-title {
      font-size: 11px;
      font-weight: 600;
      text-transform: uppercase;
      letter-spacing: 0.5px;
      color: rgba(255, 255, 255, 0.5);
      margin-bottom: 8px;
    }

    .property-row {
      display: flex;
      justify-content: space-between;
      padding: 4px 0;
      font-size: 12px;
    }

    .property-label {
      color: rgba(255, 255, 255, 0.6);
    }

    .property-value {
      color: white;
      font-family: monospace;
    }

    .list-item {
      padding: 4px 8px;
      margin: 2px 0;
      background: rgba(255, 255, 255, 0.03);
      border-radius: 4px;
      font-size: 12px;
      font-family: monospace;
      color: rgba(255, 255, 255, 0.8);
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .list-item:hover {
      background: rgba(255, 255, 255, 0.06);
      color: white;
      cursor: pointer;
    }

    .list-icon {
      font-size: 14px;
    }

    .metrics-grid {
      display: grid;
      grid-template-columns: repeat(2, 1fr);
      gap: 12px;
    }

    .metric-card {
      background: rgba(255, 255, 255, 0.03);
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 8px;
      padding: 12px;
      text-align: center;
    }

    .metric-value {
      font-size: 24px;
      font-weight: bold;
      color: white;
      margin-bottom: 4px;
    }

    .metric-label {
      font-size: 11px;
      color: rgba(255, 255, 255, 0.5);
      text-transform: uppercase;
    }

    .complexity-indicator {
      display: inline-block;
      width: 8px;
      height: 8px;
      border-radius: 50%;
      margin-left: 8px;
    }

    .complexity-low {
      background: #2ecc71;
    }

    .complexity-medium {
      background: #f39c12;
    }

    .complexity-high {
      background: #e74c3c;
    }

    .path-breadcrumb {
      padding: 8px 12px;
      background: rgba(255, 255, 255, 0.03);
      border-radius: 4px;
      font-family: monospace;
      font-size: 11px;
      color: rgba(255, 255, 255, 0.7);
      word-break: break-all;
      margin-bottom: 16px;
    }

    /* Scrollbar */
    .content::-webkit-scrollbar {
      width: 6px;
    }

    .content::-webkit-scrollbar-track {
      background: rgba(255, 255, 255, 0.05);
    }

    .content::-webkit-scrollbar-thumb {
      background: rgba(255, 255, 255, 0.2);
      border-radius: 3px;
    }

    .content::-webkit-scrollbar-thumb:hover {
      background: rgba(255, 255, 255, 0.3);
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    
    // Listen for file selection
    eventBus.on('file:selected', ({ path, name, type, extension }) => {
      // Simulate fetching detailed file info
      this.selectedFile = {
        path,
        name,
        type,
        extension,
        size: Math.floor(Math.random() * 50000),
        lines: Math.floor(Math.random() * 500) + 50,
        complexity: Math.floor(Math.random() * 100),
        imports: type === 'file' ? [
          './utils/helper',
          '../state/store',
          'react',
          'lodash/debounce'
        ] : undefined,
        exports: type === 'file' ? [
          'default MyComponent',
          'useCustomHook',
          'CONSTANTS'
        ] : undefined,
        classes: type === 'file' && extension === 'ts' ? [
          'GraphNode',
          'GraphManager',
          'Parser'
        ] : undefined,
        functions: type === 'file' ? [
          'initialize()',
          'processData()',
          'renderGraph()',
          'handleClick()'
        ] : undefined
      };
    });
  }

  private getComplexityClass(complexity?: number): string {
    if (!complexity) return '';
    if (complexity < 30) return 'complexity-low';
    if (complexity < 70) return 'complexity-medium';
    return 'complexity-high';
  }

  private getComplexityLabel(complexity?: number): string {
    if (!complexity) return 'Unknown';
    if (complexity < 30) return 'Low';
    if (complexity < 70) return 'Medium';
    return 'High';
  }

  private formatSize(bytes?: number): string {
    if (!bytes) return '0B';
    
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)}MB`;
  }

  private renderInfoTab() {
    if (!this.selectedFile) return '';

    return html`
      <div class="path-breadcrumb">
        ${this.selectedFile.path}
      </div>

      <div class="property-group">
        <div class="property-title">General</div>
        <div class="property-row">
          <span class="property-label">Name</span>
          <span class="property-value">${this.selectedFile.name}</span>
        </div>
        <div class="property-row">
          <span class="property-label">Type</span>
          <span class="property-value">${this.selectedFile.type}</span>
        </div>
        ${this.selectedFile.extension ? html`
          <div class="property-row">
            <span class="property-label">Extension</span>
            <span class="property-value">.${this.selectedFile.extension}</span>
          </div>
        ` : ''}
        <div class="property-row">
          <span class="property-label">Size</span>
          <span class="property-value">${this.formatSize(this.selectedFile.size)}</span>
        </div>
        ${this.selectedFile.lines ? html`
          <div class="property-row">
            <span class="property-label">Lines</span>
            <span class="property-value">${this.selectedFile.lines}</span>
          </div>
        ` : ''}
      </div>

      ${this.selectedFile.classes && this.selectedFile.classes.length > 0 ? html`
        <div class="property-group">
          <div class="property-title">Classes (${this.selectedFile.classes.length})</div>
          ${this.selectedFile.classes.map(cls => html`
            <div class="list-item">
              <span class="list-icon">🏛️</span>
              <span>${cls}</span>
            </div>
          `)}
        </div>
      ` : ''}

      ${this.selectedFile.functions && this.selectedFile.functions.length > 0 ? html`
        <div class="property-group">
          <div class="property-title">Functions (${this.selectedFile.functions.length})</div>
          ${this.selectedFile.functions.map(func => html`
            <div class="list-item">
              <span class="list-icon">⚡</span>
              <span>${func}</span>
            </div>
          `)}
        </div>
      ` : ''}
    `;
  }

  private renderDependenciesTab() {
    if (!this.selectedFile || this.selectedFile.type !== 'file') {
      return html`
        <div class="empty-state">
          <div class="empty-icon">🔗</div>
          <div>No dependencies data available</div>
        </div>
      `;
    }

    return html`
      ${this.selectedFile.imports && this.selectedFile.imports.length > 0 ? html`
        <div class="property-group">
          <div class="property-title">Imports (${this.selectedFile.imports.length})</div>
          ${this.selectedFile.imports.map(imp => html`
            <div class="list-item">
              <span class="list-icon">📥</span>
              <span>${imp}</span>
            </div>
          `)}
        </div>
      ` : ''}

      ${this.selectedFile.exports && this.selectedFile.exports.length > 0 ? html`
        <div class="property-group">
          <div class="property-title">Exports (${this.selectedFile.exports.length})</div>
          ${this.selectedFile.exports.map(exp => html`
            <div class="list-item">
              <span class="list-icon">📤</span>
              <span>${exp}</span>
            </div>
          `)}
        </div>
      ` : ''}
    `;
  }

  private renderMetricsTab() {
    if (!this.selectedFile || this.selectedFile.type !== 'file') {
      return html`
        <div class="empty-state">
          <div class="empty-icon">📊</div>
          <div>No metrics available</div>
        </div>
      `;
    }

    return html`
      <div class="metrics-grid">
        <div class="metric-card">
          <div class="metric-value">${this.selectedFile.lines || 0}</div>
          <div class="metric-label">Lines of Code</div>
        </div>
        <div class="metric-card">
          <div class="metric-value">
            ${this.selectedFile.complexity || 0}
            <span class="complexity-indicator ${this.getComplexityClass(this.selectedFile.complexity)}"></span>
          </div>
          <div class="metric-label">Complexity</div>
        </div>
        <div class="metric-card">
          <div class="metric-value">${this.selectedFile.imports?.length || 0}</div>
          <div class="metric-label">Dependencies</div>
        </div>
        <div class="metric-card">
          <div class="metric-value">${this.selectedFile.exports?.length || 0}</div>
          <div class="metric-label">Exports</div>
        </div>
      </div>

      <div class="property-group" style="margin-top: 20px;">
        <div class="property-title">Code Quality</div>
        <div class="property-row">
          <span class="property-label">Complexity</span>
          <span class="property-value">
            ${this.getComplexityLabel(this.selectedFile.complexity)}
            <span class="complexity-indicator ${this.getComplexityClass(this.selectedFile.complexity)}"></span>
          </span>
        </div>
        <div class="property-row">
          <span class="property-label">Maintainability</span>
          <span class="property-value">Good</span>
        </div>
        <div class="property-row">
          <span class="property-label">Test Coverage</span>
          <span class="property-value">78%</span>
        </div>
      </div>
    `;
  }

  render() {
    if (!this.selectedFile) {
      return html`
        <div class="empty-state">
          <div class="empty-icon">📋</div>
          <div>No file selected</div>
          <div style="margin-top: 8px; font-size: 11px; color: rgba(255,255,255,0.3);">
            Select a file from the explorer to see its properties
          </div>
        </div>
      `;
    }

    return html`
      <div class="tabs">
        <button 
          class="tab ${this.activeTab === 'info' ? 'active' : ''}"
          @click=${() => this.activeTab = 'info'}
        >
          Info
        </button>
        <button 
          class="tab ${this.activeTab === 'dependencies' ? 'active' : ''}"
          @click=${() => this.activeTab = 'dependencies'}
        >
          Dependencies
        </button>
        <button 
          class="tab ${this.activeTab === 'metrics' ? 'active' : ''}"
          @click=${() => this.activeTab = 'metrics'}
        >
          Metrics
        </button>
      </div>

      <div class="content">
        ${this.activeTab === 'info' ? this.renderInfoTab() : ''}
        ${this.activeTab === 'dependencies' ? this.renderDependenciesTab() : ''}
        ${this.activeTab === 'metrics' ? this.renderMetricsTab() : ''}
      </div>
    `;
  }
}