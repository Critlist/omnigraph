import { map, atom } from 'nanostores';

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
  collapsed?: boolean;
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

// State stores
export const panelsState = map<Record<string, PanelState>>({
  'file-tree': {
    id: 'file-tree',
    mode: 'docked',
    area: 'left',
    size: { w: 280, h: 0 },
    visible: false,
    opacity: 1,
    canFloat: true,
    canTearOut: false,
    min: { w: 200, h: 300 },
    max: { w: 600 }
  },
  'properties': {
    id: 'properties',
    mode: 'docked',
    area: 'right',
    size: { w: 350, h: 0 },
    visible: false,
    opacity: 1,
    canFloat: true,
    canTearOut: false,
    min: { w: 250, h: 300 },
    max: { w: 600 }
  },
  'terminal': {
    id: 'terminal',
    mode: 'docked',
    area: 'bottom',
    size: { w: 0, h: 200 },
    visible: false,
    opacity: 1,
    canFloat: true,
    canTearOut: false,
    min: { w: 400, h: 150 },
    max: { h: 400 }
  }
});

export const activeLayout = atom<LayoutPreset>({
  name: 'explore',
  panels: panelsState.get(),
  hud: [],
  version: 1
});

export const hudElements = map<Record<string, HUDElement>>({
  'minimap': {
    id: 'minimap',
    kind: 'minimap',
    quadrant: 'br',
    priority: 1,
    fadeMs: 0,
    zoom: { min: 0, max: 2 },
    autoHide: false
  },
  'metrics': {
    id: 'metrics',
    kind: 'metric',
    quadrant: 'tr',
    priority: 1,
    fadeMs: 0,
    zoom: { min: 0, max: 2 },
    autoHide: false
  },
  'controls': {
    id: 'controls',
    kind: 'metric',
    quadrant: 'bl',
    priority: 1,
    fadeMs: 0,
    zoom: { min: 0, max: 2 },
    autoHide: false
  }
});

// Layout presets
export const layoutPresets: Record<string, LayoutPreset> = {
  explore: {
    name: 'explore',
    panels: {
      'file-tree': {
        id: 'file-tree',
        mode: 'docked',
        area: 'left',
        size: { w: 300, h: 0 },
        visible: true,
        opacity: 1
      },
      'properties': {
        id: 'properties',
        mode: 'docked',
        area: 'right',
        visible: false,
        opacity: 1
      },
      'terminal': {
        id: 'terminal',
        mode: 'docked',
        area: 'bottom',
        visible: false,
        opacity: 1
      }
    },
    hud: [
      {
        id: 'minimap',
        kind: 'minimap',
        quadrant: 'br',
        priority: 1,
        fadeMs: 0,
        zoom: { min: 0, max: 2 },
        autoHide: false
      },
      {
        id: 'breadcrumbs',
        kind: 'breadcrumb',
        quadrant: 'tl',
        priority: 1,
        fadeMs: 0,
        zoom: { min: 0, max: 2 },
        autoHide: false
      },
      {
        id: 'metrics',
        kind: 'metric',
        quadrant: 'tr',
        priority: 1,
        fadeMs: 0,
        zoom: { min: 0, max: 2 },
        autoHide: false
      }
    ],
    version: 1
  },
  inspect: {
    name: 'inspect',
    panels: {
      'file-tree': {
        id: 'file-tree',
        mode: 'docked',
        area: 'left',
        size: { w: 250, h: 0 },
        visible: true,
        opacity: 1
      },
      'properties': {
        id: 'properties',
        mode: 'docked',
        area: 'right',
        size: { w: 350, h: 0 },
        visible: true,
        opacity: 1
      },
      'terminal': {
        id: 'terminal',
        mode: 'docked',
        area: 'bottom',
        visible: false,
        opacity: 1
      }
    },
    hud: [
      {
        id: 'metrics',
        kind: 'metric',
        quadrant: 'tr',
        priority: 1,
        fadeMs: 0,
        zoom: { min: 0, max: 2 },
        autoHide: false
      }
    ],
    version: 1
  },
  debug: {
    name: 'debug',
    panels: {
      'file-tree': {
        id: 'file-tree',
        mode: 'docked',
        area: 'left',
        size: { w: 200, h: 0 },
        visible: true,
        opacity: 1
      },
      'properties': {
        id: 'properties',
        mode: 'docked',
        area: 'right',
        size: { w: 300, h: 0 },
        visible: true,
        opacity: 1
      },
      'terminal': {
        id: 'terminal',
        mode: 'docked',
        area: 'bottom',
        size: { w: 0, h: 200 },
        visible: true,
        opacity: 1
      }
    },
    hud: [
      {
        id: 'minimap',
        kind: 'minimap',
        quadrant: 'br',
        priority: 1,
        fadeMs: 0,
        zoom: { min: 0, max: 2 },
        autoHide: false
      },
      {
        id: 'metrics',
        kind: 'metric',
        quadrant: 'tr',
        priority: 1,
        fadeMs: 0,
        zoom: { min: 0, max: 2 },
        autoHide: false
      }
    ],
    version: 1
  }
};