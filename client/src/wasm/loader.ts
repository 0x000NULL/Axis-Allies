/**
 * Async WASM initialization.
 * Loads the aa-wasm module and returns the initialized exports.
 */

let wasmModule: typeof import('./pkg/aa_wasm') | null = null;

export async function initWasm(): Promise<typeof import('./pkg/aa_wasm')> {
  if (wasmModule) return wasmModule;

  try {
    const mod = await import('./pkg/aa_wasm');
    // wasm-bindgen with --target web requires calling an init function
    if (typeof mod.default === 'function') {
      await mod.default();
    }
    wasmModule = mod;
    console.log('[WASM] Engine loaded successfully');
    return mod;
  } catch (error) {
    console.error('[WASM] Failed to load engine:', error);
    throw error;
  }
}

export function getWasmModule(): typeof import('./pkg/aa_wasm') | null {
  return wasmModule;
}
