export type TurnDirection = "next" | "prev";

export interface CurlState {
  width: number;
  height: number;
  touchX: number;
  touchY: number;
  cornerX: number;
  cornerY: number;
  isRtOrLb: boolean;
  middleX: number;
  middleY: number;
  c1x: number;
  c1y: number;
  c2x: number;
  c2y: number;
  start1x: number;
  start1y: number;
  start2x: number;
  start2y: number;
  end1x: number;
  end1y: number;
  end2x: number;
  end2y: number;
  vertex1x: number;
  vertex1y: number;
  vertex2x: number;
  vertex2y: number;
  touchToCornerDis: number;
  degrees: number;
}

const EPS = 0.1;
const MAX_LENGTH_RATIO = 25 * 1.414;

const FOLDER_COLORS: [string, string] = ["rgba(51,51,51,0)", "rgba(51,51,51,0.69)"];
const BACK_COLORS: [string, string] = ["rgba(17,17,17,1)", "rgba(17,17,17,0)"];
const FRONT_COLORS: [string, string] = ["rgba(17,17,17,0.5)", "rgba(17,17,17,0)"];

export function createCurlState(width: number, height: number): CurlState {
  return {
    width,
    height,
    touchX: 0.1,
    touchY: 0.1,
    cornerX: 0,
    cornerY: 0,
    isRtOrLb: false,
    middleX: 0,
    middleY: 0,
    c1x: 0,
    c1y: 0,
    c2x: 0,
    c2y: 0,
    start1x: 0,
    start1y: 0,
    start2x: 0,
    start2y: 0,
    end1x: 0,
    end1y: 0,
    end2x: 0,
    end2y: 0,
    vertex1x: 0,
    vertex1y: 0,
    vertex2x: 0,
    vertex2y: 0,
    touchToCornerDis: 0,
    degrees: 0,
  };
}

export function calcCornerXY(state: CurlState, x: number, y: number): void {
  state.cornerX = x <= state.width / 2 ? 0 : state.width;
  state.cornerY = y <= state.height / 2 ? 0 : state.height;
  state.isRtOrLb =
    (state.cornerX === 0 && state.cornerY === state.height) ||
    (state.cornerY === 0 && state.cornerX === state.width);
}

export function updateCurl(state: CurlState): void {
  const { width } = state;
  state.middleX = (state.touchX + state.cornerX) / 2;
  state.middleY = (state.touchY + state.cornerY) / 2;

  const recomputeControls = () => {
    const dx = state.cornerX - state.middleX;
    const dy = state.cornerY - state.middleY;
    state.c1x = state.middleX - (dy * dy) / (dx || EPS);
    state.c1y = state.cornerY;
    state.c2x = state.cornerX;
    state.c2y = state.middleY - (dx * dx) / (dy || EPS);
    state.start1x = state.c1x - (state.cornerX - state.c1x) / 2;
    state.start1y = state.cornerY;
  };

  recomputeControls();

  if (state.touchX > 0 && state.touchX < width) {
    if (state.start1x < 0 || state.start1x > width) {
      if (state.start1x < 0) state.start1x = width - state.start1x;
      const f1 = Math.abs(state.cornerX - state.touchX) || EPS;
      const f2 = (width * f1) / (state.start1x || EPS);
      state.touchX = Math.abs(state.cornerX - f2);
      const f3 = (Math.abs(state.cornerX - state.touchX) * Math.abs(state.cornerY - state.touchY)) / f1;
      state.touchY = Math.abs(state.cornerY - f3);
      state.middleX = (state.touchX + state.cornerX) / 2;
      state.middleY = (state.touchY + state.cornerY) / 2;
      recomputeControls();
    }
  }

  state.start2x = state.cornerX;
  state.start2y = state.c2y - (state.cornerY - state.c2y) / 2;
  state.touchToCornerDis = Math.hypot(state.touchX - state.cornerX, state.touchY - state.cornerY);

  const cross1 = getCross(state.touchX, state.touchY, state.c1x, state.c1y, state.start1x, state.start1y, state.start2x, state.start2y);
  const cross2 = getCross(state.touchX, state.touchY, state.c2x, state.c2y, state.start1x, state.start1y, state.start2x, state.start2y);
  state.end1x = cross1.x;
  state.end1y = cross1.y;
  state.end2x = cross2.x;
  state.end2y = cross2.y;

  state.vertex1x = (state.start1x + 2 * state.c1x + state.end1x) / 4;
  state.vertex1y = (2 * state.c1y + state.start1y + state.end1y) / 4;
  state.vertex2x = (state.start2x + 2 * state.c2x + state.end2x) / 4;
  state.vertex2y = (2 * state.c2y + state.start2y + state.end2y) / 4;
}

function getCross(
  x1: number,
  y1: number,
  x2: number,
  y2: number,
  x3: number,
  y3: number,
  x4: number,
  y4: number,
): { x: number; y: number } {
  const a1 = (y2 - y1) / ((x2 - x1) || EPS);
  const b1 = (x1 * y2 - x2 * y1) / ((x1 - x2) || EPS);
  const a2 = (y4 - y3) / ((x4 - x3) || EPS);
  const b2 = (x3 * y4 - x4 * y3) / ((x3 - x4) || EPS);
  const x = (b2 - b1) / ((a1 - a2) || EPS);
  return { x, y: a1 * x + b1 };
}

function path0(state: CurlState): Path2D {
  const p = new Path2D();
  p.moveTo(state.start1x, state.start1y);
  p.quadraticCurveTo(state.c1x, state.c1y, state.end1x, state.end1y);
  p.lineTo(state.touchX, state.touchY);
  p.lineTo(state.end2x, state.end2y);
  p.quadraticCurveTo(state.c2x, state.c2y, state.start2x, state.start2y);
  p.lineTo(state.cornerX, state.cornerY);
  p.closePath();
  return p;
}

function nextAreaPath(state: CurlState): Path2D {
  const p = new Path2D();
  p.moveTo(state.start1x, state.start1y);
  p.lineTo(state.vertex1x, state.vertex1y);
  p.lineTo(state.vertex2x, state.vertex2y);
  p.lineTo(state.start2x, state.start2y);
  p.lineTo(state.cornerX, state.cornerY);
  p.closePath();
  return p;
}

function backAreaPath(state: CurlState): Path2D {
  const p = new Path2D();
  p.moveTo(state.vertex2x, state.vertex2y);
  p.lineTo(state.vertex1x, state.vertex1y);
  p.lineTo(state.end1x, state.end1y);
  p.lineTo(state.touchX, state.touchY);
  p.lineTo(state.end2x, state.end2y);
  p.closePath();
  return p;
}

function outside(state: CurlState, p: Path2D): Path2D {
  const full = new Path2D();
  full.rect(0, 0, state.width, state.height);
  full.addPath(p);
  return full;
}

type ShadowOrientation = "LR" | "RL" | "TB" | "BT";

function drawShadow(
  ctx: CanvasRenderingContext2D,
  orientation: ShadowOrientation,
  colors: [string, string],
  left: number,
  top: number,
  right: number,
  bottom: number,
): void {
  if (right - left === 0 || bottom - top === 0) return;
  const midX = (left + right) / 2;
  const midY = (top + bottom) / 2;
  let gradient: CanvasGradient;
  if (orientation === "LR") gradient = ctx.createLinearGradient(left, midY, right, midY);
  else if (orientation === "RL") gradient = ctx.createLinearGradient(right, midY, left, midY);
  else if (orientation === "TB") gradient = ctx.createLinearGradient(midX, top, midX, bottom);
  else gradient = ctx.createLinearGradient(midX, bottom, midX, top);
  gradient.addColorStop(0, colors[0]);
  gradient.addColorStop(1, colors[1]);
  ctx.fillStyle = gradient;
  ctx.fillRect(left, top, right - left, bottom - top);
}

function rotateAround(ctx: CanvasRenderingContext2D, degrees: number, cx: number, cy: number): void {
  ctx.translate(cx, cy);
  ctx.rotate((degrees * Math.PI) / 180);
  ctx.translate(-cx, -cy);
}

function drawCurrentPageArea(ctx: CanvasRenderingContext2D, state: CurlState, bitmap: HTMLCanvasElement): void {
  const p0 = path0(state);
  ctx.save();
  ctx.beginPath();
  ctx.clip(outside(state, p0), "evenodd");
  ctx.drawImage(bitmap, 0, 0, state.width, state.height);
  ctx.restore();
}

function drawNextPageAreaAndShadow(ctx: CanvasRenderingContext2D, state: CurlState, bitmap: HTMLCanvasElement): void {
  const p0 = path0(state);
  const p1 = nextAreaPath(state);
  ctx.save();
  ctx.beginPath();
  ctx.clip(p0);
  ctx.clip(p1);
  ctx.drawImage(bitmap, 0, 0, state.width, state.height);

  state.degrees = (Math.atan2(state.c1x - state.cornerX, state.c2y - state.cornerY) * 180) / Math.PI;
  rotateAround(ctx, state.degrees, state.start1x, state.start1y);

  let left: number;
  let right: number;
  let orientation: ShadowOrientation;
  if (state.isRtOrLb) {
    left = state.start1x;
    right = state.start1x + state.touchToCornerDis / 4;
    orientation = "LR";
  } else {
    left = state.start1x - state.touchToCornerDis / 4;
    right = state.start1x;
    orientation = "RL";
  }
  drawShadow(ctx, orientation, BACK_COLORS, left, state.start1y, right, state.start1y + Math.hypot(state.width, state.height));
  ctx.restore();
}

function drawCurrentPageShadow(ctx: CanvasRenderingContext2D, state: CurlState): void {
  const p0 = path0(state);
  const maxLength = Math.hypot(state.width, state.height);
  const degree = state.isRtOrLb
    ? Math.PI / 4 - Math.atan2(state.c1y - state.touchY, state.touchX - state.c1x)
    : Math.PI / 4 - Math.atan2(state.touchY - state.c1y, state.touchX - state.c1x);
  const d1 = MAX_LENGTH_RATIO * Math.cos(degree);
  const d2 = MAX_LENGTH_RATIO * Math.sin(degree);
  const x = state.touchX + d1;
  const y = state.isRtOrLb ? state.touchY + d2 : state.touchY - d2;

  // 折角阴影 1
  const wedge1 = new Path2D();
  wedge1.moveTo(x, y);
  wedge1.lineTo(state.touchX, state.touchY);
  wedge1.lineTo(state.c1x, state.c1y);
  wedge1.lineTo(state.start1x, state.start1y);
  wedge1.closePath();
  ctx.save();
  ctx.beginPath();
  ctx.clip(outside(state, p0), "evenodd");
  ctx.clip(wedge1);
  let leftX: number;
  let rightX: number;
  let orientation: ShadowOrientation;
  if (state.isRtOrLb) {
    leftX = state.c1x;
    rightX = state.c1x + 25;
    orientation = "LR";
  } else {
    leftX = state.c1x - 25;
    rightX = state.c1x + 1;
    orientation = "RL";
  }
  const rotateDegrees = (Math.atan2(state.touchX - state.c1x, state.c1y - state.touchY) * 180) / Math.PI;
  rotateAround(ctx, rotateDegrees, state.c1x, state.c1y);
  drawShadow(ctx, orientation, FRONT_COLORS, leftX, state.c1y - maxLength, rightX, state.c1y);
  ctx.restore();

  // 折角阴影 2
  const wedge2 = new Path2D();
  wedge2.moveTo(x, y);
  wedge2.lineTo(state.touchX, state.touchY);
  wedge2.lineTo(state.c2x, state.c2y);
  wedge2.lineTo(state.start2x, state.start2y);
  wedge2.closePath();
  ctx.save();
  ctx.beginPath();
  ctx.clip(outside(state, p0), "evenodd");
  ctx.clip(wedge2);
  let top: number;
  let bottom: number;
  if (state.isRtOrLb) {
    top = state.c2y;
    bottom = state.c2y + 25;
    orientation = "TB";
  } else {
    top = state.c2y - 25;
    bottom = state.c2y + 1;
    orientation = "BT";
  }
  const rotateDegrees2 = (Math.atan2(state.c2y - state.touchY, state.c2x - state.touchX) * 180) / Math.PI;
  rotateAround(ctx, rotateDegrees2, state.c2x, state.c2y);
  const temp = state.c2y < 0 ? state.c2y - state.height : state.c2y;
  const hmg = Math.hypot(state.c2x, temp);
  if (hmg > maxLength) {
    drawShadow(ctx, orientation, FRONT_COLORS, state.c2x - 25 - hmg, top, state.c2x + maxLength - hmg, bottom);
  } else {
    drawShadow(ctx, orientation, FRONT_COLORS, state.c2x - maxLength, top, state.c2x, bottom);
  }
  ctx.restore();
}

function drawCurrentBackArea(
  ctx: CanvasRenderingContext2D,
  state: CurlState,
  bitmap: HTMLCanvasElement,
  background: string,
): void {
  const p0 = path0(state);
  const p1 = backAreaPath(state);
  const i = (state.start1x + state.c1x) / 2;
  const f1 = Math.abs(i - state.c1x);
  const i1 = (state.start2y + state.c2y) / 2;
  const f2 = Math.abs(i1 - state.c2y);
  const f3 = Math.min(f1, f2);

  ctx.save();
  ctx.beginPath();
  ctx.clip(p0);
  ctx.clip(p1);
  ctx.fillStyle = background;
  ctx.fillRect(0, 0, state.width, state.height);

  const dis = Math.hypot(state.cornerX - state.c1x, state.c2y - state.cornerY) || EPS;
  const f8 = (state.cornerX - state.c1x) / dis;
  const f9 = (state.c2y - state.cornerY) / dis;
  const a = 1 - 2 * f9 * f9;
  const b = 2 * f8 * f9;
  const d = 1 - 2 * f8 * f8;
  ctx.save();
  ctx.translate(state.c1x, state.c1y);
  ctx.transform(a, b, b, d, 0, 0);
  ctx.translate(-state.c1x, -state.c1y);
  ctx.drawImage(bitmap, 0, 0, state.width, state.height);
  ctx.restore();

  rotateAround(ctx, state.degrees, state.start1x, state.start1y);
  let left: number;
  let right: number;
  let orientation: ShadowOrientation;
  if (state.isRtOrLb) {
    left = state.start1x - 1;
    right = state.start1x + f3 + 1;
    orientation = "LR";
  } else {
    left = state.start1x - f3 - 1;
    right = state.start1x + 1;
    orientation = "RL";
  }
  drawShadow(ctx, orientation, FOLDER_COLORS, left, state.start1y, right, state.start1y + Math.hypot(state.width, state.height));
  ctx.restore();
}

function curlFinite(state: CurlState): boolean {
  return [
    state.touchX,
    state.touchY,
    state.cornerX,
    state.cornerY,
    state.middleX,
    state.middleY,
    state.c1x,
    state.c1y,
    state.c2x,
    state.c2y,
    state.start1x,
    state.start1y,
    state.start2x,
    state.start2y,
    state.end1x,
    state.end1y,
    state.end2x,
    state.end2y,
    state.vertex1x,
    state.vertex1y,
    state.vertex2x,
    state.vertex2y,
    state.touchToCornerDis,
    state.degrees,
  ].every((value) => Number.isFinite(value));
}

export function drawCurl(
  ctx: CanvasRenderingContext2D,
  state: CurlState,
  fromBitmap: HTMLCanvasElement,
  toBitmap: HTMLCanvasElement,
  direction: TurnDirection,
  background: string,
): void {
  updateCurl(state);
  if (!curlFinite(state)) {
    ctx.drawImage(toBitmap, 0, 0, state.width, state.height);
    return;
  }
  if (direction === "next") {
    drawCurrentPageArea(ctx, state, fromBitmap);
    drawNextPageAreaAndShadow(ctx, state, toBitmap);
    drawCurrentPageShadow(ctx, state);
    drawCurrentBackArea(ctx, state, fromBitmap, background);
  } else {
    drawCurrentPageArea(ctx, state, toBitmap);
    drawNextPageAreaAndShadow(ctx, state, fromBitmap);
    drawCurrentPageShadow(ctx, state);
    drawCurrentBackArea(ctx, state, toBitmap, background);
  }
}

export interface AutoTurnPath {
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  duration: number;
}

export function autoTurnPath(
  direction: TurnDirection,
  width: number,
  height: number,
  animationSpeed = 300,
): AutoTurnPath {
  if (direction === "next") {
    const startX = width * 0.9;
    const startY = 1;
    const endX = startX - (width + startX);
    const endY = 1;
    const duration = (animationSpeed * Math.abs(endX)) / width;
    return { startX, startY, endX, endY, duration };
  }
  const startX = 0;
  const startY = height;
  const endX = width - startX;
  const endY = 1;
  const duration = (animationSpeed * Math.abs(endX)) / width;
  return { startX, startY, endX, endY, duration };
}
