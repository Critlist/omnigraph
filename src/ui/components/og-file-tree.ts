import { LitElement, html, css } from 'lit';
import { customElement, property, state } from 'lit/decorators.js';
import { eventBus } from '../../state/events';

interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'folder';
  children?: FileNode[];
  expanded?: boolean;
  size?: number;
  extension?: string;
  lineCount?: number;
}

@customElement('og-file-tree')
export class OGFileTree extends LitElement {
  @property({ type: Array }) files: FileNode[] = [];
  @state() private selectedPath: string | null = null;
  @state() private expandedFolders: Set<string> = new Set();
  @state() private searchTerm = '';
  @state() private hoveredPath: string | null = null;

  static styles = css`
    :host {
      display: flex;
      flex-direction: column;
      height: 100%;
      font-family: system-ui, -apple-system, sans-serif;
      font-size: 13px;
      color: var(--text-color, rgba(255, 255, 255, 0.9));
      user-select: none;
    }

    .search-container {
      padding: 8px;
      border-bottom: 1px solid rgba(255, 255, 255, 0.1);
      flex-shrink: 0;
    }

    .search-input {
      width: 100%;
      padding: 6px 10px;
      background: rgba(255, 255, 255, 0.05);
      border: 1px solid rgba(255, 255, 255, 0.1);
      border-radius: 4px;
      color: white;
      font-size: 12px;
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

    .tree-container {
      flex: 1;
      overflow-y: auto;
      overflow-x: hidden;
      padding: 4px 0;
    }

    .tree-item {
      display: flex;
      align-items: center;
      padding: 3px 8px;
      cursor: pointer;
      transition: background-color 0.15s;
      white-space: nowrap;
      position: relative;
    }

    .tree-item:hover {
      background: rgba(255, 255, 255, 0.05);
    }

    .tree-item.selected {
      background: var(--selection-bg, rgba(52, 152, 219, 0.3));
      color: white;
    }

    .tree-item.selected::before {
      content: '';
      position: absolute;
      left: 0;
      top: 0;
      bottom: 0;
      width: 3px;
      background: var(--focus-color, #3498db);
    }

    .tree-indent {
      display: inline-block;
      width: 16px;
    }

    .tree-arrow {
      width: 16px;
      height: 16px;
      display: flex;
      align-items: center;
      justify-content: center;
      flex-shrink: 0;
      transition: transform 0.15s;
      color: rgba(255, 255, 255, 0.5);
    }

    .tree-arrow.expanded {
      transform: rotate(90deg);
    }

    .tree-arrow.no-children {
      visibility: hidden;
    }

    .tree-icon {
      width: 16px;
      height: 16px;
      margin: 0 6px;
      flex-shrink: 0;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .tree-label {
      flex: 1;
      overflow: hidden;
      text-overflow: ellipsis;
    }

    .tree-info {
      margin-left: auto;
      padding-left: 8px;
      font-size: 11px;
      color: rgba(255, 255, 255, 0.4);
      flex-shrink: 0;
    }

    .tree-children {
      display: none;
    }

    .tree-children.expanded {
      display: block;
    }

    /* File type colors */
    .file-icon {
      font-size: 14px;
    }

    .folder-icon {
      color: #90a4ae;
    }

    .js-icon {
      color: #f7df1e;
    }

    .ts-icon {
      color: #3178c6;
    }

    .jsx-icon, .tsx-icon {
      color: #61dafb;
    }

    .py-icon {
      color: #3776ab;
    }

    .json-icon {
      color: #5a9fd4;
    }

    .css-icon {
      color: #264de4;
    }

    .html-icon {
      color: #e34c26;
    }

    .md-icon {
      color: #42a5f5;
    }

    .rust-icon {
      color: #ce422b;
    }

    .go-icon {
      color: #00add8;
    }

    .java-icon {
      color: #f89820;
    }

    /* Stats */
    .tree-stats {
      padding: 8px;
      border-top: 1px solid rgba(255, 255, 255, 0.1);
      font-size: 11px;
      color: rgba(255, 255, 255, 0.5);
      display: flex;
      gap: 12px;
      flex-shrink: 0;
    }

    .stat-item {
      display: flex;
      align-items: center;
      gap: 4px;
    }

    /* Empty state */
    .empty-state {
      padding: 24px;
      text-align: center;
      color: rgba(255, 255, 255, 0.4);
    }

    .empty-icon {
      font-size: 48px;
      margin-bottom: 12px;
      opacity: 0.3;
    }

    /* Scrollbar */
    .tree-container::-webkit-scrollbar {
      width: 6px;
    }

    .tree-container::-webkit-scrollbar-track {
      background: rgba(255, 255, 255, 0.05);
    }

    .tree-container::-webkit-scrollbar-thumb {
      background: rgba(255, 255, 255, 0.2);
      border-radius: 3px;
    }

    .tree-container::-webkit-scrollbar-thumb:hover {
      background: rgba(255, 255, 255, 0.3);
    }

    /* Loading skeleton */
    @keyframes shimmer {
      0% { background-position: -200% 0; }
      100% { background-position: 200% 0; }
    }

    .skeleton {
      background: linear-gradient(
        90deg,
        rgba(255, 255, 255, 0.05) 25%,
        rgba(255, 255, 255, 0.1) 50%,
        rgba(255, 255, 255, 0.05) 75%
      );
      background-size: 200% 100%;
      animation: shimmer 1.5s infinite;
      border-radius: 4px;
      height: 20px;
      margin: 4px 8px;
    }
  `;

  connectedCallback() {
    super.connectedCallback();
    
    // Listen for file system updates
    eventBus.on('files:updated', ({ files }) => {
      console.log('📁 File tree received files:', files);
      this.files = this.transformFiles(files);
      this.requestUpdate();
    });
    
    // Only load mock data if in development mode without parsed data
    if (!this.files || this.files.length === 0) {
      // Don't auto-load mock data - wait for real parsed data
      console.log('📁 File tree waiting for parsed codebase...');
    }
  }

  private transformFiles(files: any[]): FileNode[] {
    // Transform the files from backend format to our FileNode format
    return files.map(file => this.transformFileNode(file));
  }

  private transformFileNode(file: any): FileNode {
    return {
      name: file.name,
      path: file.path,
      type: file.type === 'folder' ? 'folder' : 'file',
      children: file.children ? file.children.map((child: any) => this.transformFileNode(child)) : undefined,
      expanded: false,
      size: file.size,
      extension: file.extension,
      lineCount: file.line_count || file.lineCount
    };
  }

  private loadMockData() {
    // Mock file tree for demonstration
    this.files = [
      {
        name: 'src',
        path: '/src',
        type: 'folder',
        children: [
          {
            name: 'main.ts',
            path: '/src/main.ts',
            type: 'file',
            extension: 'ts',
            size: 12543,
            lineCount: 498
          },
          {
            name: 'ui',
            path: '/src/ui',
            type: 'folder',
            children: [
              {
                name: 'components',
                path: '/src/ui/components',
                type: 'folder',
                children: [
                  {
                    name: 'og-panel.ts',
                    path: '/src/ui/components/og-panel.ts',
                    type: 'file',
                    extension: 'ts',
                    size: 5234,
                    lineCount: 156
                  },
                  {
                    name: 'og-float-panel.ts',
                    path: '/src/ui/components/og-float-panel.ts',
                    type: 'file',
                    extension: 'ts',
                    size: 7891,
                    lineCount: 287
                  },
                  {
                    name: 'og-command-palette.ts',
                    path: '/src/ui/components/og-command-palette.ts',
                    type: 'file',
                    extension: 'ts',
                    size: 9456,
                    lineCount: 412
                  }
                ]
              },
              {
                name: 'styles',
                path: '/src/ui/styles',
                type: 'folder',
                children: [
                  {
                    name: 'panels.css',
                    path: '/src/ui/styles/panels.css',
                    type: 'file',
                    extension: 'css',
                    size: 15234,
                    lineCount: 723
                  }
                ]
              }
            ]
          },
          {
            name: 'visualization',
            path: '/src/visualization',
            type: 'folder',
            children: [
              {
                name: 'graph3d.ts',
                path: '/src/visualization/graph3d.ts',
                type: 'file',
                extension: 'ts',
                size: 8234,
                lineCount: 312
              }
            ]
          },
          {
            name: 'state',
            path: '/src/state',
            type: 'folder',
            children: [
              {
                name: 'panels.ts',
                path: '/src/state/panels.ts',
                type: 'file',
                extension: 'ts',
                size: 4567,
                lineCount: 189
              },
              {
                name: 'events.ts',
                path: '/src/state/events.ts',
                type: 'file',
                extension: 'ts',
                size: 1234,
                lineCount: 45
              }
            ]
          }
        ]
      },
      {
        name: 'crates',
        path: '/crates',
        type: 'folder',
        children: [
          {
            name: 'og-parser',
            path: '/crates/og-parser',
            type: 'folder',
            children: [
              {
                name: 'src',
                path: '/crates/og-parser/src',
                type: 'folder',
                children: [
                  {
                    name: 'lib.rs',
                    path: '/crates/og-parser/src/lib.rs',
                    type: 'file',
                    extension: 'rs',
                    size: 234,
                    lineCount: 12
                  },
                  {
                    name: 'javascript.rs',
                    path: '/crates/og-parser/src/javascript.rs',
                    type: 'file',
                    extension: 'rs',
                    size: 18234,
                    lineCount: 567
                  },
                  {
                    name: 'typescript.rs',
                    path: '/crates/og-parser/src/typescript.rs',
                    type: 'file',
                    extension: 'rs',
                    size: 19456,
                    lineCount: 612
                  },
                  {
                    name: 'python.rs',
                    path: '/crates/og-parser/src/python.rs',
                    type: 'file',
                    extension: 'rs',
                    size: 15678,
                    lineCount: 489
                  }
                ]
              }
            ]
          }
        ]
      },
      {
        name: 'package.json',
        path: '/package.json',
        type: 'file',
        extension: 'json',
        size: 2234,
        lineCount: 89
      },
      {
        name: 'tsconfig.json',
        path: '/tsconfig.json',
        type: 'file',
        extension: 'json',
        size: 567,
        lineCount: 27
      },
      {
        name: 'README.md',
        path: '/README.md',
        type: 'file',
        extension: 'md',
        size: 6543,
        lineCount: 198
      }
    ];
  }

  private toggleFolder(path: string) {
    if (this.expandedFolders.has(path)) {
      this.expandedFolders.delete(path);
    } else {
      this.expandedFolders.add(path);
    }
    this.requestUpdate();
  }

  private selectFile(node: FileNode) {
    this.selectedPath = node.path;
    
    // Emit selection event
    eventBus.emit('file:selected', { 
      path: node.path,
      name: node.name,
      type: node.type,
      extension: node.extension
    });

    // If it's a folder, toggle it
    if (node.type === 'folder') {
      this.toggleFolder(node.path);
    }
  }

  private getFileIcon(node: FileNode): string {
    if (node.type === 'folder') {
      return this.expandedFolders.has(node.path) ? '📂' : '📁';
    }

    const ext = node.extension?.toLowerCase();
    const iconMap: Record<string, string> = {
      'js': '📜',
      'jsx': '⚛️',
      'ts': '📘',
      'tsx': '⚛️',
      'py': '🐍',
      'rs': '🦀',
      'go': '🐹',
      'java': '☕',
      'json': '📋',
      'css': '🎨',
      'html': '🌐',
      'md': '📝',
      'txt': '📄',
      'yml': '⚙️',
      'yaml': '⚙️',
      'toml': '⚙️',
      'lock': '🔒',
      'gitignore': '🚫',
      'env': '🔐'
    };

    return iconMap[ext || ''] || '📄';
  }

  private getFileClass(extension?: string): string {
    if (!extension) return '';
    
    const classMap: Record<string, string> = {
      'js': 'js-icon',
      'jsx': 'jsx-icon',
      'ts': 'ts-icon',
      'tsx': 'tsx-icon',
      'py': 'py-icon',
      'rs': 'rust-icon',
      'go': 'go-icon',
      'java': 'java-icon',
      'json': 'json-icon',
      'css': 'css-icon',
      'html': 'html-icon',
      'md': 'md-icon'
    };

    return classMap[extension.toLowerCase()] || '';
  }

  private formatSize(bytes?: number): string {
    if (!bytes) return '';
    
    if (bytes < 1024) return `${bytes}B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
  }

  private filterNodes(nodes: FileNode[]): FileNode[] {
    if (!this.searchTerm) return nodes;
    
    const term = this.searchTerm.toLowerCase();
    
    const filterRecursive = (node: FileNode): FileNode | null => {
      const nameMatches = node.name.toLowerCase().includes(term);
      
      if (node.type === 'file') {
        return nameMatches ? node : null;
      }
      
      // For folders, check children
      const filteredChildren = node.children
        ?.map(child => filterRecursive(child))
        .filter(Boolean) as FileNode[] | undefined;
      
      if (nameMatches || (filteredChildren && filteredChildren.length > 0)) {
        return {
          ...node,
          children: filteredChildren
        };
      }
      
      return null;
    };
    
    return nodes
      .map(node => filterRecursive(node))
      .filter(Boolean) as FileNode[];
  }

  private renderTree(nodes: FileNode[], depth = 0): any {
    const filtered = this.filterNodes(nodes);
    
    return html`
      ${filtered.map(node => {
        const isExpanded = this.expandedFolders.has(node.path);
        const hasChildren = node.children && node.children.length > 0;
        const isSelected = this.selectedPath === node.path;
        const isHovered = this.hoveredPath === node.path;
        
        return html`
          <div>
            <div
              class="tree-item ${isSelected ? 'selected' : ''} ${isHovered ? 'hovered' : ''}"
              @click=${() => this.selectFile(node)}
              @mouseenter=${() => this.hoveredPath = node.path}
              @mouseleave=${() => this.hoveredPath = null}
              style="padding-left: ${depth * 16 + 8}px"
            >
              <span class="tree-arrow ${isExpanded ? 'expanded' : ''} ${!hasChildren ? 'no-children' : ''}">
                ${hasChildren ? '▶' : ''}
              </span>
              <span class="tree-icon file-icon ${node.type === 'folder' ? 'folder-icon' : this.getFileClass(node.extension)}">
                ${this.getFileIcon(node)}
              </span>
              <span class="tree-label">${node.name}</span>
              <span class="tree-info">
                ${node.type === 'file' ? html`
                  ${node.lineCount ? html`<span>${node.lineCount}L</span>` : ''}
                  ${node.size ? html`<span>${this.formatSize(node.size)}</span>` : ''}
                ` : html`
                  ${node.children ? html`<span>${node.children.length}</span>` : ''}
                `}
              </span>
            </div>
            ${hasChildren && isExpanded ? html`
              <div class="tree-children expanded">
                ${this.renderTree(node.children!, depth + 1)}
              </div>
            ` : ''}
          </div>
        `;
      })}
    `;
  }

  private countFiles(nodes: FileNode[]): { files: number; folders: number } {
    let files = 0;
    let folders = 0;
    
    const count = (node: FileNode) => {
      if (node.type === 'file') {
        files++;
      } else {
        folders++;
      }
      
      if (node.children) {
        node.children.forEach(count);
      }
    };
    
    nodes.forEach(count);
    return { files, folders };
  }

  render() {
    const stats = this.countFiles(this.files);
    
    return html`
      <div class="search-container">
        <input
          type="text"
          class="search-input"
          placeholder="Search files..."
          .value=${this.searchTerm}
          @input=${(e: Event) => {
            this.searchTerm = (e.target as HTMLInputElement).value;
          }}
        />
      </div>
      
      <div class="tree-container">
        ${this.files && this.files.length > 0 ? this.renderTree(this.files) : html`
          <div class="empty-state">
            <div class="empty-icon">📁</div>
            <div>No files loaded</div>
            <div style="margin-top: 8px; font-size: 11px;">
              Parse a codebase to see files here
            </div>
          </div>
        `}
      </div>
      
      <div class="tree-stats">
        <div class="stat-item">
          📄 ${stats.files} files
        </div>
        <div class="stat-item">
          📁 ${stats.folders} folders
        </div>
      </div>
    `;
  }
}