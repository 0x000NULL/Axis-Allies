/* tslint:disable */
/* eslint-disable */

/**
 * The WASM-exposed game engine wrapper.
 */
export class WasmEngine {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the next AI action for the current game state.
     * Returns a JSON-encoded Action.
     */
    aiNextAction(difficulty: string): string;
    /**
     * Play a full AI turn. Returns JSON array of actions.
     */
    aiPlayTurn(difficulty: string): string;
    /**
     * Check if the last action can be undone.
     */
    canUndo(): boolean;
    /**
     * Check if a victory condition has been met. Returns JSON (null or GameEvent).
     */
    checkVictory(): string;
    /**
     * Create a save file with metadata. Returns JSON string.
     */
    createSaveFile(name: string, timestamp: number): string;
    /**
     * Get the current phase name.
     */
    currentPhase(): string;
    /**
     * Get the current power name.
     */
    currentPower(): string;
    /**
     * Get the engine version string.
     */
    static engineVersion(): string;
    /**
     * Restore an engine from a JSON-serialized game state.
     */
    static fromState(state_json: string): WasmEngine;
    /**
     * Get the current game state as JSON.
     */
    getState(): string;
    /**
     * Get the list of currently legal actions as JSON.
     */
    legalActions(): string;
    /**
     * Load a game from MessagePack bytes.
     */
    static loadFromSave(data: Uint8Array): WasmEngine;
    /**
     * Load a game from a save file JSON string.
     */
    static loadSaveFile(json: string): WasmEngine;
    /**
     * Create a new game with the given RNG seed.
     */
    constructor(seed: bigint);
    /**
     * Serialize the game state to MessagePack bytes (for save files).
     */
    serializeForSave(): Uint8Array;
    /**
     * Submit a player action (JSON-encoded). Returns JSON result or error.
     */
    submitAction(action_json: string): string;
    /**
     * Get the current turn number.
     */
    turnNumber(): number;
    /**
     * Get a summary string for the current turn state.
     */
    turnSummary(): string;
}

/**
 * Set up panic hook for better error messages in the browser console.
 */
export function init(): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmengine_free: (a: number, b: number) => void;
    readonly init: () => void;
    readonly wasmengine_aiNextAction: (a: number, b: number, c: number) => [number, number];
    readonly wasmengine_aiPlayTurn: (a: number, b: number, c: number) => [number, number];
    readonly wasmengine_canUndo: (a: number) => number;
    readonly wasmengine_checkVictory: (a: number) => [number, number];
    readonly wasmengine_createSaveFile: (a: number, b: number, c: number, d: number) => [number, number, number, number];
    readonly wasmengine_currentPhase: (a: number) => [number, number];
    readonly wasmengine_currentPower: (a: number) => [number, number];
    readonly wasmengine_engineVersion: () => [number, number];
    readonly wasmengine_fromState: (a: number, b: number) => [number, number, number];
    readonly wasmengine_getState: (a: number) => [number, number];
    readonly wasmengine_legalActions: (a: number) => [number, number];
    readonly wasmengine_loadFromSave: (a: number, b: number) => [number, number, number];
    readonly wasmengine_loadSaveFile: (a: number, b: number) => [number, number, number];
    readonly wasmengine_new: (a: bigint) => number;
    readonly wasmengine_serializeForSave: (a: number) => [number, number, number, number];
    readonly wasmengine_submitAction: (a: number, b: number, c: number) => [number, number];
    readonly wasmengine_turnNumber: (a: number) => number;
    readonly wasmengine_turnSummary: (a: number) => [number, number];
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
