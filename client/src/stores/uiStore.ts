/**
 * UI state store. Manages selections, panels, and other UI-only state.
 */

import { create } from 'zustand';
import type { Screen, PlayMode, UISelection, PanelState } from '../types/ui';

interface UIStore {
  screen: Screen;
  playMode: PlayMode | null;
  selection: UISelection;
  panels: PanelState;
  wasmReady: boolean;

  // Actions
  setScreen: (screen: Screen) => void;
  setPlayMode: (mode: PlayMode) => void;
  selectTerritory: (id: number | null) => void;
  selectUnit: (id: number | null) => void;
  hoverTerritory: (id: number | null) => void;
  toggleSidePanel: () => void;
  toggleEventLog: () => void;
  toggleSettings: () => void;
  toggleSaveLoad: () => void;
  setWasmReady: (ready: boolean) => void;
}

export const useUIStore = create<UIStore>((set) => ({
  screen: 'mainMenu',
  playMode: null,
  selection: {
    selectedTerritory: null,
    selectedUnit: null,
    hoveredTerritory: null,
  },
  panels: {
    sidePanelOpen: true,
    eventLogOpen: false,
    settingsOpen: false,
    saveLoadOpen: false,
  },
  wasmReady: false,

  setScreen: (screen) => set({ screen }),
  setPlayMode: (mode) => set({ playMode: mode }),
  selectTerritory: (id) =>
    set((s) => ({ selection: { ...s.selection, selectedTerritory: id } })),
  selectUnit: (id) =>
    set((s) => ({ selection: { ...s.selection, selectedUnit: id } })),
  hoverTerritory: (id) =>
    set((s) => ({ selection: { ...s.selection, hoveredTerritory: id } })),
  toggleSidePanel: () =>
    set((s) => ({ panels: { ...s.panels, sidePanelOpen: !s.panels.sidePanelOpen } })),
  toggleEventLog: () =>
    set((s) => ({ panels: { ...s.panels, eventLogOpen: !s.panels.eventLogOpen } })),
  toggleSettings: () =>
    set((s) => ({ panels: { ...s.panels, settingsOpen: !s.panels.settingsOpen } })),
  toggleSaveLoad: () =>
    set((s) => ({ panels: { ...s.panels, saveLoadOpen: !s.panels.saveLoadOpen } })),
  setWasmReady: (ready) => set({ wasmReady: ready }),
}));
