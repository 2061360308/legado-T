<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { LayoutPage } from "@/api/bindings";
import { getCachedPageImage, preloadPageImages } from "./drawPage";
import { drawInfoBar, type InfoBarPayload } from "../infoBar";

const props = withDefaults(
  defineProps<{
    page: LayoutPage | null;
    width: number;
    height: number;
    scrollTop: number;
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
let raf = 0;

function scheduleDraw() {
  if (raf) return;
  raf = requestAnimationFrame(() => {
    raf = 0;
    draw();
  });
}

function draw() {
  const canvas = canvasRef.value;
  if (!canvas) return;
  preloadPageImages(props.page, scheduleDraw);
  const ctx = canvas.getContext("2d");
  if (!ctx) return;

  const dpr = window.devicePixelRatio || 1;
  const bufferWidth = Math.max(1, Math.floor(props.width * dpr));
  const bufferHeight = Math.max(1, Math.floor(props.height * dpr));
  if (canvas.width !== bufferWidth || canvas.height !== bufferHeight) {
    canvas.width = bufferWidth;
    canvas.height = bufferHeight;
    canvas.style.width = `${props.width}px`;
    canvas.style.height = `${props.height}px`;
  }
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  ctx.fillStyle = props.background;
  ctx.fillRect(0, 0, props.width, props.height);

  const background = props.backgroundImage;
  if (background && background.complete && background.naturalWidth > 0) {
    const scale = Math.max(props.width / background.naturalWidth, props.height / background.naturalHeight);
    const sourceWidth = props.width / scale;
    const sourceHeight = props.height / scale;
    ctx.save();
    ctx.globalAlpha = props.backgroundAlpha ?? 1;
    ctx.drawImage(
      background,
      (background.naturalWidth - sourceWidth) / 2,
      (background.naturalHeight - sourceHeight) / 2,
      sourceWidth,
      sourceHeight,
      0,
      0,
      props.width,
      props.height,
    );
    ctx.restore();
  }

  const page = props.page;
  if (page) {
    const top = props.scrollTop;
    const bottom = top + props.height;
    ctx.save();
    ctx.translate(0, -top);
    page.lines.forEach((line, index) => {
      if (line.y + line.height < top - 80 || line.y > bottom + 80) return;
      if (index === props.highlightLine) {
        ctx.fillStyle = "rgba(255, 213, 79, 0.35)";
        ctx.fillRect(line.x - 2, line.y - 2, line.width + 4, line.height);
      }
      if (line.image) {
        const image = getCachedPageImage(line.image.src);
        if (image && image.complete && image.naturalWidth > 0) {
          ctx.drawImage(image, line.x, line.y, line.width, line.height);
        }
        return;
      }
      ctx.fillStyle = props.textColor;
      const weight = line.light ? "300 " : line.bold ? "600 " : "";
      ctx.font = `${weight}${line.fontSize}px ${props.fontFamily}`;
      for (const char of line.chars) {
        ctx.fillText(char.ch, char.x, char.y);
      }
      if (props.underline && line.width > 0) {
        ctx.fillRect(line.x, line.y + 2, line.width, 1);
      }
    });
    ctx.restore();
  }

  if (props.infoBar) {
    drawInfoBar(ctx, props.infoBar, props.width, props.height);
  }
}

watch(
  () => [
    props.page,
    props.width,
    props.height,
    props.scrollTop,
    props.highlightLine,
    props.underline,
    props.infoBar,
    props.backgroundImage,
    props.backgroundAlpha,
    props.background,
    props.textColor,
    props.fontFamily,
  ],
  scheduleDraw,
);

onMounted(draw);
</script>

<template>
  <canvas ref="canvasRef" class="scroll-reader-canvas" />
</template>

<style scoped>
.scroll-reader-canvas {
  display: block;
}
</style>
