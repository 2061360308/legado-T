import { invoke } from "./backend";
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
  content?: string | null;
}): Promise<Bookmark> {
  return invoke<Bookmark>(COMMANDS.bookmarkAdd, input);
}

export function bookmarkUpdate(
  time: number,
  bookText: string,
  content: string | null,
): Promise<Bookmark> {
  return invoke<Bookmark>(COMMANDS.bookmarkUpdate, { time, bookText, content });
}

export function bookmarkDelete(time: number): Promise<void> {
  return invoke<void>(COMMANDS.bookmarkDelete, { time });
}
