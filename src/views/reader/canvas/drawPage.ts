import type { LayoutPage } from "@/api/bindings";
import { drawInfoBar, type InfoBarPayload } from "../infoBar";

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
