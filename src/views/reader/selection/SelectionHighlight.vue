<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { LayoutPage } from "@/api/bindings";
import { selectionRects, type TextSelection } from "./textSelection";

const props = withDefaults(
  defineProps<{
    page: LayoutPage;
    selection: TextSelection;
    width: number;
    height: number;
    scrollTop?: number;
    offsetX?: number;
    color?: string;
  }>(),
  { scrollTop: 0, offsetX: 0, color: "rgba(255, 213, 79, 0.38)" },
);

const canvasRef = ref<HTMLCanvasElement | null>(null);

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  const dpr = window.devicePixelRatio || 1;
  canvas.width = Math.max(1, Math.floor(props.width * dpr));
  canvas.height = Math.max(1, Math.floor(props.height * dpr));
  canvas.style.width = `${props.width}px`;
  canvas.style.height = `${props.height}px`;
  const ctx = canvas.getContext("2d");
  if (!ctx) return;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.clearRect(0, 0, props.width, props.height);
  ctx.fillStyle = props.color;
  for (const rect of selectionRects(props.page, props.selection)) {
    const y = rect.y - (props.scrollTop ?? 0);
    if (y + rect.height < 0 || y > props.height) continue;
    ctx.fillRect(rect.x + (props.offsetX ?? 0), y, rect.width, rect.height);
  }
}

watch(
  () => [props.page, props.selection, props.width, props.height, props.scrollTop, props.offsetX, props.color],
  draw,
  { deep: true },
);

onMounted(draw);
</script>

<template>
  <canvas ref="canvasRef" class="selection-highlight" />
</template>

<style scoped>
.selection-highlight {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 2;
}
</style>
