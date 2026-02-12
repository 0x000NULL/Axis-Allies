import { useEffect, useState } from 'react';
import { useUIStore } from './stores/uiStore';
import { useGameStore } from './stores/gameStore';
import { MainMenu } from './components/MainMenu';
import { GameScreen } from './components/GameScreen';
import { GameEngine } from './wasm/engine';

export function App() {
  const screen = useUIStore((s) => s.screen);
  const wasmReady = useUIStore((s) => s.wasmReady);
  const setWasmReady = useUIStore((s) => s.setWasmReady);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    async function loadWasm() {
      try {
        const wasmModule = await import('./wasm/pkg/aa_wasm');
        await wasmModule.default();

        // Create a new game engine with a random seed
        const wasmEngine = new wasmModule.WasmEngine(BigInt(Date.now()));
        const engine = new GameEngine(wasmEngine);
        useGameStore.getState().setEngine(engine);
        setWasmReady(true);
        console.log('[App] WASM engine initialized');
      } catch (err) {
        console.error('[App] Failed to load WASM:', err);
        setLoadError(
          err instanceof Error ? err.message : 'Failed to load game engine'
        );
      }
    }
    loadWasm();
  }, [setWasmReady]);

  if (loadError) {
    return (
      <div style={styles.center}>
        <h1>Failed to Load Game Engine</h1>
        <p style={{ color: '#ff6b6b' }}>{loadError}</p>
        <p>Make sure the WASM module has been built:</p>
        <code style={styles.code}>npm run wasm:build</code>
      </div>
    );
  }

  if (!wasmReady) {
    return (
      <div style={styles.center}>
        <h1>Axis &amp; Allies Global 1940</h1>
        <p>Loading game engine...</p>
      </div>
    );
  }

  switch (screen) {
    case 'mainMenu':
      return <MainMenu />;
    case 'game':
      return <GameScreen />;
    default:
      return <MainMenu />;
  }
}

const styles = {
  center: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    gap: '16px',
  },
  code: {
    background: '#333',
    padding: '8px 16px',
    borderRadius: '4px',
    fontFamily: 'monospace',
  },
};
