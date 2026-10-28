import type { InfoBarPadding, InfoBarStyle, TipKey } from "./types";

export const INFO_FONT_SIZE = 12;
export const INFO_BATTERY_FONT_SIZE = 11;
export const SLOT_PAD_TOP = 3;
export const SLOT_PAD_BOTTOM = 3;
export const SLOT_PAD_LEFT = 4;
export const SLOT_PAD_RIGHT = 6;
export const BATTERY_ICON_WIDTH = 8;
export const BATTERY_ICON_TERMINAL = 2;
export const BATTERY_ICON_GAP = 3;

export interface InfoBarPrefsLike {
  headerMode: number;
  footerMode: number;
  tipHeaderLeft: string;
  tipHeaderMiddle: string;
  tipHeaderRight: string;
  tipFooterLeft: string;
  tipFooterMiddle: string;
  tipFooterRight: string;
  tipColor: string;
  tipDividerColor: string;
  fontFamily: string;
  headerPaddingLeft: number;
  headerPaddingTop: number;
  headerPaddingRight: number;
  headerPaddingBottom: number;
  footerPaddingLeft: number;
  footerPaddingTop: number;
  footerPaddingRight: number;
  footerPaddingBottom: number;
  showHeaderLine: boolean;
  showFooterLine: boolean;
}

let measureContext: CanvasRenderingContext2D | null = null;

function measureCtx(): CanvasRenderingContext2D | null {
  if (measureContext) return measureContext;
  const canvas = document.createElement("canvas");
  measureContext = canvas.getContext("2d");
  return measureContext;
}

export function measureLineHeight(fontFamily: string): number {
  const ctx = measureCtx();
  if (!ctx) return 16;
  ctx.font = `${INFO_FONT_SIZE}px ${fontFamily}`;
  const metrics = ctx.measureText("国Ag");
  const height = metrics.actualBoundingBoxAscent + metrics.actualBoundingBoxDescent;
  return Math.ceil(Math.max(Number.isFinite(height) && height > 0 ? height : 0, INFO_FONT_SIZE * 1.3));
}

export function hasSlots(slots: [TipKey, TipKey, TipKey]): boolean {
  return slots.some((slot) => slot !== "none");
}

function paddingOf(
  left: number,
  top: number,
  right: number,
  bottom: number,
): InfoBarPadding {
  return { left, top, right, bottom };
}

export function buildInfoBarStyle(
  prefs: InfoBarPrefsLike,
  options: { textColor: string; dividerColor: string },
): InfoBarStyle {
  return {
    headerVisible: prefs.headerMode !== 2,
    footerVisible: prefs.footerMode === 0,
    headerSlots: [prefs.tipHeaderLeft, prefs.tipHeaderMiddle, prefs.tipHeaderRight] as [TipKey, TipKey, TipKey],
    footerSlots: [prefs.tipFooterLeft, prefs.tipFooterMiddle, prefs.tipFooterRight] as [TipKey, TipKey, TipKey],
    headerPadding: paddingOf(
      prefs.headerPaddingLeft,
      prefs.headerPaddingTop,
      prefs.headerPaddingRight,
      prefs.headerPaddingBottom,
    ),
    footerPadding: paddingOf(
      prefs.footerPaddingLeft,
      prefs.footerPaddingTop,
      prefs.footerPaddingRight,
      prefs.footerPaddingBottom,
    ),
    showHeaderLine: prefs.showHeaderLine,
    showFooterLine: prefs.showFooterLine,
    textColor: prefs.tipColor || options.textColor,
    dividerColor:
      prefs.tipDividerColor === "text"
        ? options.textColor
        : prefs.tipDividerColor || options.dividerColor,
    fontFamily: prefs.fontFamily,
  };
}

export interface InfoBarHeights {
  line: number;
  content: number;
  header: number;
  footer: number;
}

export function measureInfoBarHeights(style: InfoBarStyle): InfoBarHeights {
  const line = measureLineHeight(style.fontFamily);
  const content = SLOT_PAD_TOP + line + SLOT_PAD_BOTTOM;
  const header =
    style.headerVisible && hasSlots(style.headerSlots)
      ? style.headerPadding.top +
        style.headerPadding.bottom +
        content +
        (style.showHeaderLine ? 0.5 : 0)
      : 0;
  const footer =
    style.footerVisible && hasSlots(style.footerSlots)
      ? style.footerPadding.top +
        style.footerPadding.bottom +
        content +
        (style.showFooterLine ? 0.5 : 0)
      : 0;
  return { line, content, header, footer };
}
