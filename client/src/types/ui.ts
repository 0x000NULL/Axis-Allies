/**
 * UI-specific types (not part of the game engine state).
 */

export type Screen = 'mainMenu' | 'gameSetup' | 'game';

export type PlayMode = 'hotseat' | 'online' | 'singleplayer';

export interface UISelection {
  selectedTerritory: number | null;
  selectedUnit: number | null;
  hoveredTerritory: number | null;
}

export interface PanelState {
  sidePanelOpen: boolean;
  eventLogOpen: boolean;
  settingsOpen: boolean;
  saveLoadOpen: boolean;
}
