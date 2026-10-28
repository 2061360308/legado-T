<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import type { LayoutPage } from "@/api/bindings";
import { renderPageTo, type PageDrawOptions } from "./drawPage";
import type { InfoBarPayload } from "../infoBar";
import {
  calcCornerXY,
  createCurlState,
  drawCurl,
  type CurlState,
  type TurnDirection,
} from "./simulationCurl";

const MAX_DPR = 1.5;

const props = defineProps<{
  mode: "cover" | "simulation";
  direction: TurnDirection;
  fromPage: LayoutPage | null;
  toPage: LayoutPage | null;
  width: number;
  height: number;
  background: string;
  textColor: string;
  fontFamily: string;
  underline?: boolean;
  brightness?: number;
  backgroundImage?: HTMLImageElement | null;
  backgroundAlpha?: number;
  fromInfoBar?: InfoBarPayload | null;
  toInfoBar?: InfoBarPayload | null;
}>();

const emit = defineEmits<{ done: [completed: boolean] }>();

const canvasRef = ref<HTMLCanvasElement | null>(null);
let fromCanvas: HTMLCanvasElement | null = null;
let toCanvas: HTMLCanvasElement | null = null;

// 对应 ReadView.startX/startY、touchX/touchY
let startPosX = 0;
let startPosY = 0;
let touchX = 0;
let touchY = 0;
let lastDx = 0;
let curl: CurlState = createCurlState(props.width, props.height);
let raf = 0;
let drawRaf = 0;
let interactive = false;

function renderDpr(): number {
  return Math.min(window.devicePixelRatio || 1, MAX_DPR);
}

function baseOptions(): PageDrawOptions {
  return {
    width: props.width,
    height: props.height,
    background: props.background,
    textColor: props.textColor,
    fontFamily: props.fontFamily,
    underline: props.underline,
    highlightLine: -1,
    backgroundImage: props.backgroundImage ?? null,
    backgroundAlpha: props.backgroundAlpha ?? 1,
  };
}

function renderBitmaps() {
  const dpr = renderDpr();
  const brightness = props.brightness ?? 1;
  fromCanvas = renderPageTo(
    fromCanvas ?? document.createElement("canvas"),
    props.fromPage,
    { ...baseOptions(), infoBar: props.fromInfoBar ?? undefined },
    dpr,
    brightness,
  );
  toCanvas = renderPageTo(
    toCanvas ?? document.createElement("canvas"),
    props.toPage ?? props.fromPage,
    { ...baseOptions(), infoBar: (props.toInfoBar ?? props.fromInfoBar) ?? undefined },
    dpr,
    brightness,
  );
  draw();
}

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const dpr = renderDpr();
  const bufferWidth = Math.max(1, Math.floor(props.width * dpr));
  const bufferHeight = Math.max(1, Math.floor(props.height * dpr));
  if (canvas.width !== bufferWidth || canvas.height !== bufferHeight) {
    canvas.width = bufferWidth;
    canvas.height = bufferHeight;
    canvas.style.width = `${props.width}px`;
    canvas.style.height = `${props.height}px`;
  }
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, props.width, props.height);

  const from = fromCanvas;
  const to = toCanvas;
  if (!from || !to) return;

  if (!interactive) {
    ctx.drawImage(from, 0, 0, props.width, props.height);
    return;
  }

  if (props.mode === "cover") {
    drawCover(ctx, from, to);
  } else {
    // 与 ReadView.dispatchDraw 一致：底层始终是当前页（curPage）
    ctx.drawImage(from, 0, 0, props.width, props.height);
    drawCurl(ctx, curl, from, to, props.direction, props.background);
  }
}

function scheduleDraw() {
  if (drawRaf) return;
  drawRaf = requestAnimationFrame(() => {
    drawRaf = 0;
    draw();
  });
}

function cancelScheduledDraw() {
  if (drawRaf) {
    cancelAnimationFrame(drawRaf);
    drawRaf = 0;
  }
}

function drawCover(ctx: CanvasRenderingContext2D, from: HTMLCanvasElement, to: HTMLCanvasElement) {
  const { width, height } = props;
  if (props.direction === "next") {
    const offset = Math.min(0, Math.max(-width, touchX - startPosX));
    ctx.drawImage(to, 0, 0, width, height);
    ctx.save();
    ctx.translate(offset, 0);
    ctx.drawImage(from, 0, 0, width, height);
    ctx.restore();
    drawSeamShadow(ctx, width + offset);
  } else {
    const offset = Math.min(width, Math.max(0, touchX - startPosX));
    ctx.drawImage(from, 0, 0, width, height);
    ctx.save();
    ctx.translate(offset - width, 0);
    ctx.drawImage(to, 0, 0, width, height);
    ctx.restore();
    drawSeamShadow(ctx, offset);
  }
}

function drawSeamShadow(ctx: CanvasRenderingContext2D, x: number) {
  const left = Math.max(0, Math.min(props.width, x));
  if (left >= props.width) return;
  const gradient = ctx.createLinearGradient(left, 0, left + 30, 0);
  gradient.addColorStop(0, "rgba(17,17,17,0.4)");
  gradient.addColorStop(1, "rgba(17,17,17,0)");
  ctx.fillStyle = gradient;
  ctx.fillRect(left, 0, 30, props.height);
}

// 对应 calcCornerXY(ACTION_DOWN) + setDirection
function applyDirectionCorner(direction: TurnDirection, refX: number, refY: number) {
  curl = createCurlState(props.width, props.height);
  calcCornerXY(curl, refX, refY);
  if (direction === "prev") {
    // 上一页滑动不出现对角
    if (refX > props.width / 2) calcCornerXY(curl, refX, props.height);
    else calcCornerXY(curl, props.width - refX, props.height);
  } else if (props.width / 2 > refX) {
    calcCornerXY(curl, props.width - refX, refY);
  }
}

function clampX(value: number): number {
  return Math.min(Math.max(value, 0), props.width);
}

function clampY(value: number): number {
  return Math.min(Math.max(value, 0), props.height);
}

// 跟手拖动：down 点决定折角，start 点为越过滑动阈值的位置
function beginDrag(
  direction: TurnDirection,
  downX: number,
  downY: number,
  startX: number,
  startY: number,
) {
  stopAnimation();
  interactive = true;
  applyDirectionCorner(direction, clampX(downX), clampY(downY));
  startPosX = clampX(startX);
  startPosY = clampY(startY);
  touchX = startPosX;
  touchY = startPosY;
  lastDx = 0;
  curl.touchX = touchX;
  curl.touchY = touchY;
  draw();
}

// 对应 SimulationPageDelegate.onTouch(ACTION_MOVE)
function follow(x: number, y: number) {
  const previousX = touchX;
  // 安卓上手指无法离开屏幕，这里把拖动坐标限制在页面内，到达边缘后折痕不再跟手
  touchX = clampX(x);
  touchY = clampY(y);
  lastDx = touchX - previousX;
  const { height } = props;
  if ((startPosY > height / 3 && startPosY < (height * 2) / 3) || props.direction === "prev") {
    touchY = height;
  }
  if (startPosY > height / 3 && startPosY < height / 2 && props.direction === "next") {
    touchY = 1;
  }
  curl.touchX = touchX;
  curl.touchY = touchY;
  scheduleDraw();
}

function setTouch(x: number, y: number) {
  touchX = x;
  touchY = y;
  curl.touchX = x;
  curl.touchY = y;
  draw();
}

// 对应 onAnimStart
function release(speed = 300) {
  const cancel = props.direction === "next" ? lastDx > 0 : lastDx < 0;
  animateTo(!cancel, speed);
}

function animateTo(complete: boolean, animationSpeed = 300) {
  stopAnimation();
  const { width, height } = props;
  let dx = 0;
  let dy = 0;
  let endX: number;
  let endY: number;

  if (props.mode === "cover") {
    endX = complete
      ? props.direction === "next"
        ? startPosX - width
        : startPosX + width
      : startPosX;
    endY = startPosY;
    dx = endX - touchX;
    dy = 0;
  } else {
    const { cornerX, cornerY } = curl;
    if (!complete) {
      if (props.direction === "next") {
        dx = cornerX > 0 ? width - touchX : -touchX;
      } else {
        dx = -(width + touchX);
      }
      dy = cornerY > 0 ? height - touchY : -touchY;
    } else {
      if (props.direction === "next") {
        dx = cornerX > 0 ? -(width + touchX) : width - touchX;
      } else {
        dx = width - touchX;
      }
      dy = cornerY > 0 ? height - touchY : 1 - touchY;
    }
    endX = touchX + dx;
    endY = touchY + dy;
  }

  const distance = dx !== 0 ? Math.abs(dx) : Math.abs(dy);
  const span = dx !== 0 ? width : height;
  const duration = Math.max(80, (animationSpeed * distance) / span);
  const fromX = touchX;
  const fromY = touchY;
  const startTime = performance.now();

  const step = (now: number) => {
    try {
      const t = Math.min(1, (now - startTime) / duration);
      const eased = 1 - Math.pow(1 - t, 3);
      setTouch(fromX + (endX - fromX) * eased, fromY + (endY - fromY) * eased);
      if (t < 1) {
        raf = requestAnimationFrame(step);
      } else {
        raf = 0;
        emit("done", complete);
      }
    } catch {
      raf = 0;
      emit("done", false);
    }
  };
  raf = requestAnimationFrame(step);
}

// 对应 nextPageByAnim / prevPageByAnim
function autoFrom(direction: TurnDirection, refX: number, refY: number, speed = 300) {
  stopAnimation();
  interactive = true;
  applyDirectionCorner(direction, refX, refY);
  if (direction === "next") {
    startPosX = props.width * 0.9;
    startPosY = refY > props.height / 2 ? props.height * 0.9 : 1;
  } else {
    startPosX = 0;
    startPosY = props.height;
  }
  touchX = startPosX;
  touchY = startPosY;
  lastDx = 0;
  curl.touchX = touchX;
  curl.touchY = touchY;
  draw();
  animateTo(true, speed);
}

function stopAnimation() {
  if (raf) {
    cancelAnimationFrame(raf);
    raf = 0;
  }
}

function stop() {
  stopAnimation();
  cancelScheduledDraw();
}

function showStatic() {
  interactive = false;
  draw();
}

defineExpose({ beginDrag, follow, release, autoFrom, stop, showStatic, render: renderBitmaps });

watch(
  () => [
    props.fromPage,
    props.toPage,
    props.width,
    props.height,
    props.background,
    props.textColor,
    props.fontFamily,
    props.underline,
    props.brightness,
    props.backgroundImage,
    props.backgroundAlpha,
    props.fromInfoBar,
    props.toInfoBar,
  ],
  () => renderBitmaps(),
);

onMounted(renderBitmaps);
onBeforeUnmount(stop);
</script>

<template>
  <canvas ref="canvasRef" class="turn-canvas" />
</template>

<style scoped>
.turn-canvas {
  display: block;
}
</style>
