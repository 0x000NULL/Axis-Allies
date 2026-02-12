import { Canvas } from '@react-three/fiber';
import { OrbitControls } from '@react-three/drei';
import { useGameStore } from '../stores/gameStore';
import { useUIStore } from '../stores/uiStore';
import type { Action, Phase } from '../types/game';

function BoardPlaceholder() {
  return (
    <>
      {/* Ground plane */}
      <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.1, 0]}>
        <planeGeometry args={[100, 60]} />
        <meshStandardMaterial color="#1a3a5c" />
      </mesh>

      {/* Placeholder territory - a raised land piece */}
      <mesh position={[0, 0.1, 0]}>
        <boxGeometry args={[5, 0.2, 3]} />
        <meshStandardMaterial color="#6b8e4e" />
      </mesh>

      {/* Another placeholder territory */}
      <mesh position={[-8, 0.1, 2]}>
        <boxGeometry args={[4, 0.2, 4]} />
        <meshStandardMaterial color="#8a8a8a" />
      </mesh>

      {/* Placeholder for a unit token */}
      <mesh position={[0, 0.5, 0]}>
        <cylinderGeometry args={[0.3, 0.3, 0.4, 16]} />
        <meshStandardMaterial color="#cc3333" />
      </mesh>
    </>
  );
}

export function GameScreen() {
  const gameState = useGameStore((s) => s.gameState);
  const submitAction = useGameStore((s) => s.submitAction);
  const error = useGameStore((s) => s.error);
  const setScreen = useUIStore((s) => s.setScreen);

  const confirmActionForPhase = (phase: Phase): Action => {
    switch (phase) {
      case 'PurchaseAndRepair': return 'ConfirmPurchases';
      case 'CombatMovement': return 'ConfirmCombatMovement';
      case 'ConductCombat': return 'ConfirmPhase';
      case 'NonCombatMovement': return 'ConfirmNonCombatMovement';
      case 'Mobilize': return 'ConfirmMobilization';
      case 'CollectIncome': return 'ConfirmIncome';
    }
  };

  const advancePhase = () => {
    if (gameState) {
      submitAction(confirmActionForPhase(gameState.current_phase));
    }
  };

  return (
    <div style={styles.container}>
      {/* Top Bar */}
      <div style={styles.topBar}>
        <button style={styles.backButton} onClick={() => setScreen('mainMenu')}>
          Menu
        </button>
        <div style={styles.turnInfo}>
          {gameState && (
            <>
              <span style={styles.turnLabel}>
                Turn {gameState.turn_number}
              </span>
              <span style={styles.powerLabel}>
                {gameState.current_power}
              </span>
              <span style={styles.phaseLabel}>
                {gameState.current_phase}
              </span>
            </>
          )}
        </div>
        <button style={styles.advanceButton} onClick={advancePhase}>
          Confirm Phase
        </button>
      </div>

      {/* Error display */}
      {error && (
        <div style={styles.error}>{error}</div>
      )}

      {/* 3D Canvas */}
      <div style={styles.canvasContainer}>
        <Canvas
          camera={{ position: [0, 30, 30], fov: 50 }}
          style={{ background: '#0a0a1a' }}
        >
          <ambientLight intensity={0.4} />
          <directionalLight position={[10, 20, 10]} intensity={0.8} />
          <BoardPlaceholder />
          <OrbitControls
            maxPolarAngle={Math.PI / 2.5}
            minPolarAngle={Math.PI / 8}
            minDistance={5}
            maxDistance={80}
          />
        </Canvas>
      </div>

      {/* Bottom Bar */}
      <div style={styles.bottomBar}>
        <span>
          {gameState
            ? `IPCs: ${gameState.powers.find(
                (p) => p.power === gameState.current_power
              )?.ipcs ?? 0}`
            : 'Loading...'}
        </span>
      </div>
    </div>
  );
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    height: '100vh',
    width: '100vw',
  },
  topBar: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '8px 16px',
    background: '#1e1e3a',
    borderBottom: '1px solid #333',
    zIndex: 10,
  },
  backButton: {
    padding: '6px 12px',
    background: '#333',
    color: '#ccc',
    border: '1px solid #555',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '14px',
  },
  turnInfo: {
    display: 'flex',
    gap: '16px',
    alignItems: 'center',
  },
  turnLabel: {
    color: '#aaa',
    fontSize: '14px',
  },
  powerLabel: {
    color: '#d4a574',
    fontSize: '16px',
    fontWeight: 'bold' as const,
  },
  phaseLabel: {
    color: '#7aa2f7',
    fontSize: '14px',
  },
  advanceButton: {
    padding: '6px 16px',
    background: '#2a5a2a',
    color: '#e0e0e0',
    border: '1px solid #3a7a3a',
    borderRadius: '4px',
    cursor: 'pointer',
    fontSize: '14px',
  },
  error: {
    padding: '8px 16px',
    background: '#4a1a1a',
    color: '#ff6b6b',
    textAlign: 'center' as const,
    fontSize: '14px',
  },
  canvasContainer: {
    flex: 1,
    position: 'relative' as const,
  },
  bottomBar: {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '8px 16px',
    background: '#1e1e3a',
    borderTop: '1px solid #333',
    color: '#aaa',
    fontSize: '14px',
    zIndex: 10,
  },
};
