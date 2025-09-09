import mitt from 'mitt';
import type { PanelState, DockArea, HUDElement } from './panels';

export type Events = {
  'panel:mounted': { panelId: string };
  'panel:docked': { panelId: string; area: DockArea };
  'panel:floated': { panelId: string };
  'panel:closed': { panelId: string };
  'panel:resized': { panelId: string; size: { w: number; h: number } };
  'panel:collapsed': { panelId: string; collapsed: boolean };
  'panel:detached': { panelId: string };
  'panel:drag': { panelId: string; pos: { x: number; y: number } };
  'panel:toggle': { panelId: string };
  'panels:toggle-all': {};
  'hud:show': { element: HUDElement };
  'hud:hide': { elementId: string };
  'hud:fade': { elementId: string; duration: number };
  'layout:switch': { preset: string };
  'layout:save': { name: string };
  'layout:load': { name: string };
  'snap:preview': { zone: string | null };
  'snap:commit': { panelId: string; zone: string };
  'camera:change': { zoom: number; rotation: { x: number; y: number; z: number } };
  'graph:reset-view': {};
  'file:selected': { path: string; name: string; type: 'file' | 'folder'; extension?: string };
  'files:updated': { files: any[] };
  'metrics:updated': { metrics: any[]; summary: any };
  'node:selected': { nodeId: string; metrics?: any };
};

export const eventBus = mitt<Events>();