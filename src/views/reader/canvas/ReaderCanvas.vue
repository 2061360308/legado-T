<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { LayoutPage } from "@/api/bindings";
import { drawPage, preloadPageImages } from "./drawPage";
import type { InfoBarPayload } from "../infoBar";

const props = withDefaults(
  defineProps<{
    page: LayoutPage | null;
    width: number;
    height: number;
    background?: string;
    backgroundImage?: HTMLImageElement | null;
    backgroundAlpha?: number;
    textColor?: string;
    highlightLine?: number;
    fontFamily?: string;
    underline?: boolean;
    infoBar?: InfoBarPayload | null;
  }>(),
  {
    background: "#f7f1e3",
    backgroundImage: null,
    backgroundAlpha: 1,
    textColor: "#2f2a24",
    highlightLine: -1,
    fontFamily: 'system-ui, "Microsoft YaHei", "PingFang SC", sans-serif',
    underline: false,
    infoBar: undefined,
  },
);

const canvasRef = ref<HTMLCanvasElement | null>(null);

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  preloadPageImages(props.page, draw);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.floor(props.width * dpr));
  canvas.height = Math.max(1, Math.floor(props.height * dpr));
  canvas.style.width = `${props.width}px`;
  canvas.style.height = `${props.height}px`;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

  drawPage(ctx, props.page, {
    width: props.width,
    height: props.height,
    background: props.background,
    textColor: props.textColor,
    fontFamily: props.fontFamily,
    underline: props.underline,
    highlightLine: props.highlightLine,
    backgroundImage: props.backgroundImage,
    backgroundAlpha: props.backgroundAlpha,
    infoBar: props.infoBar ?? undefined,
  });
}

watch(
  () => [props.page, props.width, props.height, props.highlightLine, props.underline, props.infoBar, props.backgroundImage, props.backgroundAlpha],
  () => draw(),
);

onMounted(draw);
</script>

<template>
  <canvas ref="canvasRef" class="reader-canvas" />
</template>

<style scoped>
.reader-canvas {
  display: block;
}
</style>
