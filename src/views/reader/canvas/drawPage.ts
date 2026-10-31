import { platform } from "@/runtime";
import type { LayoutPage } from "@/core/domain";
import { drawInfoBar, type InfoBarPayload } from "../infoBar";

const pageImages = new Map<string, HTMLImageElement>();
const loadingImages = new Set<string>();

export function imageSource(src: string): string {
  if (/^(https?:|data:|asset:|blob:)/i.test(src)) return src;
  try {
    return platform.assets.fileSrc(src);
  } catch {
    return src;
  }
}

export function getCachedPageImage(src: string): HTMLImageElement | undefined {
  return pageImages.get(src);
}

export function preloadPageImages(page: LayoutPage | null, onLoad?: () => void): void {
  if (!page) return;
  for (const line of page.lines) {
    const src = line.image?.src;
    if (!src || pageImages.has(src) || loadingImages.has(src)) continue;
    loadingImages.add(src);
    const image = new Image();
    image.onload = () => {
      loadingImages.delete(src);
      pageImages.set(src, image);
      onLoad?.();
    };
    image.onerror = () => {
      loadingImages.delete(src);
    };
    image.src = imageSource(src);
  }
}

export interface PageDrawOptions {
  width: number;
  height: number;
  background: string;
  textColor: string;
  fontFamily: string;
  underline?: boolean;
  highlightLine?: number;
  backgroundImage?: HTMLImageElement | null;
  backgroundAlpha?: number;
  infoBar?: InfoBarPayload;
}

export function drawPage(
  ctx: CanvasRenderingContext2D,
  page: LayoutPage | null,
  options: PageDrawOptions,
): void {
  ctx.fillStyle = options.background;
  ctx.fillRect(0, 0, options.width, options.height);
  const image = options.backgroundImage;
  if (image && image.complete && image.naturalWidth > 0) {
    const scale = Math.max(options.width / image.naturalWidth, options.height / image.naturalHeight);
    const sourceWidth = options.width / scale;
    const sourceHeight = options.height / scale;
    const sourceX = (image.naturalWidth - sourceWidth) / 2;
    const sourceY = (image.naturalHeight - sourceHeight) / 2;
    ctx.save();
    ctx.globalAlpha = options.backgroundAlpha ?? 1;
    ctx.drawImage(
      image,
      sourceX,
      sourceY,
      sourceWidth,
      sourceHeight,
      0,
      0,
      options.width,
      options.height,
    );
    ctx.restore();
  }
  if (options.infoBar) {
    drawInfoBar(ctx, options.infoBar, options.width, options.height);
  }
  if (!page) return;
  const highlight = options.highlightLine ?? -1;

  page.lines.forEach((line, index) => {
    if (index === highlight) {
      ctx.fillStyle = "rgba(255, 213, 79, 0.35)";
      ctx.fillRect(line.x - 2, line.y - 2, line.width + 4, line.height);
    }
    if (line.image) {
      const image = pageImages.get(line.image.src);
      if (image && image.complete && image.naturalWidth > 0) {
        ctx.drawImage(image, line.x, line.y, line.width, line.height);
      }
      return;
    }
    ctx.fillStyle = options.textColor;
    const weight = line.light ? "300 " : line.bold ? "600 " : "";
    ctx.font = `${weight}${line.fontSize}px ${options.fontFamily}`;
    for (const char of line.chars) {
      ctx.fillText(char.ch, char.x, line.y);
    }
    if (options.underline && line.width > 0) {
      ctx.fillRect(line.x, line.y + 2, line.width, 1);
    }
  });
}

export function renderPageBitmap(
  page: LayoutPage | null,
  options: PageDrawOptions,
  dpr = window.devicePixelRatio || 1,
): HTMLCanvasElement {
  return renderPageTo(document.createElement("canvas"), page, options, dpr);
}

export function renderSpreadTo(
  canvas: HTMLCanvasElement,
  leftPage: LayoutPage | null,
  rightPage: LayoutPage | null,
  options: PageDrawOptions,
  dpr = window.devicePixelRatio || 1,
  brightness = 1,
): HTMLCanvasElement {
  const halfWidth = Math.max(1, options.width / 2);
  canvas.width = Math.max(1, Math.floor(options.width * dpr));
  canvas.height = Math.max(1, Math.floor(options.height * dpr));
  const ctx = canvas.getContext("2d");
  if (!ctx) return canvas;
  const left = renderPageTo(
    document.createElement("canvas"),
    leftPage,
    { ...options, width: halfWidth, infoBar: undefined },
    dpr,
    brightness,
  );
  const right = renderPageTo(
    document.createElement("canvas"),
    rightPage,
    { ...options, width: halfWidth, infoBar: undefined },
    dpr,
    brightness,
  );
  ctx.setTransform(1, 0, 0, 1, 0, 0);
  ctx.clearRect(0, 0, canvas.width, canvas.height);
  ctx.drawImage(left, 0, 0);
  ctx.drawImage(right, canvas.width / 2, 0);
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  if (options.infoBar) {
    drawInfoBar(ctx, options.infoBar, options.width, options.height);
  }
  return canvas;
}

export function renderPageTo(
  canvas: HTMLCanvasElement,
  page: LayoutPage | null,
  options: PageDrawOptions,
  dpr = window.devicePixelRatio || 1,
  brightness = 1,
): HTMLCanvasElement {
  canvas.width = Math.max(1, Math.floor(options.width * dpr));
  canvas.height = Math.max(1, Math.floor(options.height * dpr));
  const ctx = canvas.getContext("2d");
  if (ctx) {
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, options.width, options.height);
    if (brightness !== 1) {
      ctx.filter = `brightness(${brightness})`;
    }
    drawPage(ctx, page, options);
    ctx.filter = "none";
  }
  return canvas;
}
