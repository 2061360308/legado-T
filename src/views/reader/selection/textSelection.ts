import type { CharPos, LayoutPage, TextLine } from "@/core/domain";

export interface SelectionPos {
  line: number;
  char: number;
}

export interface TextSelection {
  start: SelectionPos;
  end: SelectionPos;
  text: string;
}

export interface SelectionRect {
  x: number;
  y: number;
  width: number;
  height: number;
}

export interface HitResult {
  line: number;
  char: number;
}

function isCjk(ch: string): boolean {
  return /[\u3400-\u9fff\uf900-\ufaff]/.test(ch);
}

function isWordChar(ch: string): boolean {
  return /[A-Za-z0-9_]/.test(ch);
}

function isSelectableChar(ch: string): boolean {
  return !/\s/.test(ch);
}

/** 命中某一行和行内插入位置（0..chars.length）。 */
export function hitTestText(
  page: LayoutPage,
  x: number,
  y: number,
): HitResult | null {
  const line = page.lines.find(
    (item) => y >= item.y && y <= item.y + Math.max(item.height, 1),
  );
  if (!line) return null;
  const chars = line.chars;
  if (chars.length === 0) return { line: page.lines.indexOf(line), char: 0 };

  let char = chars.length;
  for (let i = 0; i < chars.length; i++) {
    const item = chars[i];
    if (x < item.x + item.width * 0.5) {
      char = i;
      break;
    }
  }
  return { line: page.lines.indexOf(line), char };
}

function normalize(
  start: SelectionPos,
  end: SelectionPos,
): { start: SelectionPos; end: SelectionPos } {
  if (
    start.line > end.line ||
    (start.line === end.line && start.char > end.char)
  ) {
    return { start: end, end: start };
  }
  return { start, end };
}

function charAt(line: TextLine, char: number): CharPos | null {
  return line.chars[char] ?? null;
}

/** 长按选中：中文取单字，英文/数字取整词，空白不选中。 */
export function wordSelection(
  page: LayoutPage,
  lineIndex: number,
  charIndex: number,
): TextSelection | null {
  const line = page.lines[lineIndex];
  if (!line) return null;
  const chars = line.chars;
  if (chars.length === 0) return null;
  const index = Math.min(Math.max(charIndex, 0), chars.length - 1);
  const ch = chars[index].ch;
  if (!isSelectableChar(ch)) return null;

  let start = index;
  let end = index + 1;
  if (isWordChar(ch)) {
    while (start > 0 && isWordChar(chars[start - 1].ch)) start -= 1;
    while (end < chars.length && isWordChar(chars[end].ch)) end += 1;
  } else if (!isCjk(ch)) {
    while (
      start > 0 &&
      !isWordChar(chars[start - 1].ch) &&
      !isCjk(chars[start - 1].ch) &&
      isSelectableChar(chars[start - 1].ch)
    ) {
      start -= 1;
    }
    while (
      end < chars.length &&
      !isWordChar(chars[end].ch) &&
      !isCjk(chars[end].ch) &&
      isSelectableChar(chars[end].ch)
    ) {
      end += 1;
    }
  }

  return buildSelection(page, { line: lineIndex, char: start }, {
    line: lineIndex,
    char: end,
  });
}

/** 将选择范围转换为可复制的正文；跨段补换行，段内折行不补。 */
export function selectionText(
  page: LayoutPage,
  start: SelectionPos,
  end: SelectionPos,
): string {
  const range = normalize(start, end);
  let text = "";
  for (let lineIndex = range.start.line; lineIndex <= range.end.line; lineIndex++) {
    const line = page.lines[lineIndex];
    if (!line) continue;
    const from = lineIndex === range.start.line ? range.start.char : 0;
    const to = lineIndex === range.end.line ? range.end.char : line.chars.length;
    if (to <= from) continue;
    if (text && lineIndex > range.start.line) {
      const previous = page.lines[lineIndex - 1];
      if (previous && line.startChar > previous.endChar) text += "\n";
    }
    text += line.chars
      .slice(from, to)
      .map((item) => item.ch)
      .join("");
  }
  return text;
}

export function buildSelection(
  page: LayoutPage,
  start: SelectionPos,
  end: SelectionPos,
): TextSelection | null {
  const range = normalize(start, end);
  const text = selectionText(page, range.start, range.end);
  if (!text.trim()) return null;
  return { start: range.start, end: range.end, text };
}

/** 每行合并成一个高亮矩形，减少绘制次数。 */
export function selectionRects(
  page: LayoutPage,
  selection: TextSelection,
): SelectionRect[] {
  const rects: SelectionRect[] = [];
  for (
    let lineIndex = selection.start.line;
    lineIndex <= selection.end.line;
    lineIndex++
  ) {
    const line = page.lines[lineIndex];
    if (!line || line.chars.length === 0) continue;
    const from = lineIndex === selection.start.line ? selection.start.char : 0;
    const to =
      lineIndex === selection.end.line ? selection.end.char : line.chars.length;
    if (to <= from) continue;
    const first = charAt(line, from) ?? line.chars[0];
    const last = charAt(line, to - 1) ?? line.chars[line.chars.length - 1];
    rects.push({
      x: first.x,
      y: line.y,
      width: Math.max(1, last.x + last.width - first.x),
      height: Math.max(1, line.height),
    });
  }
  return rects;
}

export interface HandlePos {
  x: number;
  y: number;
}

export function selectionHandles(
  page: LayoutPage,
  selection: TextSelection,
): { start: HandlePos; end: HandlePos } | null {
  const startLine = page.lines[selection.start.line];
  const endLine = page.lines[selection.end.line];
  if (!startLine || !endLine) return null;
  const startChar =
    charAt(startLine, selection.start.char) ??
    startLine.chars[startLine.chars.length - 1];
  let endHandleLine = endLine;
  let endChar = charAt(endLine, selection.end.char - 1);
  if (!endChar && selection.end.line > selection.start.line) {
    endHandleLine = page.lines[selection.end.line - 1] ?? endLine;
    endChar = endHandleLine.chars[endHandleLine.chars.length - 1];
  }
  if (!endChar) endChar = endLine.chars[endLine.chars.length - 1];
  if (!startChar || !endChar) return null;
  return {
    start: {
      x: startChar.x,
      y: startLine.y + startLine.height,
    },
    end: {
      x: endChar.x + endChar.width,
      y: endHandleLine.y + endHandleLine.height,
    },
  };
}

export function selectionBounds(
  page: LayoutPage,
  selection: TextSelection,
): { left: number; right: number; top: number; bottom: number } | null {
  const rects = selectionRects(page, selection);
  if (rects.length === 0) return null;
  return rects.reduce(
    (box, rect) => ({
      left: Math.min(box.left, rect.x),
      right: Math.max(box.right, rect.x + rect.width),
      top: Math.min(box.top, rect.y),
      bottom: Math.max(box.bottom, rect.y + rect.height),
    }),
    {
      left: Infinity,
      right: -Infinity,
      top: Infinity,
      bottom: -Infinity,
    },
  );
}
