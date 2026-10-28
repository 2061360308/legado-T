import { invoke } from "@tauri-apps/api/core";
import {
  COMMANDS,
  type LayoutStyle,
  type ReaderSnapshot,
  type ReplaceRule,
} from "./bindings";

export function readerOpen(
  bookUrl: string,
  chapterIndex: number | null,
  chapterPos: number | null,
  style: LayoutStyle,
): Promise<ReaderSnapshot> {
  return invoke<ReaderSnapshot>(COMMANDS.readerOpen, {
    bookUrl,
    chapterIndex,
    chapterPos,
    style,
  });
}

export function readerSaveProgress(
  bookUrl: string,
  chapterIndex: number,
  chapterPos: number,
  chapterTitle: string,
): Promise<void> {
  return invoke<void>(COMMANDS.readerSaveProgress, {
    bookUrl,
    chapterIndex,
    chapterPos,
    chapterTitle,
  });
}

export function readerSaveChapterContent(
  bookUrl: string,
  chapterIndex: number,
  content: string,
): Promise<void> {
  return invoke<void>(COMMANDS.readerSaveChapterContent, {
    bookUrl,
    chapterIndex,
    content,
  });
}

export function readerEffectiveRules(
  bookUrl: string,
  chapterIndex: number,
  style: LayoutStyle,
): Promise<ReplaceRule[]> {
  return invoke<ReplaceRule[]>(COMMANDS.readerEffectiveRules, {
    bookUrl,
    chapterIndex,
    style,
  });
}
