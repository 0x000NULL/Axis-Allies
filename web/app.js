// ============================================================
// Axis & Allies Global 1940 — Web Client
// ============================================================

import { POWER_COLORS, UNIT_ICONS, UNIT_COSTS, UNIT_SHORT, TERRITORIES, SEA_ZONES } from './map-data.js';

// ---- WASM Engine ----
let engine = null;
let gameState = null;
let legalActions = [];

// ---- Map state ----
let mapOffsetX = -1200;
let mapOffsetY = -100;
let mapScale = 1.0;
let isDragging = false;
let dragStartX = 0, dragStartY = 0;
let selectedTerritory = null; // { type: 'land'|'sea', id: number }
let hoveredTerritory = null;

// ---- Canvas ----
const canvas = document.getElementById('map-canvas');
const ctx = canvas.getContext('2d');

// ---- Initialize ----
async function init() {
  try {
    const wasm = await import('./pkg/aa_wasm.js');
    await wasm.default();
    engine = new wasm.WasmEngine(BigInt(Date.now()));
    refreshState();
    log('Game initialized. Germany begins Turn 1.');
  } catch(e) {
    console.error('WASM load failed:', e);
    log('⚠ WASM engine failed to load. Using demo mode.');
    // Create a demo state for UI development
    createDemoState();
  }
  
  setupCanvas();
  setupEvents();
  render();
  updateUI();
}

function createDemoState() {
  // Minimal demo state when WASM isn't available
  gameState = {
    turn_number: 1,
    current_power: 'Germany',
    current_phase: 'PurchaseAndRepair',
    territories: TERRITORIES.map((t, i) => ({
      owner: getDefaultOwner(i),
      units: getDefaultUnits(i),
      facilities: [],
      just_captured: false,
    })),
    sea_zones: SEA_ZONES.map(() => ({ units: [] })),
    powers: [
      { power: 'Germany', ipcs: 30 },
      { power: 'SovietUnion', ipcs: 37 },
      { power: 'Japan', ipcs: 26 },
      { power: 'UnitedStates', ipcs: 52 },
      { power: 'China', ipcs: 12 },
      { power: 'UnitedKingdom', ipcs: 28 },
      { power: 'Italy', ipcs: 10 },
      { power: 'ANZAC', ipcs: 10 },
      { power: 'France', ipcs: 0 },
    ],
    pending_purchases: [],
  };
  legalActions = [];
}

function getDefaultOwner(id) {
  // Simplified default ownership
  const owners = {
    0: 'Germany', 1: 'Germany', 2: 'Germany', 3: 'Germany', 7: 'Germany',
    8: 'Germany', 11: 'Germany', 12: 'Germany', 13: 'Germany', 14: 'Romania',
    5: 'Germany', 4: 'Germany', 6: 'Germany',
    19: 'Italy', 20: 'Italy', 21: 'Italy', 22: 'Italy', 38: 'Italy', 39: 'Italy',
    42: 'Italy', 43: 'Italy',
    26: 'UnitedKingdom', 27: 'UnitedKingdom', 40: 'UnitedKingdom', 41: 'UnitedKingdom',
    44: 'UnitedKingdom', 45: 'UnitedKingdom', 46: 'UnitedKingdom', 49: 'UnitedKingdom',
    50: 'UnitedKingdom', 51: 'UnitedKingdom', 52: 'UnitedKingdom', 54: 'UnitedKingdom',
    57: 'UnitedKingdom', 114: 'UnitedKingdom', 115: 'UnitedKingdom',
    116: 'UnitedKingdom', 118: 'UnitedKingdom', 121: 'UnitedKingdom',
    65: 'SovietUnion', 66: 'SovietUnion', 67: 'SovietUnion', 68: 'SovietUnion',
    69: 'SovietUnion', 70: 'SovietUnion', 71: 'SovietUnion', 72: 'SovietUnion',
    73: 'SovietUnion', 74: 'SovietUnion', 75: 'SovietUnion', 76: 'SovietUnion',
    77: 'SovietUnion', 78: 'SovietUnion', 79: 'SovietUnion', 80: 'SovietUnion',
    81: 'SovietUnion', 82: 'SovietUnion', 83: 'SovietUnion', 84: 'SovietUnion',
    85: 'SovietUnion', 86: 'SovietUnion', 87: 'SovietUnion', 88: 'SovietUnion',
    95: 'China', 96: 'China', 97: 'China', 98: 'China', 99: 'China',
    100: 'China', 101: 'China', 104: 'China',
    106: 'Japan', 107: 'Japan', 108: 'Japan', 109: 'Japan', 111: 'Japan',
    112: 'Japan', 113: 'Japan', 126: 'Japan', 127: 'Japan', 128: 'Japan',
    129: 'Japan', 130: 'Japan', 131: 'Japan', 132: 'Japan', 133: 'Japan',
    120: 'Japan', 119: 'Japan', 117: 'Japan', 139: 'Japan',
    141: 'ANZAC', 142: 'ANZAC', 143: 'ANZAC', 144: 'ANZAC', 145: 'ANZAC',
    146: 'ANZAC', 147: 'ANZAC',
    151: 'UnitedStates', 152: 'UnitedStates', 153: 'UnitedStates',
    154: 'UnitedStates', 136: 'UnitedStates', 140: 'UnitedStates',
    35: 'France', 36: 'France', 37: 'France', 47: 'France', 48: 'France',
    59: 'France',
  };
  return owners[id] || null;
}

function getDefaultUnits(id) {
  // Just a few sample units for key territories in demo mode
  const units = {
    0: [{ unit_type: 'Infantry', owner: 'Germany', id: 1 }, { unit_type: 'Infantry', owner: 'Germany', id: 2 }, { unit_type: 'Tank', owner: 'Germany', id: 3 }],
    26: [{ unit_type: 'Infantry', owner: 'UnitedKingdom', id: 100 }, { unit_type: 'Fighter', owner: 'UnitedKingdom', id: 101 }],
    69: [{ unit_type: 'Infantry', owner: 'SovietUnion', id: 200 }, { unit_type: 'Infantry', owner: 'SovietUnion', id: 201 }, { unit_type: 'Tank', owner: 'SovietUnion', id: 202 }],
    126: [{ unit_type: 'Infantry', owner: 'Japan', id: 300 }, { unit_type: 'Fighter', owner: 'Japan', id: 301 }],
    151: [{ unit_type: 'Infantry', owner: 'UnitedStates', id: 400 }, { unit_type: 'Infantry', owner: 'UnitedStates', id: 401 }, { unit_type: 'Battleship', owner: 'UnitedStates', id: 402 }],
    114: [{ unit_type: 'Infantry', owner: 'UnitedKingdom', id: 500 }, { unit_type: 'Artillery', owner: 'UnitedKingdom', id: 501 }],
  };
  return units[id] || [];
}

function refreshState() {
  if (!engine) return;
  try {
    const stateJson = engine.getState();
    gameState = JSON.parse(stateJson);
    const legalJson = engine.legalActions();
    legalActions = JSON.parse(legalJson);
  } catch(e) {
    console.error('Failed to refresh state:', e);
  }
}

function submitAction(action) {
  if (!engine) {
    log('Engine not loaded — demo mode');
    return null;
  }
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
    // Log events
    if (result.events) {
      for (const evt of result.events) {
        logEvent(evt);
      }
    }
    return result;
  } catch(e) {
    log('⚠ Action failed: ' + e);
    return null;
  }
}

// ---- Canvas Setup ----
function setupCanvas() {
  resizeCanvas();
  window.addEventListener('resize', () => { resizeCanvas(); render(); });
}

function resizeCanvas() {
  const rect = canvas.parentElement.getBoundingClientRect();
  canvas.width = rect.width * window.devicePixelRatio;
  canvas.height = rect.height * window.devicePixelRatio;
  canvas.style.width = rect.width + 'px';
  canvas.style.height = rect.height + 'px';
  ctx.setTransform(window.devicePixelRatio, 0, 0, window.devicePixelRatio, 0, 0);
}

// ---- Events ----
function setupEvents() {
  // Map dragging
  canvas.addEventListener('mousedown', (e) => {
    isDragging = true;
    dragStartX = e.clientX;
    dragStartY = e.clientY;
  });

  window.addEventListener('mousemove', (e) => {
    if (isDragging) {
      mapOffsetX += e.clientX - dragStartX;
      mapOffsetY += e.clientY - dragStartY;
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
    
    // Zoom towards mouse
    mapOffsetX = mx - (mx - mapOffsetX) * (mapScale / oldScale);
    mapOffsetY = my - (my - mapOffsetY) * (mapScale / oldScale);
    
    render();
  }, { passive: false });

  canvas.addEventListener('click', (e) => {
    if (isDragging) return;
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

  // Buttons
  document.getElementById('btn-end-phase').addEventListener('click', endPhase);
  document.getElementById('btn-undo').addEventListener('click', () => submitAction({ Undo: null }));
  document.getElementById('btn-confirm-purchases').addEventListener('click', () => {
    submitAction('ConfirmPurchases');
    document.getElementById('purchase-dialog').classList.add('hidden');
  });
  document.getElementById('btn-confirm-mobilize').addEventListener('click', () => {
    submitAction('ConfirmMobilization');
    document.getElementById('mobilize-dialog').classList.add('hidden');
  });
}

function screenToMap(sx, sy) {
  return {
    x: (sx - mapOffsetX) / mapScale,
    y: (sy - mapOffsetY) / mapScale
  };
}

function mapToScreen(mx, my) {
  return {
    x: mx * mapScale + mapOffsetX,
    y: my * mapScale + mapOffsetY
  };
}

function hitTest(e) {
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
  const { x, y } = screenToMap(sx, sy);
  
  const LAND_RADIUS = 22;
  const SEA_RADIUS = 20;
  
  // Check territories first
  for (let i = 0; i < TERRITORIES.length; i++) {
    const t = TERRITORIES[i];
    const dx = x - t.x, dy = y - t.y;
    if (dx*dx + dy*dy < LAND_RADIUS*LAND_RADIUS) {
      return { type: 'land', id: i };
    }
  }
  
  // Check sea zones
  for (let i = 0; i < SEA_ZONES.length; i++) {
    const sz = SEA_ZONES[i];
    const dx = x - sz.x, dy = y - sz.y;
    if (dx*dx + dy*dy < SEA_RADIUS*SEA_RADIUS) {
      return { type: 'sea', id: i };
    }
  }
  
  return null;
}

function handleHover(e) {
  const rect = canvas.getBoundingClientRect();
  const sx = e.clientX - rect.left;
  const sy = e.clientY - rect.top;
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
        html += `<br>Units: ${ts.units.length}`;
        const counts = {};
        for (const u of ts.units) {
          counts[u.unit_type] = (counts[u.unit_type] || 0) + 1;
        }
        for (const [type, count] of Object.entries(counts)) {
          html += `<br>&nbsp; ${UNIT_ICONS[type] || '•'} ${UNIT_SHORT[type] || type} ×${count}`;
        }
      }
    } else {
      const sz = SEA_ZONES[hit.id];
      html = `<strong>Sea Zone ${sz.board}</strong>`;
      const szState = gameState.sea_zones[hit.id];
      if (szState?.units?.length) {
        html += `<br>Units: ${szState.units.length}`;
        const counts = {};
        for (const u of szState.units) {
          const key = `${u.owner}:${u.unit_type}`;
          counts[key] = (counts[key] || 0) + 1;
        }
        for (const [key, count] of Object.entries(counts)) {
          const [owner, type] = key.split(':');
          const color = POWER_COLORS[owner || 'null'];
          html += `<br>&nbsp; <span style="color:${color.fill}">${UNIT_ICONS[type] || '•'} ${UNIT_SHORT[type] || type} ×${count}</span>`;
        }
      }
    }
    
    tooltip.innerHTML = html;
    tooltip.classList.remove('hidden');
    tooltip.style.left = (e.clientX - rect.left + 15) + 'px';
    tooltip.style.top = (e.clientY - rect.top + 15) + 'px';
    
    render();
  } else if (hoveredTerritory) {
    hoveredTerritory = null;
    tooltip.classList.add('hidden');
    render();
  }
}

// ---- Rendering ----
function render() {
  const w = canvas.width / window.devicePixelRatio;
  const h = canvas.height / window.devicePixelRatio;
  
  ctx.save();
  
  // Background — ocean
  const grad = ctx.createLinearGradient(0, 0, 0, h);
  grad.addColorStop(0, '#0a1628');
  grad.addColorStop(0.3, '#0d2137');
  grad.addColorStop(0.7, '#0d2137');
  grad.addColorStop(1, '#0a1628');
  ctx.fillStyle = grad;
  ctx.fillRect(0, 0, w, h);
  
  // Grid lines for ocean feel
  ctx.save();
  ctx.translate(mapOffsetX, mapOffsetY);
  ctx.scale(mapScale, mapScale);
  
  drawOceanGrid();
  drawSeaZones();
  drawTerritories();
  drawUnits();
  drawSelection();
  
  ctx.restore();
  ctx.restore();
}

function drawOceanGrid() {
  ctx.strokeStyle = 'rgba(30, 60, 100, 0.3)';
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
    
    // Sea zone circle
    ctx.beginPath();
    ctx.arc(sz.x, sz.y, 16, 0, Math.PI * 2);
    ctx.fillStyle = isHovered ? 'rgba(40, 80, 140, 0.5)' : 'rgba(20, 50, 100, 0.3)';
    ctx.fill();
    
    if (isSelected) {
      ctx.strokeStyle = '#e94560';
      ctx.lineWidth = 2;
      ctx.stroke();
    }
    
    // Sea zone number
    ctx.fillStyle = 'rgba(100, 160, 220, 0.7)';
    ctx.font = '9px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText(sz.board.toString(), sz.x, sz.y);
    
    // Draw units in sea zones
    if (gameState && gameState.sea_zones[i]?.units?.length) {
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
    
    // Territory shape (hexagonal-ish)
    const r = 18;
    ctx.beginPath();
    for (let a = 0; a < 6; a++) {
      const angle = (a * 60 - 30) * Math.PI / 180;
      const px = t.x + r * Math.cos(angle);
      const py = t.y + r * Math.sin(angle);
      if (a === 0) ctx.moveTo(px, py);
      else ctx.lineTo(px, py);
    }
    ctx.closePath();
    
    // Fill with power color
    let fillColor = colors.fill;
    if (isHovered) {
      fillColor = lightenColor(fillColor, 0.2);
    }
    ctx.fillStyle = fillColor;
    ctx.fill();
    
    // Border
    ctx.strokeStyle = isSelected ? '#e94560' : colors.stroke;
    ctx.lineWidth = isSelected ? 2.5 : 1;
    ctx.stroke();
    
    // Glow for selected
    if (isSelected) {
      ctx.save();
      ctx.shadowColor = '#e94560';
      ctx.shadowBlur = 10;
      ctx.strokeStyle = '#e94560';
      ctx.lineWidth = 2;
      ctx.stroke();
      ctx.restore();
    }
    
    // Territory name
    ctx.fillStyle = colors.text;
    ctx.font = 'bold 7px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    
    // Truncate long names
    let name = t.name;
    if (name.length > 12) name = name.substring(0, 11) + '.';
    ctx.fillText(name, t.x, t.y - 3);
    
    // IPC value
    const ipc = getIpcValue(i);
    if (ipc > 0) {
      ctx.fillStyle = '#ffd700';
      ctx.font = 'bold 7px sans-serif';
      ctx.fillText(ipc.toString(), t.x, t.y + 6);
    }
    
    // Victory city marker
    if (isVictoryCity(i)) {
      ctx.fillStyle = '#ff0';
      ctx.font = '6px sans-serif';
      ctx.fillText('★', t.x + 12, t.y - 10);
    }
  }
}

function drawUnits() {
  if (!gameState) return;
  
  for (let i = 0; i < TERRITORIES.length; i++) {
    const t = TERRITORIES[i];
    const ts = gameState.territories[i];
    if (!ts?.units?.length) continue;
    drawUnitStack(t.x, t.y + 14, ts.units);
  }
}

function drawUnitStack(x, y, units) {
  // Group by owner+type
  const groups = {};
  for (const u of units) {
    const key = `${u.owner}:${u.unit_type}`;
    groups[key] = (groups[key] || 0) + 1;
  }
  
  const entries = Object.entries(groups);
  const totalWidth = entries.length * 16;
  let ox = x - totalWidth / 2;
  
  for (const [key, count] of entries) {
    const [owner, type] = key.split(':');
    const colors = POWER_COLORS[owner || 'null'];
    
    // Unit pip
    ctx.beginPath();
    ctx.arc(ox + 8, y, 6, 0, Math.PI * 2);
    ctx.fillStyle = colors.fill;
    ctx.fill();
    ctx.strokeStyle = 'rgba(0,0,0,0.5)';
    ctx.lineWidth = 0.5;
    ctx.stroke();
    
    // Unit type letter
    ctx.fillStyle = colors.text;
    ctx.font = 'bold 6px sans-serif';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    const letter = (UNIT_SHORT[type] || type)[0];
    ctx.fillText(letter, ox + 8, y);
    
    // Count badge
    if (count > 1) {
      ctx.fillStyle = '#e94560';
      ctx.beginPath();
      ctx.arc(ox + 13, y - 5, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.fillStyle = '#fff';
      ctx.font = 'bold 5px sans-serif';
      ctx.fillText(count.toString(), ox + 13, y - 5);
    }
    
    ox += 16;
  }
}

function drawSelection() {
  // Draw highlight for valid move targets, etc.
  // TODO: Implement for movement phases
}

// ---- Helper functions ----
function lightenColor(hex, amount) {
  const num = parseInt(hex.replace('#', ''), 16);
  const r = Math.min(255, ((num >> 16) & 0xff) + 255 * amount);
  const g = Math.min(255, ((num >> 8) & 0xff) + 255 * amount);
  const b = Math.min(255, (num & 0xff) + 255 * amount);
  return `rgb(${Math.round(r)},${Math.round(g)},${Math.round(b)})`;
}

// IPC values from engine data (approximate, would be read from state ideally)
const IPC_VALUES = {};
// Will be populated from state; for now use basic values
function getIpcValue(id) {
  // Key territories with known IPC values
  const known = {
    0: 3, 1: 2, 2: 2, 3: 2, 4: 2, 5: 4, 6: 1, 7: 2, 8: 3, 9: 1,
    11: 2, 13: 2, 14: 3, 15: 2, 16: 1, 17: 2, 19: 4, 20: 2, 
    26: 6, 27: 2, 29: 0, 31: 2, 34: 2,
    35: 1, 36: 1, 37: 1, 38: 1, 39: 1, 40: 2, 41: 0,
    42: 1, 45: 1, 46: 1, 47: 1, 48: 1, 49: 1, 50: 1, 51: 1, 52: 2,
    57: 1, 58: 2, 59: 1, 60: 2, 61: 1, 62: 1, 63: 0, 64: 0,
    65: 3, 67: 2, 69: 3, 70: 2, 71: 2, 72: 2, 73: 2, 74: 2,
    75: 2, 76: 1, 77: 1, 78: 3, 80: 2, 85: 2, 87: 1, 88: 1,
    95: 3, 96: 1, 97: 1, 98: 1, 99: 1, 100: 1, 101: 1,
    104: 1, 106: 3, 107: 2, 108: 1, 109: 3, 113: 3,
    114: 3, 115: 1, 116: 1, 118: 3, 119: 2, 120: 2,
    122: 4, 123: 3, 124: 2, 125: 3,
    126: 8, 129: 3, 130: 1, 140: 3,
    141: 2, 142: 1, 143: 0, 144: 1, 145: 0, 146: 2,
    151: 20, 152: 12, 153: 10, 154: 2,
    155: 2, 159: 3,
  };
  return known[id] || 0;
}

const VICTORY_CITIES = new Set([
  0, 5, 17, 19, 26, 40, 52, 65, 69, 114, 126, 136, 141, 151, 153
]);

function isVictoryCity(id) {
  return VICTORY_CITIES.has(id);
}

// ---- UI Updates ----
function updateUI() {
  if (!gameState) return;
  
  // Turn info
  document.getElementById('turn-number').textContent = `Turn ${gameState.turn_number}`;
  
  const powerEl = document.getElementById('current-power');
  const powerColors = POWER_COLORS[gameState.current_power] || POWER_COLORS['null'];
  powerEl.textContent = powerColors.name;
  powerEl.style.background = powerColors.fill;
  powerEl.style.color = powerColors.text;
  
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
  document.querySelectorAll('.phase-step').forEach(el => {
    el.classList.remove('active', 'completed');
    if (el.dataset.phase === gameState.current_phase) {
      el.classList.add('active');
    }
  });
  
  // Power list
  const powerList = document.getElementById('power-list');
  powerList.innerHTML = '';
  for (const ps of gameState.powers) {
    const colors = POWER_COLORS[ps.power] || POWER_COLORS['null'];
    const isActive = ps.power === gameState.current_power;
    const row = document.createElement('div');
    row.className = `power-row${isActive ? ' active-power' : ''}`;
    row.innerHTML = `
      <div class="power-dot" style="background:${colors.fill}"></div>
      <span>${colors.name}</span>
      <span class="power-ipcs">${ps.ipcs} IPC</span>
    `;
    powerList.appendChild(row);
  }
  
  // Show purchase dialog in purchase phase
  updateActionPanel();
}

function updateActionPanel() {
  const content = document.getElementById('action-content');
  if (!gameState) { content.innerHTML = '<em>Loading...</em>'; return; }
  
  const phase = gameState.current_phase;
  
  if (phase === 'PurchaseAndRepair') {
    content.innerHTML = '<button class="btn btn-primary" onclick="window.showPurchaseDialog()">Open Purchase Panel</button>';
    window.showPurchaseDialog = showPurchaseDialog;
  } else if (phase === 'CombatMovement' || phase === 'NonCombatMovement') {
    content.innerHTML = '<p style="font-size:12px;color:var(--text-muted)">Click a territory to select units, then click a destination to move.</p>';
  } else if (phase === 'ConductCombat') {
    content.innerHTML = '<p style="font-size:12px;color:var(--text-muted)">Select a battle to resolve.</p>';
    showPendingBattles();
  } else if (phase === 'Mobilize') {
    content.innerHTML = '<button class="btn btn-primary" onclick="window.showMobilizeDialog()">Place Units</button>';
    window.showMobilizeDialog = showMobilizeDialog;
  } else if (phase === 'CollectIncome') {
    content.innerHTML = '<p style="font-size:12px;color:var(--text-muted)">Collecting income...</p>';
  }
}

// ---- Purchase Dialog ----
function showPurchaseDialog() {
  const dialog = document.getElementById('purchase-dialog');
  dialog.classList.remove('hidden');
  
  const currentPower = gameState.powers.find(p => p.power === gameState.current_power);
  document.getElementById('purchase-ipcs').textContent = currentPower?.ipcs || 0;
  
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
        <button onclick="window.purchaseUnit('${type}', -1)">−</button>
        <span class="qty" id="qty-${type}">0</span>
        <button onclick="window.purchaseUnit('${type}', 1)">+</button>
      </div>
    `;
    grid.appendChild(item);
  }
  
  window.purchaseUnit = (type, delta) => {
    if (delta > 0) {
      submitAction({ PurchaseUnit: { unit_type: type, count: 1 } });
    } else {
      submitAction({ RemovePurchase: { unit_type: type, count: 1 } });
    }
    updatePurchaseDisplay();
  };
}

function updatePurchaseDisplay() {
  if (!gameState) return;
  const currentPower = gameState.powers.find(p => p.power === gameState.current_power);
  document.getElementById('purchase-ipcs').textContent = currentPower?.ipcs || 0;
  
  // Update quantities from pending_purchases
  const counts = {};
  for (const [type, count] of (gameState.pending_purchases || [])) {
    counts[type] = count;
  }
  
  document.querySelectorAll('.purchase-item').forEach(item => {
    const type = item.dataset.type;
    const qtyEl = item.querySelector('.qty');
    if (qtyEl) qtyEl.textContent = counts[type] || 0;
  });
}

// ---- Mobilize Dialog ----
function showMobilizeDialog() {
  const dialog = document.getElementById('mobilize-dialog');
  dialog.classList.remove('hidden');
  
  const content = document.getElementById('mobilize-content');
  content.innerHTML = '<p style="font-size:12px;color:var(--text-muted)">Click territories with industrial complexes to place purchased units.</p>';
  
  // Show pending purchases
  if (gameState.pending_purchases?.length) {
    let html = '<h4 style="margin:8px 0 4px;color:var(--gold);font-size:13px">Units to place:</h4>';
    for (const [type, count] of gameState.pending_purchases) {
      html += `<div style="font-size:12px">${UNIT_ICONS[type]} ${UNIT_SHORT[type]} ×${count}</div>`;
    }
    content.innerHTML += html;
  }
}

// ---- Combat ----
function showPendingBattles() {
  // Check legal actions for SelectBattle
  const battles = legalActions.filter(a => a.action?.SelectBattle);
  if (battles.length > 0) {
    const content = document.getElementById('action-content');
    let html = '<h4 style="color:var(--gold);font-size:12px;margin-bottom:6px">Pending Battles:</h4>';
    for (const b of battles) {
      const loc = b.action.SelectBattle.location;
      html += `<button class="btn btn-secondary" style="margin:2px;font-size:11px" 
        onclick='window.selectBattle(${JSON.stringify(loc)})'>${b.description}</button>`;
    }
    content.innerHTML += html;
  }
  
  window.selectBattle = (location) => {
    submitAction({ SelectBattle: { location } });
  };
}

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
      html += '<div>Uncontrolled</div>';
    }
    
    const ipc = getIpcValue(hit.id);
    if (ipc > 0) html += `<div>IPC Value: <span style="color:#ffd700">${ipc}</span></div>`;
    if (isVictoryCity(hit.id)) html += '<div>★ Victory City</div>';
    
    // Facilities
    if (ts?.facilities?.length) {
      html += '<div style="margin-top:4px"><strong>Facilities:</strong></div>';
      for (const f of ts.facilities) {
        html += `<div>&nbsp; ${f.facility_type}${f.damage > 0 ? ` (${f.damage} dmg)` : ''}</div>`;
      }
    }
    
    // Units
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
        html += `<div>&nbsp; ${UNIT_ICONS[type]} <span style="color:${colors.fill}">${count}× ${type}</span></div>`;
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
        html += `<div>&nbsp; ${UNIT_ICONS[type]} <span style="color:${colors.fill}">${count}× ${type}</span></div>`;
      }
    } else {
      html = '<div style="color:var(--text-muted)">Empty</div>';
    }
    
    detailsEl.innerHTML = html;
  }
}

function handleTerritoryClick(hit) {
  if (!gameState) return;
  
  const phase = gameState.current_phase;
  
  if (phase === 'Mobilize' && hit.type === 'land') {
    // Try to place a unit here
    const pending = gameState.pending_purchases;
    if (pending?.length) {
      const [type] = pending[0];
      submitAction({ PlaceUnit: { unit_type: type, territory_id: hit.id } });
    }
  }
}

// ---- Phase Progression ----
function endPhase() {
  if (!gameState) return;
  
  const phase = gameState.current_phase;
  const confirmActions = {
    PurchaseAndRepair: 'ConfirmPurchases',
    CombatMovement: 'ConfirmCombatMovement',
    NonCombatMovement: 'ConfirmNonCombatMovement',
    Mobilize: 'ConfirmMobilization',
    CollectIncome: 'ConfirmIncome',
    ConductCombat: 'ConfirmPhase',
  };
  
  const action = confirmActions[phase];
  if (action) {
    submitAction(action);
  }
}

// ---- Logging ----
function log(msg) {
  const logDiv = document.getElementById('log-content');
  const entry = document.createElement('div');
  entry.className = 'log-entry';
  entry.textContent = msg;
  logDiv.prepend(entry);
}

function logEvent(evt) {
  const type = Object.keys(evt)[0] || 'Unknown';
  const data = evt[type] || evt;
  
  const entry = document.createElement('div');
  entry.className = 'log-entry';
  
  switch(type) {
    case 'PhaseChanged':
      entry.innerHTML = `<span class="log-event">Phase:</span> ${data.to}`;
      break;
    case 'TurnChanged':
      entry.innerHTML = `<span class="log-event">Turn ${data.turn}:</span> ${POWER_COLORS[data.power]?.name || data.power}`;
      break;
    case 'UnitsPurchased':
      entry.innerHTML = `<span class="log-event">Purchased:</span> ${data.count}× ${data.unit_type} (${data.cost} IPC)`;
      break;
    case 'IncomeCollected':
      entry.innerHTML = `<span class="log-event">Income:</span> ${POWER_COLORS[data.power]?.name} +${data.amount} IPC`;
      break;
    case 'VictoryAchieved':
      entry.innerHTML = `<span class="log-event">🏆 VICTORY:</span> ${data.winner} wins!`;
      break;
    default:
      entry.innerHTML = `<span class="log-event">${type}</span>`;
  }
  
  document.getElementById('log-content').prepend(entry);
}

// ---- Start ----
init();
