/** 输入：键盘 / 鼠标 / 触屏 / 虚拟摇杆 */

import { TILE, BUILD_ORDER, DIRS } from './config.js';

export function createInput(canvas) {
  const input = {
    keys: new Set(),
    mouse: { x: 0, y: 0, down: false, right: false },
    worldAim: null,
    hoverTile: { x: 0, y: 0 },
    mining: false,
    shooting: false,
    placeRequest: null, // { x, y }
    removeRequest: null,
    rotateRequest: false,
    selectBuild: null, // type id or null (null = 选择/拆除模式可另行处理)
    stick: { active: false, x: 0, y: 0, id: null },
    cam: { x: 0, y: 0 },
    buildIndex: 0,
  };

  window.addEventListener('keydown', (e) => {
    input.keys.add(e.code);
    if (e.code === 'KeyR') input.rotateRequest = true;
    if (e.code === 'KeyE') input.mining = true;
    if (e.code === 'Space') {
      e.preventDefault();
      input.shooting = true;
    }
    if (e.code === 'KeyQ' || e.code === 'Escape') input.selectBuild = null;
    if (e.code === 'KeyX' || e.code === 'Backspace') {
      input.removeRequest = { ...input.hoverTile };
    }
    // 数字键选建筑 1-5
    const num = Number(e.key);
    if (num >= 1 && num <= BUILD_ORDER.length) {
      input.selectBuild = BUILD_ORDER[num - 1];
      input.buildIndex = num - 1;
    }
    if (e.code === 'Tab') {
      e.preventDefault();
      input.buildIndex = (input.buildIndex + 1) % BUILD_ORDER.length;
      input.selectBuild = BUILD_ORDER[input.buildIndex];
    }
  });

  window.addEventListener('keyup', (e) => {
    input.keys.delete(e.code);
    if (e.code === 'KeyE') input.mining = false;
    if (e.code === 'Space') input.shooting = false;
  });

  canvas.addEventListener('contextmenu', (e) => e.preventDefault());

  canvas.addEventListener('mousedown', (e) => {
    updateMouse(input, canvas, e);
    if (e.button === 0) {
      input.mouse.down = true;
      if (input.selectBuild) {
        input.placeRequest = { ...input.hoverTile };
      } else {
        input.shooting = true;
      }
    }
    if (e.button === 2) {
      input.mouse.right = true;
      input.removeRequest = { ...input.hoverTile };
    }
  });

  canvas.addEventListener('mouseup', (e) => {
    if (e.button === 0) {
      input.mouse.down = false;
      input.shooting = false;
    }
    if (e.button === 2) input.mouse.right = false;
  });

  canvas.addEventListener('mousemove', (e) => {
    updateMouse(input, canvas, e);
    if (input.mouse.down && input.selectBuild && e.buttons & 1) {
      // 拖拽连续建造（传送带友好）
      input.placeRequest = { ...input.hoverTile };
    }
    if (input.mouse.right || e.buttons & 2) {
      input.removeRequest = { ...input.hoverTile };
    }
  });

  // 触屏
  canvas.addEventListener('touchstart', (e) => {
    e.preventDefault();
    for (const t of e.changedTouches) {
      const rect = canvas.getBoundingClientRect();
      const x = t.clientX - rect.left;
      const y = t.clientY - rect.top;
      // 左下虚拟摇杆区
      if (x < rect.width * 0.38 && y > rect.height * 0.55) {
        input.stick.active = true;
        input.stick.id = t.identifier;
        updateStick(input, x, y, rect);
      } else if (x > rect.width * 0.7 && y > rect.height * 0.55) {
        // 右下：挖矿 / 射击
        input.mining = true;
        if (!input.selectBuild) input.shooting = true;
        else input.placeRequest = screenToTile(input, canvas, x, y);
      } else {
        updatePointer(input, canvas, x, y);
        if (input.selectBuild) input.placeRequest = { ...input.hoverTile };
        else input.shooting = true;
      }
    }
  }, { passive: false });

  canvas.addEventListener('touchmove', (e) => {
    e.preventDefault();
    const rect = canvas.getBoundingClientRect();
    for (const t of e.changedTouches) {
      const x = t.clientX - rect.left;
      const y = t.clientY - rect.top;
      if (input.stick.active && t.identifier === input.stick.id) {
        updateStick(input, x, y, rect);
      } else {
        updatePointer(input, canvas, x, y);
        if (input.selectBuild) input.placeRequest = { ...input.hoverTile };
      }
    }
  }, { passive: false });

  canvas.addEventListener('touchend', (e) => {
    for (const t of e.changedTouches) {
      if (t.identifier === input.stick.id) {
        input.stick.active = false;
        input.stick.x = 0;
        input.stick.y = 0;
        input.stick.id = null;
      }
    }
    if (e.touches.length === 0) {
      input.mining = false;
      input.shooting = false;
    }
  });

  return input;
}

function updateMouse(input, canvas, e) {
  const rect = canvas.getBoundingClientRect();
  const x = e.clientX - rect.left;
  const y = e.clientY - rect.top;
  updatePointer(input, canvas, x, y);
}

function updatePointer(input, canvas, x, y) {
  input.mouse.x = x;
  input.mouse.y = y;
  const scaleX = canvas.width / canvas.getBoundingClientRect().width;
  const scaleY = canvas.height / canvas.getBoundingClientRect().height;
  const wx = (x * scaleX + input.cam.x) / TILE;
  const wy = (y * scaleY + input.cam.y) / TILE;
  input.worldAim = { x: wx, y: wy };
  input.hoverTile = { x: Math.floor(wx), y: Math.floor(wy) };
}

function screenToTile(input, canvas, x, y) {
  updatePointer(input, canvas, x, y);
  return { ...input.hoverTile };
}

function updateStick(input, x, y, rect) {
  const cx = rect.width * 0.18;
  const cy = rect.height * 0.78;
  const dx = (x - cx) / (rect.width * 0.12);
  const dy = (y - cy) / (rect.width * 0.12);
  const len = Math.hypot(dx, dy) || 1;
  const mag = Math.min(1, len);
  input.stick.x = (dx / len) * mag;
  input.stick.y = (dy / len) * mag;
}

export function consumeFlags(input) {
  const flags = {
    placeRequest: input.placeRequest,
    removeRequest: input.removeRequest,
    rotateRequest: input.rotateRequest,
  };
  input.placeRequest = null;
  input.removeRequest = null;
  input.rotateRequest = false;
  return flags;
}

export function conveyorRotFromDrag(prev, cur) {
  if (!prev) return 0;
  const dx = cur.x - prev.x;
  const dy = cur.y - prev.y;
  if (dx === 0 && dy === 0) return null;
  if (Math.abs(dx) >= Math.abs(dy)) return dx > 0 ? 0 : 2;
  return dy > 0 ? 1 : 3;
}

export { DIRS };
