/**
 * Core game state store. Synced from the WASM engine.
 * The engine is the single source of truth; this store is a React-friendly mirror.
 */

import { create } from 'zustand';
import type { GameState, Action, ActionResult, GameEvent } from '../types/game';
import type { GameEngine } from '../wasm/engine';

interface GameStore {
  // State
  engine: GameEngine | null;
  gameState: GameState | null;
  events: GameEvent[];
  isLoading: boolean;
  error: string | null;

  // Actions
  setEngine: (engine: GameEngine) => void;
  syncState: () => void;
  submitAction: (action: Action) => ActionResult | null;
  undo: () => ActionResult | null;
  canUndo: () => boolean;
  clearError: () => void;
}

export const useGameStore = create<GameStore>((set, get) => ({
  engine: null,
  gameState: null,
  events: [],
  isLoading: false,
  error: null,

  setEngine: (engine: GameEngine) => {
    set({ engine, gameState: engine.getState(), error: null });
  },

  syncState: () => {
    const { engine } = get();
    if (engine) {
      set({ gameState: engine.getState() });
    }
  },

  submitAction: (action: Action): ActionResult | null => {
    const { engine } = get();
    if (!engine) {
      set({ error: 'Engine not initialized' });
      return null;
    }

    try {
      const result = engine.submitAction(action);
      // Sync state from engine after action
      set({
        gameState: engine.getState(),
        events: [...get().events, ...result.events],
        error: null,
      });
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Unknown error';
      set({ error: message });
      return null;
    }
  },

  undo: (): ActionResult | null => {
    const { engine } = get();
    if (!engine) {
      set({ error: 'Engine not initialized' });
      return null;
    }

    try {
      const result = engine.undo();
      set({
        gameState: engine.getState(),
        events: [...get().events, ...result.events],
        error: null,
      });
      return result;
    } catch (e) {
      const message = e instanceof Error ? e.message : 'Unknown error';
      set({ error: message });
      return null;
    }
  },

  canUndo: (): boolean => {
    const { engine } = get();
    return engine ? engine.canUndo() : false;
  },

  clearError: () => set({ error: null }),
}));
