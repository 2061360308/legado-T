import type { LayoutPage } from "@/core/domain";

export interface LayoutOutput {
  pages: LayoutPage[];
  marginFellBack: boolean;
}

export function findPageIndex(pages: LayoutPage[], pos: number): number {
  if (!pages.length) return 0;
  for (const page of pages) {
    if (pos < page.endChar) return page.index;
  }
  return pages.length - 1;
}

export function findScrollOffset(
  page: LayoutPage,
  pos: number,
  paddingTop: number,
): number {
  for (const line of page.lines) {
    if (pos <= line.endChar) {
      return Math.max(0, line.y - paddingTop);
    }
  }
  const last = page.lines[page.lines.length - 1];
  return last ? Math.max(0, last.y + last.height - paddingTop) : 0;
}
