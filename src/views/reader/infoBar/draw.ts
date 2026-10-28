import { renderSlot } from "./format";
import {
  BATTERY_ICON_GAP,
  BATTERY_ICON_TERMINAL,
  BATTERY_ICON_WIDTH,
  INFO_BATTERY_FONT_SIZE,
  INFO_FONT_SIZE,
  SLOT_PAD_LEFT,
  SLOT_PAD_RIGHT,
  SLOT_PAD_TOP,
  measureInfoBarHeights,
} from "./style";
import type {
  InfoBarContext,
  InfoBarPadding,
  InfoBarPayload,
  InfoBarStyle,
  SlotRender,
  TipKey,
} from "./types";

function fontOf(style: InfoBarStyle, slot: SlotRender): string {
  const size = slot.showBatteryIcon ? INFO_BATTERY_FONT_SIZE : INFO_FONT_SIZE;
  return `${size}px ${style.fontFamily}`;
}

function measureWidth(ctx: CanvasRenderingContext2D, text: string, font: string): number {
  ctx.font = font;
  return ctx.measureText(text).width;
}

function fitText(
  ctx: CanvasRenderingContext2D,
  text: string,
  font: string,
  maxWidth: number,
): string {
  if (maxWidth <= 0) return "";
  ctx.font = font;
  if (ctx.measureText(text).width <= maxWidth) return text;
  const ellipsis = "…";
  let low = 0;
  let high = text.length;
  while (low < high) {
    const mid = Math.ceil((low + high) / 2);
    const candidate = `${text.slice(0, mid)}${ellipsis}`;
    if (ctx.measureText(candidate).width <= maxWidth) low = mid;
    else high = mid - 1;
  }
  return low > 0 ? `${text.slice(0, low)}${ellipsis}` : "";
}

function iconExtra(slot: SlotRender): number {
  return slot.showBatteryIcon ? BATTERY_ICON_GAP + BATTERY_ICON_WIDTH : 0;
}

function totalWidth(ctx: CanvasRenderingContext2D, style: InfoBarStyle, slot: SlotRender): number {
  return measureWidth(ctx, slot.text, fontOf(style, slot)) + iconExtra(slot);
}

function drawBatteryIcon(
  ctx: CanvasRenderingContext2D,
  style: InfoBarStyle,
  x: number,
  top: number,
  lineHeight: number,
): void {
  const height = Math.max(6, Math.min(10, lineHeight - 4));
  const y = top + (lineHeight - height) / 2;
  ctx.save();
  ctx.strokeStyle = style.textColor;
  ctx.fillStyle = style.textColor;
  ctx.lineWidth = 1;
  ctx.strokeRect(x + 0.5, y + 0.5, BATTERY_ICON_WIDTH - 1, height - 1);
  ctx.fillRect(x + BATTERY_ICON_WIDTH, y + height / 2 - 2, BATTERY_ICON_TERMINAL, 4);
  ctx.restore();
}

function drawSlot(
  ctx: CanvasRenderingContext2D,
  style: InfoBarStyle,
  slot: SlotRender,
  x: number,
  top: number,
  lineHeight: number,
): void {
  if (!slot.text) return;
  const font = fontOf(style, slot);
  ctx.font = font;
  ctx.fillStyle = style.textColor;
  ctx.textBaseline = "top";
  ctx.textAlign = "left";
  ctx.fillText(slot.text, x, top);
  if (slot.showBatteryIcon) {
    const textWidth = measureWidth(ctx, slot.text, font);
    drawBatteryIcon(ctx, style, x + textWidth + BATTERY_ICON_GAP, top, lineHeight);
  }
}

function drawBar(
  ctx: CanvasRenderingContext2D,
  style: InfoBarStyle,
  context: InfoBarContext,
  slots: [TipKey, TipKey, TipKey],
  padding: InfoBarPadding,
  width: number,
  barTop: number,
  lineHeight: number,
): void {
  const left = renderSlot(slots[0], context);
  const middle = renderSlot(slots[1], context);
  const right = renderSlot(slots[2], context);
  const textTop = barTop + padding.top + SLOT_PAD_TOP;
  const innerLeft = padding.left + SLOT_PAD_LEFT;
  const innerRight = width - padding.right - SLOT_PAD_RIGHT;

  const rightWidth = right.text ? totalWidth(ctx, style, right) : 0;
  const rightX = innerRight - rightWidth;
  const middleWidth = middle.text ? totalWidth(ctx, style, middle) : 0;

  if (right.text) {
    const maxText = Math.max(0, width - padding.left - padding.right - iconExtra(right));
    right.text = fitText(ctx, right.text, fontOf(style, right), maxText);
    const widthWithIcon = totalWidth(ctx, style, right);
    drawSlot(ctx, style, right, innerRight - widthWithIcon, textTop, lineHeight);
  }

  if (left.text) {
    let maxWidth = (right.text ? rightX - 8 : innerRight) - innerLeft;
    if (middle.text) {
      maxWidth = Math.min(maxWidth, width / 2 - middleWidth / 2 - 8 - innerLeft);
    }
    const maxText = Math.max(0, maxWidth - iconExtra(left));
    left.text = fitText(ctx, left.text, fontOf(style, left), maxText);
    drawSlot(ctx, style, left, innerLeft, textTop, lineHeight);
  }

  if (middle.text) {
    const maxText = Math.max(
      0,
      width - padding.left - padding.right - iconExtra(middle),
    );
    middle.text = fitText(ctx, middle.text, fontOf(style, middle), maxText);
    const widthWithIcon = totalWidth(ctx, style, middle);
    drawSlot(ctx, style, middle, width / 2 - widthWithIcon / 2, textTop, lineHeight);
  }
}

export function drawInfoBar(
  ctx: CanvasRenderingContext2D,
  payload: InfoBarPayload,
  width: number,
  height: number,
): void {
  const { style, context } = payload;
  const heights = measureInfoBarHeights(style);
  if (heights.header > 0) {
    if (style.showHeaderLine) {
      ctx.fillStyle = style.dividerColor;
      ctx.fillRect(0, heights.header - 0.5, width, 0.5);
    }
    drawBar(
      ctx,
      style,
      context,
      style.headerSlots,
      style.headerPadding,
      width,
      0,
      heights.line,
    );
  }
  if (heights.footer > 0) {
    const footerTop = height - heights.footer;
    if (style.showFooterLine) {
      ctx.fillStyle = style.dividerColor;
      ctx.fillRect(0, footerTop, width, 0.5);
    }
    drawBar(
      ctx,
      style,
      context,
      style.footerSlots,
      style.footerPadding,
      width,
      footerTop,
      heights.line,
    );
  }
}
