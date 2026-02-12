/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const __wbg_wasmengine_free: (a: number, b: number) => void;
export const wasmengine_canUndo: (a: number) => number;
export const wasmengine_checkVictory: (a: number) => [number, number];
export const wasmengine_engineVersion: () => [number, number];
export const wasmengine_fromState: (a: number, b: number) => [number, number, number];
export const wasmengine_getState: (a: number) => [number, number];
export const wasmengine_legalActions: (a: number) => [number, number];
export const wasmengine_loadFromSave: (a: number, b: number) => [number, number, number];
export const wasmengine_new: (a: bigint) => number;
export const wasmengine_serializeForSave: (a: number) => [number, number, number, number];
export const wasmengine_submitAction: (a: number, b: number, c: number) => [number, number];
export const wasmengine_turnSummary: (a: number) => [number, number];
export const init: () => void;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_malloc: (a: number, b: number) => number;
export const __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __externref_table_dealloc: (a: number) => void;
export const __wbindgen_start: () => void;
