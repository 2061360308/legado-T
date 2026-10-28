import { invoke } from "@tauri-apps/api/core";
import { COMMANDS, type Bookmark } from "./bindings";

export function bookmarkList(): Promise<Bookmark[]> {
  return invoke<Bookmark[]>(COMMANDS.bookmarkList);
}

export function bookmarkAdd(input: {
  bookUrl: string;
  bookName: string;
  bookAuthor: string;
  chapterIndex: number;
  chapterPos: number;
  chapterName: string;
  bookText: string;
}): Promise<Bookmark> {
  return invoke<Bookmark>(COMMANDS.bookmarkAdd, input);
}

export function bookmarkDelete(time: number): Promise<void> {
  return invoke<void>(COMMANDS.bookmarkDelete, { time });
}
