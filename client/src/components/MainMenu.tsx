import { useUIStore } from '../stores/uiStore';
import { useGameStore } from '../stores/gameStore';

export function MainMenu() {
  const setScreen = useUIStore((s) => s.setScreen);
  const setPlayMode = useUIStore((s) => s.setPlayMode);
  const gameState = useGameStore((s) => s.gameState);

  const startGame = (mode: 'hotseat' | 'singleplayer' | 'online') => {
    setPlayMode(mode);
    setScreen('game');
  };

  return (
    <div style={styles.container}>
      <div style={styles.titleArea}>
        <h1 style={styles.title}>Axis &amp; Allies</h1>
        <h2 style={styles.subtitle}>Global 1940 - 2nd Edition</h2>
      </div>

      <div style={styles.menu}>
        <button style={styles.button} onClick={() => startGame('hotseat')}>
          Local Hotseat
        </button>
        <button style={styles.button} onClick={() => startGame('singleplayer')}>
          vs AI
        </button>
        <button style={{ ...styles.button, opacity: 0.5, cursor: 'not-allowed' }} disabled>
          Online Multiplayer (Coming Soon)
        </button>
      </div>

      {gameState && (
        <div style={styles.status}>
          Engine loaded | Turn {gameState.turn_number} |{' '}
          {gameState.current_power} | {gameState.current_phase}
        </div>
      )}
    </div>
  );
}

const styles = {
  container: {
    display: 'flex',
    flexDirection: 'column' as const,
    alignItems: 'center',
    justifyContent: 'center',
    height: '100vh',
    gap: '32px',
  },
  titleArea: {
    textAlign: 'center' as const,
  },
  title: {
    fontSize: '48px',
    fontWeight: 'bold' as const,
    color: '#d4a574',
    margin: 0,
  },
  subtitle: {
    fontSize: '20px',
    color: '#888',
    fontWeight: 'normal' as const,
    margin: 0,
  },
  menu: {
    display: 'flex',
    flexDirection: 'column' as const,
    gap: '12px',
    minWidth: '300px',
  },
  button: {
    padding: '14px 32px',
    fontSize: '18px',
    background: '#2a2a4a',
    color: '#e0e0e0',
    border: '1px solid #444',
    borderRadius: '6px',
    cursor: 'pointer',
    transition: 'background 0.2s',
  },
  status: {
    fontSize: '12px',
    color: '#666',
    fontFamily: 'monospace',
  },
};
