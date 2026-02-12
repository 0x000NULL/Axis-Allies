/**
 * TypeScript wrapper around the WASM game engine.
 * Provides a typed API for the Zustand store to call.
 */

import type { WasmEngine } from './pkg/aa_wasm';
import type { GameState, Action, ActionResult, LegalAction, GameEvent } from '../types/game';

export class GameEngine {
  private engine: WasmEngine;

  constructor(engine: WasmEngine) {
    this.engine = engine;
  }

  getState(): GameState {
    const json = this.engine.getState();
    return JSON.parse(json) as GameState;
  }

  submitAction(action: Action): ActionResult {
    const json = this.engine.submitAction(JSON.stringify(action));
    const result = JSON.parse(json);
    if (result.error) {
      throw new Error(result.message);
    }
    return result as ActionResult;
  }

  undo(): ActionResult {
    return this.submitAction('Undo');
  }

  canUndo(): boolean {
    return this.engine.canUndo();
  }

  legalActions(): LegalAction[] {
    const json = this.engine.legalActions();
    return JSON.parse(json) as LegalAction[];
  }

  checkVictory(): GameEvent | null {
    const json = this.engine.checkVictory();
    return JSON.parse(json) as GameEvent | null;
  }

  turnSummary(): string {
    return this.engine.turnSummary();
  }

  serializeForSave(): Uint8Array {
    return this.engine.serializeForSave();
  }
}
