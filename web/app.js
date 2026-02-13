// ============================================================
// Axis & Allies Global 1940 — Web Client
// ============================================================

import { POWER_COLORS, UNIT_ICONS, UNIT_COSTS, UNIT_SHORT, TERRITORIES, SEA_ZONES } from './map-data.js';

// ---- Game Configuration ----
let gameMode = 'ai';       // 'ai' or 'hotseat'
let playerSide = 'allies'; // 'allies' or 'axis'
let aiDifficulty = 'normal';
let soundEnabled = true;

// ---- WASM Engine ----
let wasm = null;
let engine = null;
let gameState = null;
let legalActions = [];

// ---- AI State ----
let aiPowers = new Set();   // Powers controlled by AI
let aiPlaying = false;

// ---- Map state ----
let mapOffsetX = -1200;
let mapOffsetY = -100;
let mapScale = 1.0;
let isDragging = false;
let dragStartX = 0, dragStartY = 0;
let dragMoved = false;
let selectedTerritory = null;
let hoveredTerritory = null;

// ---- Combat highlight state ----
let combatHighlights = [];  // territory IDs currently in combat
let highlightFrame = 0;

// ---- Canvas ----
let canvas, ctx;

// ---- Sound Effects ----
const AudioCtx = window.AudioContext || window.webkitAudioContext;
let audioCtx = null;

function getAudioCtx() {
  if (!audioCtx) audioCtx = new AudioCtx();
  return audioCtx;
}

function playTone(freq, duration, type = 'square', volume = 0.08) {
  if (!soundEnabled) return;
  try {
    const ac = getAudioCtx();
    const osc = ac.createOscillator();
    const gain = ac.createGain();
    osc.type = type;
    osc.frequency.value = freq;
    gain.gain.value = volume;
    gain.gain.exponentialRampToValueAtTime(0.001, ac.currentTime + duration);
    osc.connect(gain);
    gain.connect(ac.destination);
    osc.start();
    osc.stop(ac.currentTime + duration);
  } catch(e) { /* audio not available */ }
}

function sfxClick() { playTone(800, 0.05, 'sine', 0.05); }
function sfxPhase() { playTone(440, 0.1, 'sine', 0.06); playTone(660, 0.15, 'sine', 0.06); }
function sfxPurchase() { playTone(523, 0.08, 'sine', 0.05); }
function sfxDice() {
  for (let i = 0; i < 5; i++) {
    setTimeout(() => playTone(200 + Math.random() * 400, 0.04, 'noise', 0.03), i * 30);
  }
}
function sfxCombat() { playTone(150, 0.3, 'sawtooth', 0.06); }
function sfxVictory() {
  [523, 659, 784, 1047].forEach((f, i) => {
    setTimeout(() => playTone(f, 0.3, 'sine', 0.08), i * 150);
  });
}
function sfxTurn() { playTone(330, 0.15, 'triangle', 0.06); }

// ---- Setup Screen ----
window.selectMode = function(mode) {
  gameMode = mode;
  document.querySelectorAll('.mode-btn').forEach(b => b.classList.toggle('selected', b.dataset.mode === mode));
  document.getElementById('ai-settings').classList.toggle('hidden', mode !== 'ai');
};

window.selectSide = function(side) {
  playerSide = side;
  document.querySelectorAll('.side-btn').forEach(b => b.classList.toggle('selected', b.dataset.side === side));
};

window.selectDifficulty = function(diff) {
  aiDifficulty = diff;
  document.querySelectorAll('.diff-btn').forEach(b => b.classList.toggle('selected', b.dataset.diff === diff));
};

window.startNewGame = async function() {
  sfxClick();
  await initWasm();
  if (!wasm) return;

  engine = new wasm.WasmEngine(BigInt(Date.now()));
  setupAiPowers();
  refreshState();

  document.getElementById('setup-screen').classList.add('hidden');
  document.getElementById('game-screen').classList.remove('hidden');

  setupCanvas();
  setupEvents();
  render();
  updateUI();
  log('Game started! Turn 1 begins.');
  autoSave();

  // If first power is AI, start AI turn
  if (isCurrentPowerAi()) {
    setTimeout(() => playAiTurn(), 500);
  }
};

window.loadGameFromFile = function() {
  document.getElementById('file-input').click();
};

window.handleFileLoad = async function(event) {
  const file = event.target.files[0];
  if (!file) return;
  
  const text = await file.text();
  await initWasm();
  if (!wasm) return;
  
  try {
    engine = wasm.WasmEngine.loadSaveFile(text);
    setupAiPowers();
    refreshState();
    
    document.getElementById('setup-screen').classList.add('hidden');
    document.getElementById('game-screen').classList.remove('hidden');
    
    setupCanvas();
    setupEvents();
    render();
    updateUI();
    log('Game loaded from ' + file.name);
    
    if (isCurrentPowerAi()) {
      setTimeout(() => playAiTurn(), 500);
    }
  } catch(e) {
    alert('Failed to load save file: ' + e);
  }
  event.target.value = '';
};

function setupAiPowers() {
  aiPowers.clear();
  if (gameMode === 'ai') {
    const axisPowers = ['Germany', 'Japan', 'Italy'];
    const alliedPowers = ['SovietUnion', 'UnitedStates', 'China', 'UnitedKingdom', 'ANZAC', 'France'];
    
    if (playerSide === 'allies') {
      axisPowers.forEach(p => aiPowers.add(p));
    } else {
      alliedPowers.forEach(p => aiPowers.add(p));
    }
  }
}

function isCurrentPowerAi() {
  return gameState && aiPowers.has(gameState.current_power);
}

// ---- WASM Initialization ----
async function initWasm() {
  if (wasm) return;
  try {
    wasm = await import('./pkg/aa_wasm.js');
    await wasm.default();
    const ver = wasm.WasmEngine.engineVersion();
    document.getElementById('engine-version').textContent = `Engine v${ver}`;
  } catch(e) {
    console.error('WASM load failed:', e);
    alert('Failed to load game engine. Please ensure WASM files are built.');
  }
}

// ---- State Management ----
function refreshState() {
  if (!engine) return;
  try {
    gameState = JSON.parse(engine.getState());
    legalActions = JSON.parse(engine.legalActions());
  } catch(e) {
    console.error('Failed to refresh state:', e);
  }
}

function submitAction(action) {
  if (!engine) return null;
  try {
    const resultJson = engine.submitAction(JSON.stringify(action));
    const result = JSON.parse(resultJson);
    if (result.error) {
      log('⚠ ' + result.error);
      return null;
    }
    refreshState();
    updateUI();
    render();
    
    if (result.events) {
      for (const evt of result.events) {
        logEvent(evt);
        handleEvent(evt);
      }
    }
    return result;
  } catch(e) {
    log('⚠ Action failed: ' + e);
    return null;
  }
}

function handleEvent(evt) {
  const type = Object.keys(evt)[0];
  const data = evt[type];
  
  switch(type) {
    case 'PhaseChanged':
      sfxPhase();
      break;
    case 'TurnChanged':
      sfxTurn();
      autoSave();
      // Check if new power is AI
      if (isCurrentPowerAi()) {
        setTimeout(() => playAiTurn(), 600);
      }
      break;
    case 'BattleStarted':
      sfxCombat();
      break;
    case 'BattleEnded':
      sfxDice();
      break;
    case 'VictoryAchieved':
      sfxVictory();
      showVictoryScreen(data);
      break;
    case 'UnitsPurchased':
      sfxPurchase();
      break;
  }
}

// ---- AI Turn ----
async function playAiTurn() {
  if (!engine || !isCurrentPowerAi() || aiPlaying) return;
  
  aiPlaying = true;
  const banner = document.getElementById('ai-banner');
  const bannerText = document.getElementById('ai-banner-text');
  const powerName = POWER_COLORS[gameState.current_power]?.name || gameState.current_power;
  
  banner.classList.remove('hidden');
  bannerText.textContent = `${powerName} (AI) is thinking...`;
  
  // Disable player controls
  document.getElementById('btn-end-phase').disabled = true;
  
  // Get AI actions
  let actionsJson;
  try {
    actionsJson = engine.aiPlayTurn(aiDifficulty);
  } catch(e) {
    console.error('AI failed:', e);
    banner.classList.add('hidden');
    aiPlaying = false;
    return;
  }
  
  const actions = JSON.parse(actionsJson);
  
  // Play actions with delay for visual feedback
  const startPower = gameState.current_power;
  
  for (let i = 0; i < actions.length; i++) {
    if (gameState.current_power !== startPower) break;
    
    const action = actions[i];
    const actionType = typeof action === 'string' ? action : Object.keys(action)[0];
    
    // Update banner text based on current phase
    const phaseNames = {
      PurchaseAndRepair: 'purchasing units',
      CombatMovement: 'planning attacks',
      ConductCombat: 'resolving combat',
      NonCombatMovement: 'moving reinforcements',
      Mobilize: 'deploying units',
      CollectIncome: 'collecting income',
    };
    bannerText.textContent = `${powerName} is ${phaseNames[gameState.current_phase] || 'thinking'}...`;
    
    const result = submitAction(action);
    
    // Add small delays for key actions so user can follow
    if (actionType === 'ConfirmPurchases' || actionType === 'ConfirmCombatMovement' ||
        actionType === 'ConfirmPhase' || actionType === 'ConfirmNonCombatMovement' ||
        actionType === 'ConfirmMobilization' || actionType === 'ConfirmIncome' ||
        actionType === 'MoveUnit' || actionType === 'PurchaseUnit') {
      await sleep(150);
    } else {
      await sleep(30);
    }
  }
  
  banner.classList.add('hidden');
  document.getElementById('btn-end-phase').disabled = false;
  aiPlaying = false;
  
  // If still AI's turn (incomplete), try again
  if (isCurrentPowerAi()) {
    setTimeout(() => playAiTurn(), 300);
  }
}

function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}

// ---- Save / Load ----
window.saveGame = function() {
  if (!engine) return;
  sfxClick();
  try {
    const timestamp = Math.floor(Date.now() / 1000);
    const name = `Turn ${gameState.turn_number} - ${POWER_COLORS[gameState.current_power]?.name || gameState.current_power}`;
    const json = engine.createSaveFile(name, timestamp);
    
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `aa1940_turn${gameState.turn_number}_${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
    
    log('💾 Game saved.');
  } catch(e) {
    log('⚠ Save failed: ' + e);
  }
};

function autoSave() {
  if (!engine) return;
  try {
    const timestamp = Math.floor(Date.now() / 1000);
    const json = engine.createSaveFile('Autosave', timestamp);
    localStorage.setItem('aa1940_autosave', json);
  } catch(e) {
    console.warn('Autosave failed:', e);
  }
}

// Check for autosave on load
function checkAutoSave() {
  const save = localStorage.getItem('aa1940_autosave');
  if (save) {
    // Add a "Continue" button to setup
    const actions = document.querySelector('.setup-actions');
    const btn = document.createElement('button');
    btn.className = 'btn btn-primary';
    btn.textContent = '▶ Continue Last Game';
    btn.onclick = async () => {
      await initWasm();
      if (!wasm) return;
      try {
        engine = wasm.WasmEngine.loadSaveFile(save);
        setupAiPowers();
        refreshState();
        document.getElementById('setup-screen').classList.add('hidden');
        document.getElementById('game-screen').classList.remove('hidden');
        setupCanvas();
        setupEvents();
        render();
        updateUI();
        log('Resumed from autosave.');
        if (isCurrentPowerAi()) setTimeout(() => playAiTurn(), 500);
      } catch(e) {
        console.warn('Autosave corrupted:', e);
        localStorage.removeItem('aa1940_autosave');
      }
    };
    actions.insertBefore(btn, actions.firstChild);
  }
}

// ---- Menu ----
window.showMenu = function() { document.getElementById('menu-dialog').classList.remove('hidden'); };
window.closeMenu = function() { document.getElementById('menu-dialog').classList.add('hidden'); };

window.quitToMenu = function() {
  document.getElementById('menu-dialog').classList.add('hidden');
  document.getElementById('game-screen').classList.add('hidden');
  document.getElementById('setup-screen').classList.remove('hidden');
  engine = null;
  gameState = null;
};

window.toggleSoundEffects = function() {
  soundEnabled = !soundEnabled;
  document.getElementById('btn-sound').textContent = soundEnabled ? '🔊' : '🔇';
  if (soundEnabled) sfxClick();
};

window.showGameStats = function() {
  if (!gameState) return;
  let html = '<div class="summary-section"><h4>Game Statistics</h4>';
  html += `<div>Turn: ${gameState.turn_number}</div>`;
  html += `<div>Current Power: ${POWER_COLORS[gameState.current_power]?.name}</div>`;
  html += `<div>Actions Taken: ${gameState.action_log?.length || 0}</div>`;
  
  let totalUnits = 0;
  gameState.territories.forEach(t => { totalUnits += (t.units?.length || 0); });
  gameState.sea_zones.forEach(sz => { totalUnits += (sz.units?.length || 0); });
  html += `<div>Total Units on Map: ${totalUnits}</div>`;
  html += '</div>';
  
  // IPC totals by team
  let axisIpc = 0, alliedIpc = 0;
  for (const ps of gameState.powers) {
    if (['Germany', 'Japan', 'Italy'].includes(ps.power)) axisIpc += ps.ipcs;
    else alliedIpc += ps.ipcs;
  }
  html += `<div class="summary-section"><h4>Economic Summary</h4>`;
  html += `<div>Axis Total: <span style="color:var(--red)">${axisIpc} IPC</span></div>`;
  html += `<div>Allies Total: <span style="color:var(--green)">${alliedIpc} IPC</span></div>`;
  html += '</div>';
  
  document.getElementById('turn-summary-title').textContent = '📊 Game Statistics';
  document.getElementById('turn-summary-content').innerHTML = html;
  document.getElementById('turn-summary-dialog').classList.remove('hidden');
};

function showVictoryScreen(data) {
  const winner = data?.winner || 'Unknown';
  let html = `<div style="text-align:center;padding:20px;">`;
  html += `<div style="font-size:64px;margin-bottom:16px;">🏆</div>`;
  html += `<div style="font-size:24px;font-weight:800;color:var(--gold);margin-bottom:8px;">${winner} Victory!</div>`;
  html += `<div style="color:var(--text-muted);">The ${winner} have achieved victory!</div>`;
  html += `</div>`;
  
  document.getElementById('turn-summary-title').textContent = '🏆 Victory!';
  document.getElementById('turn-summary-content').innerHTML = html;
  document.getElementById('turn-summary-dialog').classList.remove('hidden');
}

window.closeTurnSummary = function() {
  document.getElementById('turn-summary-dialog').classList.add('hidden');
};

// ---- Canvas Setup ----
function setupCanvas() {
  canvas = document.getElementById('map-canvas');
  ctx = canvas.getContext('2d');
  resizeCanvas();
  window.addEventListener('resize', () => { resizeCanvas(); render(); });
}

function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect();
  const dpr = window.devicePixelRatio || 1;
  canvas.width = rect.width * dpr;
  canvas.height = rect.height * dpr;
  canvas.style.width = rect.width + 'px';
  canvas.style.height = rect.height + 'px';
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}

// ---- Map Controls ----
window.zoomIn = function() { mapScale = Math.min(3, mapScale * 1.2); render(); };
window.zoomOut = function() { mapScale = Math.max(0.3, mapScale / 1.2); render(); };
window.resetView = function() { mapOffsetX = -1200; mapOffsetY = -100; mapScale = 1.0; render(); };

// ---- Events ----
function setupEvents() {
  canvas.addEventListener('mousedown', (e) => {
    isDragging = true;
    dragMoved = false;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
  });

  window.addEventListener('mousemove', (e) => {
    if (isDragging) {
      const dx = e.clientX - dragStartX;
      const dy = e.clientY - dragStartY;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) dragMoved = true;
      mapOffsetX += dx;
      mapOffsetY += dy;
      dragStartX = e.clientX;
      dragStartY = e.clientY;
      render();
    } else {
      handleHover(e);
    }
  });

  window.addEventListener('mouseup', () => { isDragging = false; });

  canvas.addEventListener('wheel', (e) => {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    const mx = e.clientX - rect.left;
    const my = e.clientY - rect.top;
    
    const oldScale = mapScale;
    mapScale *= e.deltaY < 0 ? 1.1 : 0.9;
    mapScale = Math.max(0.3, Math.min(3, mapScale));
    
    mapOffsetX = mx - (mx - mapOffsetX) * (mapScale / oldScale);
    mapOffsetY = my - (my - mapOffsetY) * (mapScale / oldScale);
    render();
  }, { passive: false });

  canvas.addEventListener('click', (e) => {
    if (dragMoved) return;
    sfxClick();
    const hit = hitTest(e);
    if (hit) {
      selectedTerritory = hit;
      showTerritoryInfo(hit);
      handleTerritoryClick(hit);
    } else {
      selectedTerritory = null;
      document.getElementById('territory-info').classList.add('hidden');
    }
    render();
  });

  // Bottom bar buttons
  document.getElementById('btn-end-phase').addEventListener('click', () => {
    if (aiPlaying) return;
    sfxClick();
    endPhase();
  });
  document.getElementById('btn-undo').addEventListener('click', () => {
    sfxClick();
    submitAction('Undo');
  });

  // Purchase dialog confirm
  document.getElementById('btn-confirm-purchases').addEventListener('click', () => {
    submitAction('ConfirmPurchases');
    closePurchaseDialog();
  });

  // Mobilize dialog confirm
  document.getElementById('btn-confirm-mobilize').addEventListener('click', () => {
    submitAction('ConfirmMobilization');
    closeMobilizeDialog();
  });

  // Touch support
  let touchStartX, touchStartY, touchStartDist;
  canvas.addEventListener('touchstart', (e) => {
    if (e.touches.length === 1) {
      isDragging = true;
      dragMoved = false;
      dragStartX = e.touches[0].clientX;
      dragStartY = e.touches[0].clientY;
    } else if (e.touches.length === 2) {
      touchStartDist = Math.hypot(
        e.touches[0].clientX - e.touches[1].clientX,
        e.touches[0].clientY - e.touches[1].clientY
      );
    }
    e.preventDefault();
  }, { passive: false });

  canvas.addEventListener('touchmove', (e) => {
    if (e.touches.length === 1 && isDragging) {
      const dx = e.touches[0].clientX - dragStartX;
      const dy = e.touches[0].clientY - dragStartY;
      if (Math.abs(dx) > 3 || Math.abs(dy) > 3) dragMoved = true;
      mapOffsetX += dx;
      mapOffsetY += dy;
      dragStartX = e.touches[0].clientX;
      dragStartY = e.touches[0].clientY;
      render();
    } else if (e.touches.length === 2 && touchStartDist) {
      const dist = Math.hypot(
        e.touches[0].clientX - e.touches[1].clientX,
        e.touches[0].clientY - e.touches[1].clientY
      );
      mapScale *= dist / touchStartDist;
      mapScale = Math.max(0.3, Math.min(3, mapScale));
      touchStartDist = dist;
      render();
    }
    e.preventDefault();
  }, { passive: false });

  canvas.addEventListener('touchend', () => { isDragging = false; });
}

function screenToMap(sx, sy) {
  return { x: (sx - mapOffsetX) / mapScale, y: (sy - mapOffsetY) / mapScale };
}

function hitTest(e) {
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
  const { x, y } = screenToMap(sx, sy);
  
  for (let i = 0; i < TERRITORIES.length; i++) {
    const t = TERRITORIES[i];
    const dx = x - t.x, dy = y - t.y;
    if (dx*dx + dy*dy < 22*22) return { type: 'land', id: i };
  }
  for (let i = 0; i < SEA_ZONES.length; i++) {
    const sz = SEA_ZONES[i];
    const dx = x - sz.x, dy = y - sz.y;
    if (dx*dx + dy*dy < 20*20) return { type: 'sea', id: i };
  }
  return null;
}

function handleHover(e) {
  const rect = canvas.getBoundingClientRect();
  if (!rect) return;
  const hit = hitTest(e);
  const tooltip = document.getElementById('map-tooltip');
  
  if (hit && gameState) {
    hoveredTerritory = hit;
    let html = '';
    if (hit.type === 'land') {
      const t = TERRITORIES[hit.id];
      const ts = gameState.territories[hit.id];
      const owner = ts?.owner;
      const color = POWER_COLORS[owner || 'null'];
      html = `<strong>${t.name}</strong>`;
      if (owner) html += `<br>Owner: <span style="color:${color.fill}">${color.name}</span>`;
      if (ts?.units?.length) {
        const counts = {};
        for (const u of ts.units) counts[u.unit_type] = (counts[u.unit_type] || 0) + 1;
        for (const [type, count] of Object.entries(counts)) {
          html += `<br>${UNIT_ICONS[type] || '•'} ${UNIT_SHORT[type] || type} ×${count}`;
        }
      }
    } else {
      const sz = SEA_ZONES[hit.id];
      html = `<strong>Sea Zone ${sz.board}</strong>`;
      const szState = gameState.sea_zones[hit.id];
      if (szState?.units?.length) {
        const counts = {};
        for (const u of szState.units) {
          const key = `${u.owner}:${u.unit_type}`;
          counts[key] = (counts[key] || 0) + 1;
        }
        for (const [key, count] of Object.entries(counts)) {
          const [owner, type] = key.split(':');
          const color = POWER_COLORS[owner || 'null'];
          html += `<br><span style="color:${color.fill}">${UNIT_ICONS[type] || '•'} ${UNIT_SHORT[type]} ×${count}</span>`;
        }
      }
    }
    
    tooltip.innerHTML = html;
    tooltip.classList.remove('hidden');
    tooltip.style.left = Math.min(e.clientX - rect.left + 15, rect.width - 230) + 'px';
    tooltip.style.top = Math.min(e.clientY - rect.top + 15, rect.height - 100) + 'px';
    render();
  } else if (hoveredTerritory) {
    hoveredTerritory = null;
    tooltip.classList.add('hidden');
    render();
  }
}

// ---- Rendering ----
function render() {
  if (!canvas || !ctx) return;
  const w = canvas.width / (window.devicePixelRatio || 1);
  const h = canvas.height / (window.devicePixelRatio || 1);
  
  ctx.save();
  
  // Ocean background
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, '#0a1628');
  grad.addColorStop(0.3, '#0d2137');
  grad.addColorStop(0.7, '#0d2137');
  grad.addColorStop(1, '#0a1628');
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, w, h);
  
  ctx.translate(mapOffsetX, mapOffsetY);
  ctx.scale(mapScale, mapScale);
  
  drawOceanGrid();
  drawSeaZones();
  drawTerritories();
  drawUnits();
  drawSelection();
  
  ctx.restore();
  
  highlightFrame++;
}

function drawOceanGrid() {
  ctx.strokeStyle = 'rgba(30, 60, 100, 0.2)';
  ctx.lineWidth = 0.5;
  for (let x = 0; x <= 3200; x += 100) {
    ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, 1600); ctx.stroke();
  }
  for (let y = 0; y <= 1600; y += 100) {
    ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(3200, y); ctx.stroke();
  }
}

function drawSeaZones() {
  for (let i = 0; i < SEA_ZONES.length; i++) {
    const sz = SEA_ZONES[i];
    const isHovered = hoveredTerritory?.type === 'sea' && hoveredTerritory?.id === i;
    const isSelected = selectedTerritory?.type === 'sea' && selectedTerritory?.id === i;
    
    ctx.beginPath();
    ctx.arc(sz.x, sz.y, 16, 0, Math.PI * 2);
    ctx.fillStyle = isHovered ? 'rgba(40, 80, 140, 0.5)' : 'rgba(20, 50, 100, 0.25)';
    ctx.fill();
    
    if (isSelected) {
      ctx.strokeStyle = '#e94560';
      ctx.lineWidth = 2;
      ctx.stroke();
    }
    
    ctx.fillStyle = 'rgba(80, 140, 200, 0.6)';
    ctx.font = '8px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(sz.board.toString(), sz.x, sz.y);
    
    if (gameState?.sea_zones[i]?.units?.length) {
      drawUnitStack(sz.x, sz.y + 18, gameState.sea_zones[i].units);
    }
  }
}

function drawTerritories() {
  if (!gameState) return;
  
  for (let i = 0; i < TERRITORIES.length; i++) {
    const t = TERRITORIES[i];
    const ts = gameState.territories[i];
    if (!ts) continue;
    
    const owner = ts.owner;
    const colors = POWER_COLORS[owner || 'null'];
    const isHovered = hoveredTerritory?.type === 'land' && hoveredTerritory?.id === i;
    const isSelected = selectedTerritory?.type === 'land' && selectedTerritory?.id === i;
    const inCombat = combatHighlights.includes(i);
    
    const r = 18;
    ctx.beginPath();
    for (let a = 0; a < 6; a++) {
      const angle = (a * 60 - 30) * Math.PI / 180;
      const px = t.x + r * Math.cos(angle);
      const py = t.y + r * Math.sin(angle);
      if (a === 0) ctx.moveTo(px, py); else ctx.lineTo(px, py);
    }
    ctx.closePath();
    
    let fillColor = colors.fill;
    if (isHovered) fillColor = lightenColor(fillColor, 0.15);
    if (inCombat) {
      const pulse = 0.5 + 0.5 * Math.sin(highlightFrame * 0.15);
      fillColor = blendColor(fillColor, '#ff0000', pulse * 0.3);
    }
    ctx.fillStyle = fillColor;
    ctx.fill();
    
    ctx.strokeStyle = isSelected ? '#e94560' : colors.stroke;
    ctx.lineWidth = isSelected ? 2.5 : 0.8;
    ctx.stroke();
    
    if (isSelected) {
      ctx.save();
      ctx.shadowColor = '#e94560';
      ctx.shadowBlur = 8;
      ctx.strokeStyle = '#e94560';
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.restore();
    }
    
    // Territory name
    ctx.fillStyle = colors.text;
    ctx.font = 'bold 6.5px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    let name = t.name;
    if (name.length > 12) name = name.substring(0, 11) + '.';
    ctx.fillText(name, t.x, t.y - 4);
    
    // IPC value
    const ipc = getIpcValue(i);
    if (ipc > 0) {
      ctx.fillStyle = '#ffd700';
      ctx.font = 'bold 6px sans-serif';
      ctx.fillText(ipc.toString(), t.x, t.y + 5);
    }
    
    // Victory city
    if (isVictoryCity(i)) {
      ctx.fillStyle = '#ff0';
      ctx.font = '7px sans-serif';
      ctx.fillText('★', t.x + 13, t.y - 11);
    }
  }
}

function drawUnits() {
  if (!gameState) return;
  for (let i = 0; i < TERRITORIES.length; i++) {
    const t = TERRITORIES[i];
    const ts = gameState.territories[i];
    if (!ts?.units?.length) continue;
    drawUnitStack(t.x, t.y + 13, ts.units);
  }
}

function drawUnitStack(x, y, units) {
  const groups = {};
  for (const u of units) {
    const key = `${u.owner}:${u.unit_type}`;
    groups[key] = (groups[key] || 0) + 1;
  }
  
  const entries = Object.entries(groups);
  const totalWidth = entries.length * 15;
  let ox = x - totalWidth / 2;
  
  for (const [key, count] of entries) {
    const [owner, type] = key.split(':');
    const colors = POWER_COLORS[owner || 'null'];
    
    // Unit pip
    ctx.beginPath();
    ctx.arc(ox + 7, y, 5.5, 0, Math.PI * 2);
    ctx.fillStyle = colors.fill;
    ctx.fill();
    ctx.strokeStyle = 'rgba(0,0,0,0.6)';
    ctx.lineWidth = 0.5;
    ctx.stroke();
    
    // Unit letter
    ctx.fillStyle = colors.text;
    ctx.font = 'bold 5.5px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText((UNIT_SHORT[type] || type)[0], ox + 7, y);
    
    // Count badge
    if (count > 1) {
      ctx.fillStyle = '#e94560';
      ctx.beginPath();
      ctx.arc(ox + 12, y - 4, 4.5, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = '#fff';
      ctx.font = 'bold 4.5px sans-serif';
      ctx.fillText(count.toString(), ox + 12, y - 4);
    }
    
    ox += 15;
  }
}

function drawSelection() {
  // Could draw movement arrows or valid targets here
}

// ---- Color Helpers ----
function lightenColor(hex, amount) {
  const num = parseInt(hex.replace('#', ''), 16);
  const r = Math.min(255, ((num >> 16) & 0xff) + 255 * amount);
  const g = Math.min(255, ((num >> 8) & 0xff) + 255 * amount);
  const b = Math.min(255, (num & 0xff) + 255 * amount);
  return `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
}

function blendColor(hex1, hex2, t) {
  const c1 = parseInt(hex1.replace('#', '').replace(/rgb\(|\)/g, ''), 16) || 0;
  const c2 = parseInt(hex2.replace('#', ''), 16);
  const r1 = (c1 >> 16) & 0xff, g1 = (c1 >> 8) & 0xff, b1 = c1 & 0xff;
  const r2 = (c2 >> 16) & 0xff, g2 = (c2 >> 8) & 0xff, b2 = c2 & 0xff;
  const r = Math.round(r1 + (r2 - r1) * t);
  const g = Math.round(g1 + (g2 - g1) * t);
  const b = Math.round(b1 + (b2 - b1) * t);
  return `rgb(${r},${g},${b})`;
}

// ---- IPC & Victory Data ----
const IPC_MAP = {
  0: 3, 1: 2, 2: 2, 3: 2, 4: 2, 5: 4, 6: 1, 7: 2, 8: 3, 9: 1,
  11: 2, 13: 2, 14: 3, 15: 2, 16: 1, 17: 2, 19: 4, 20: 2,
  26: 6, 27: 2, 29: 0, 31: 2, 34: 2,
  35: 1, 36: 1, 37: 1, 38: 1, 39: 1, 40: 2, 41: 0,
  42: 1, 45: 1, 46: 1, 47: 1, 48: 1, 49: 1, 50: 1, 51: 1, 52: 2,
  57: 1, 58: 2, 59: 1, 60: 2, 61: 1, 62: 1,
  65: 3, 67: 2, 69: 3, 70: 2, 71: 2, 72: 2, 73: 2, 74: 2,
  75: 2, 76: 1, 77: 1, 78: 3, 80: 2, 85: 2, 87: 1, 88: 1,
  95: 3, 96: 1, 97: 1, 98: 1, 99: 1, 100: 1, 101: 1,
  104: 1, 106: 3, 107: 2, 108: 1, 109: 3, 113: 3,
  114: 3, 115: 1, 116: 1, 118: 3, 119: 2, 120: 2,
  122: 4, 123: 3, 124: 2, 125: 3,
  126: 8, 129: 3, 130: 1, 140: 3,
  141: 2, 142: 1, 144: 1, 146: 2,
  151: 20, 152: 12, 153: 10, 154: 2, 155: 2, 159: 3,
};

function getIpcValue(id) { return IPC_MAP[id] || 0; }

const VICTORY_CITIES = new Set([0, 5, 17, 19, 26, 40, 52, 65, 69, 114, 126, 136, 141, 151, 153]);
function isVictoryCity(id) { return VICTORY_CITIES.has(id); }

// ---- UI Updates ----
function updateUI() {
  if (!gameState) return;
  
  document.getElementById('turn-number').textContent = `Turn ${gameState.turn_number}`;
  
  const powerEl = document.getElementById('current-power');
  const pc = POWER_COLORS[gameState.current_power] || POWER_COLORS['null'];
  powerEl.textContent = pc.name + (isCurrentPowerAi() ? ' 🤖' : '');
  powerEl.style.background = pc.fill;
  powerEl.style.color = pc.text;
  
  const phaseNames = {
    PurchaseAndRepair: 'Purchase & Repair',
    CombatMovement: 'Combat Movement',
    ConductCombat: 'Conduct Combat',
    NonCombatMovement: 'Non-Combat Movement',
    Mobilize: 'Mobilize',
    CollectIncome: 'Collect Income',
  };
  document.getElementById('current-phase').textContent = phaseNames[gameState.current_phase] || gameState.current_phase;
  
  // Phase bar
  const phases = ['PurchaseAndRepair', 'CombatMovement', 'ConductCombat', 'NonCombatMovement', 'Mobilize', 'CollectIncome'];
  const currentIdx = phases.indexOf(gameState.current_phase);
  document.querySelectorAll('.phase-step').forEach((el, i) => {
    el.classList.remove('active', 'completed');
    if (i === currentIdx) el.classList.add('active');
    else if (i < currentIdx) el.classList.add('completed');
  });
  
  // Power list
  const powerList = document.getElementById('power-list');
  powerList.innerHTML = '';
  for (const ps of gameState.powers) {
    const colors = POWER_COLORS[ps.power] || POWER_COLORS['null'];
    const isActive = ps.power === gameState.current_power;
    const isAi = aiPowers.has(ps.power);
    const row = document.createElement('div');
    row.className = `power-row${isActive ? ' active-power' : ''}`;
    row.innerHTML = `
      <div class="power-dot" style="background:${colors.fill}"></div>
      <span>${colors.name}</span>
      ${isAi ? '<span class="power-ai-tag">AI</span>' : ''}
      <span class="power-ipcs">${ps.ipcs} IPC</span>
    `;
    powerList.appendChild(row);
  }
  
  // Action panel
  updateActionPanel();
  
  // Bottom bar state
  const isAiTurn = isCurrentPowerAi();
  document.getElementById('btn-end-phase').disabled = isAiTurn || aiPlaying;
  document.getElementById('btn-undo').disabled = isAiTurn || aiPlaying;
  
  const status = document.getElementById('status-text');
  if (isAiTurn) {
    status.textContent = `${pc.name} (AI) is playing...`;
  } else {
    status.textContent = `Your turn: ${pc.name}`;
  }
}

function updateActionPanel() {
  const content = document.getElementById('action-content');
  if (!gameState) { content.innerHTML = '<em>Loading...</em>'; return; }
  
  if (isCurrentPowerAi()) {
    content.innerHTML = '<p style="color:var(--blue);font-size:12px">🤖 AI is playing this power.</p>';
    return;
  }
  
  const phase = gameState.current_phase;
  
  if (phase === 'PurchaseAndRepair') {
    content.innerHTML = '<button class="btn btn-primary" style="width:100%" onclick="showPurchaseDialog()">🛒 Open Purchase Panel</button>';
  } else if (phase === 'CombatMovement' || phase === 'NonCombatMovement') {
    content.innerHTML = '<p style="font-size:11px;color:var(--text-muted)">Select units on the map and click a destination to move them.</p>';
  } else if (phase === 'ConductCombat') {
    content.innerHTML = '';
    showPendingBattles();
  } else if (phase === 'Mobilize') {
    content.innerHTML = '<button class="btn btn-primary" style="width:100%" onclick="showMobilizeDialog()">🏭 Place Units</button>';
  } else if (phase === 'CollectIncome') {
    const ps = gameState.powers.find(p => p.power === gameState.current_power);
    content.innerHTML = `<p style="font-size:12px">Collecting income: <span style="color:var(--gold);font-weight:700">${ps?.ipcs || 0} IPC</span></p>`;
  }
}

// ---- Purchase Dialog ----
window.showPurchaseDialog = function() {
  const dialog = document.getElementById('purchase-dialog');
  dialog.classList.remove('hidden');
  
  const currentPower = gameState.powers.find(p => p.power === gameState.current_power);
  document.getElementById('purchase-ipcs').textContent = currentPower?.ipcs || 0;
  document.getElementById('purchase-spent').textContent = '0';
  
  const grid = document.getElementById('purchase-grid');
  grid.innerHTML = '';
  
  const unitTypes = [
    'Infantry', 'MechInfantry', 'Artillery', 'Tank', 'AAA',
    'Fighter', 'TacticalBomber', 'StrategicBomber',
    'Transport', 'Submarine', 'Destroyer', 'Cruiser', 'Carrier', 'Battleship'
  ];
  
  for (const type of unitTypes) {
    const item = document.createElement('div');
    item.className = 'purchase-item';
    item.dataset.type = type;
    item.innerHTML = `
      <div class="unit-icon">${UNIT_ICONS[type]}</div>
      <div class="unit-name">${UNIT_SHORT[type]}</div>
      <div class="unit-cost">${UNIT_COSTS[type]} IPC</div>
      <div class="unit-count">
        <button onclick="purchaseUnit('${type}', -1)">−</button>
        <span class="qty" id="qty-${type}">0</span>
        <button onclick="purchaseUnit('${type}', 1)">+</button>
      </div>
    `;
    grid.appendChild(item);
  }
  
  updatePurchaseDisplay();
};

window.closePurchaseDialog = function() {
  document.getElementById('purchase-dialog').classList.add('hidden');
};

window.purchaseUnit = function(type, delta) {
  sfxPurchase();
  if (delta > 0) {
    submitAction({ PurchaseUnit: { unit_type: type, count: 1 } });
  } else {
    submitAction({ RemovePurchase: { unit_type: type, count: 1 } });
  }
  updatePurchaseDisplay();
};

function updatePurchaseDisplay() {
  if (!gameState) return;
  const currentPower = gameState.powers.find(p => p.power === gameState.current_power);
  
  // Calculate spent from phase state
  let spent = 0;
  const counts = {};
  if (gameState.phase_state?.Purchase) {
    spent = gameState.phase_state.Purchase.ipcs_spent || 0;
    const purchases = gameState.phase_state.Purchase.purchases || [];
    for (const [type, count] of purchases) {
      counts[type] = count;
    }
  }
  
  document.getElementById('purchase-ipcs').textContent = currentPower?.ipcs || 0;
  document.getElementById('purchase-spent').textContent = spent;
  
  document.querySelectorAll('.purchase-item').forEach(item => {
    const type = item.dataset.type;
    const qty = counts[type] || 0;
    const qtyEl = item.querySelector('.qty');
    if (qtyEl) qtyEl.textContent = qty;
    item.classList.toggle('has-units', qty > 0);
  });
}

// ---- Mobilize Dialog ----
window.showMobilizeDialog = function() {
  document.getElementById('mobilize-dialog').classList.remove('hidden');
  updateMobilizeDisplay();
};

window.closeMobilizeDialog = function() {
  document.getElementById('mobilize-dialog').classList.add('hidden');
};

function updateMobilizeDisplay() {
  const content = document.getElementById('mobilize-content');
  let html = '<p style="font-size:12px;color:var(--text-muted);margin-bottom:8px">Click territories with factories to place units.</p>';
  
  if (gameState.phase_state?.Mobilize) {
    const ms = gameState.phase_state.Mobilize;
    if (ms.units_to_place?.length) {
      html += '<div style="margin-top:8px"><strong style="color:var(--gold)">Units to place:</strong></div>';
      for (const [type, count] of ms.units_to_place) {
        const placed = (ms.placements || []).filter(([pt]) => pt === type).length;
        const remaining = count - placed;
        if (remaining > 0) {
          html += `<div style="font-size:12px;margin:2px 0">${UNIT_ICONS[type]} ${UNIT_SHORT[type]} ×${remaining}</div>`;
        }
      }
    }
  }
  
  content.innerHTML = html;
}

// ---- Combat ----
function showPendingBattles() {
  const content = document.getElementById('action-content');
  const battles = legalActions.filter(a => {
    const action = a.action;
    return action && (action.SelectBattle || (typeof action === 'object' && Object.keys(action)[0] === 'SelectBattle'));
  });
  
  if (battles.length > 0) {
    let html = '<div style="margin-bottom:8px"><strong style="color:var(--gold);font-size:11px">Pending Battles:</strong></div>';
    for (const b of battles) {
      html += `<button class="btn btn-secondary" style="margin:2px 0;font-size:11px;width:100%;text-align:left" 
        onclick='selectBattle(${JSON.stringify(b.action.SelectBattle?.location || b.action)})'>⚔️ ${b.description}</button>`;
    }
    content.innerHTML = html;
  } else {
    content.innerHTML = '<p style="font-size:11px;color:var(--text-muted)">No pending battles. Click End Phase to continue.</p>';
  }
}

window.selectBattle = function(location) {
  sfxCombat();
  submitAction({ SelectBattle: { location } });
};

// ---- Territory Info ----
function showTerritoryInfo(hit) {
  const infoPanel = document.getElementById('territory-info');
  const nameEl = document.getElementById('territory-name');
  const detailsEl = document.getElementById('territory-details');
  
  infoPanel.classList.remove('hidden');
  
  if (hit.type === 'land') {
    const t = TERRITORIES[hit.id];
    const ts = gameState.territories[hit.id];
    nameEl.textContent = t.name;
    
    let html = '';
    const owner = ts?.owner;
    if (owner) {
      const colors = POWER_COLORS[owner];
      html += `<div>Owner: <span style="color:${colors.fill};font-weight:bold">${colors.name}</span></div>`;
    } else {
      html += '<div style="color:var(--text-muted)">Neutral / Uncontrolled</div>';
    }
    
    const ipc = getIpcValue(hit.id);
    if (ipc > 0) html += `<div>IPC Value: <span style="color:#ffd700;font-weight:700">${ipc}</span></div>`;
    if (isVictoryCity(hit.id)) html += '<div style="color:#ffd700">★ Victory City</div>';
    
    if (ts?.facilities?.length) {
      html += '<div style="margin-top:4px"><strong>Facilities:</strong></div>';
      for (const f of ts.facilities) {
        html += `<div style="margin-left:8px">${f.facility_type}${f.damage > 0 ? ` (${f.damage} dmg)` : ''}</div>`;
      }
    }
    
    if (ts?.units?.length) {
      html += '<div style="margin-top:4px"><strong>Units:</strong></div>';
      const counts = {};
      for (const u of ts.units) {
        const key = `${u.owner}:${u.unit_type}`;
        counts[key] = (counts[key] || 0) + 1;
      }
      for (const [key, count] of Object.entries(counts)) {
        const [owner, type] = key.split(':');
        const colors = POWER_COLORS[owner];
        html += `<div style="margin-left:8px">${UNIT_ICONS[type]} <span style="color:${colors?.fill}">${count}× ${UNIT_SHORT[type] || type}</span></div>`;
      }
    }
    
    detailsEl.innerHTML = html;
  } else {
    const sz = SEA_ZONES[hit.id];
    nameEl.textContent = `Sea Zone ${sz.board}`;
    
    let html = '';
    const szState = gameState.sea_zones[hit.id];
    if (szState?.units?.length) {
      html += '<div><strong>Units:</strong></div>';
      const counts = {};
      for (const u of szState.units) {
        const key = `${u.owner}:${u.unit_type}`;
        counts[key] = (counts[key] || 0) + 1;
      }
      for (const [key, count] of Object.entries(counts)) {
        const [owner, type] = key.split(':');
        const colors = POWER_COLORS[owner];
        html += `<div style="margin-left:8px">${UNIT_ICONS[type]} <span style="color:${colors?.fill}">${count}× ${UNIT_SHORT[type] || type}</span></div>`;
      }
    } else {
      html = '<div style="color:var(--text-muted)">Empty</div>';
    }
    
    detailsEl.innerHTML = html;
  }
}

function handleTerritoryClick(hit) {
  if (!gameState || isCurrentPowerAi()) return;
  
  const phase = gameState.current_phase;
  
  if (phase === 'Mobilize' && hit.type === 'land') {
    // Try to place the first available unit type
    if (gameState.phase_state?.Mobilize) {
      const ms = gameState.phase_state.Mobilize;
      for (const [type, count] of (ms.units_to_place || [])) {
        const placed = (ms.placements || []).filter(([pt]) => pt === type).length;
        if (placed < count) {
          submitAction({ PlaceUnit: { unit_type: type, territory_id: hit.id } });
          updateMobilizeDisplay();
          break;
        }
      }
    }
  }
}

// ---- Phase Progression ----
function endPhase() {
  if (!gameState || isCurrentPowerAi()) return;
  
  const confirmActions = {
    PurchaseAndRepair: 'ConfirmPurchases',
    CombatMovement: 'ConfirmCombatMovement',
    NonCombatMovement: 'ConfirmNonCombatMovement',
    Mobilize: 'ConfirmMobilization',
    CollectIncome: 'ConfirmIncome',
    ConductCombat: 'ConfirmPhase',
  };
  
  const action = confirmActions[gameState.current_phase];
  if (action) submitAction(action);
}

// ---- Logging ----
function log(msg) {
  const logDiv = document.getElementById('log-content');
  if (!logDiv) return;
  const entry = document.createElement('div');
  entry.className = 'log-entry';
  entry.textContent = msg;
  logDiv.prepend(entry);
  
  // Limit log entries
  while (logDiv.children.length > 200) {
    logDiv.removeChild(logDiv.lastChild);
  }
}

function logEvent(evt) {
  const type = Object.keys(evt)[0] || 'Unknown';
  const data = evt[type] || evt;
  
  const entry = document.createElement('div');
  entry.className = 'log-entry';
  
  switch(type) {
    case 'PhaseChanged':
      entry.innerHTML = `<span class="log-event">Phase:</span> ${formatPhase(data.to)}`;
      break;
    case 'TurnChanged': {
      const pc = POWER_COLORS[data.power];
      entry.innerHTML = `<span class="log-event">Turn ${data.turn}:</span> <span style="color:${pc?.fill}">${pc?.name || data.power}</span>`;
      break;
    }
    case 'UnitsPurchased':
      entry.innerHTML = `<span class="log-event">Purchased:</span> ${data.count}× ${UNIT_SHORT[data.unit_type] || data.unit_type} (${data.cost} IPC)`;
      break;
    case 'IncomeCollected': {
      const pc = POWER_COLORS[data.power];
      entry.innerHTML = `<span class="log-event">Income:</span> <span style="color:${pc?.fill}">${pc?.name}</span> +${data.amount} IPC`;
      break;
    }
    case 'BattleStarted':
      entry.innerHTML = `<span class="log-event">⚔️ Battle:</span> Combat begins`;
      break;
    case 'BattleEnded':
      entry.innerHTML = `<span class="log-event">⚔️ Battle:</span> ${data.attacker_won ? 'Attacker wins' : 'Defender holds'}`;
      break;
    case 'VictoryAchieved':
      entry.innerHTML = `<span class="log-event">🏆 VICTORY:</span> ${data.winner} wins!`;
      break;
    case 'WarDeclared': {
      const agg = POWER_COLORS[data.aggressor];
      const tgt = POWER_COLORS[data.target];
      entry.innerHTML = `<span class="log-event">⚡ War:</span> <span style="color:${agg?.fill}">${agg?.name}</span> declares war on <span style="color:${tgt?.fill}">${tgt?.name}</span>`;
      break;
    }
    default:
      entry.innerHTML = `<span class="log-event">${type}</span>`;
  }
  
  const logDiv = document.getElementById('log-content');
  if (logDiv) logDiv.prepend(entry);
}

function formatPhase(phase) {
  const names = {
    PurchaseAndRepair: 'Purchase & Repair',
    CombatMovement: 'Combat Movement',
    ConductCombat: 'Conduct Combat',
    NonCombatMovement: 'Non-Combat Movement',
    Mobilize: 'Mobilize',
    CollectIncome: 'Collect Income',
  };
  return names[phase] || phase;
}

// ---- Initialization ----
checkAutoSave();
