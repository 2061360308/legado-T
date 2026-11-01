import SparkMD5 from "spark-md5";
import { unzlibSync, unzipSync } from "fflate";
import * as pdfjs from "pdfjs-dist";
import pdfWorkerUrl from "pdfjs-dist/build/pdf.worker.mjs?url";
import type {
  ArchiveEntry,
  Book,
  BookChapter,
  LocalImportResult,
  ReaderBlock,
  TxtTocRule,
} from "@/core/domain";
import { chapterKey, db, nowMs, nowSeconds } from "./database";
import { analyzeTxtInWorker, parseUmdInWorker } from "./readerEngineClient";
import {
  WEB_ROOT,
  baseName,
  extensionOf,
  joinPath,
  readWebFile,
  webFileExists,
  writeWebFile,
} from "./storage";

pdfjs.GlobalWorkerOptions.workerSrc = pdfWorkerUrl;

const BOOK_TYPE_TEXT = 1;
const BOOK_TYPE_LOCAL = 1 << 4;
const SUPPORTED = new Set(["txt", "epub", "pdf", "umd", "mobi", "azw", "azw3"]);
const IMPORTABLE = new Set(["txt", "epub", "pdf", "umd", "mobi", "azw", "azw3"]);

interface SplitChapter {
  title: string;
  content: string;
  isVolume: boolean;
}


function decodeBytes(bytes: Uint8Array<ArrayBuffer>, charset?: string | null): {
  text: string;
  charset: string;
} {
  if (bytes.length >= 3 && bytes[0] === 0xef && bytes[1] === 0xbb && bytes[2] === 0xbf) {
    return { text: new TextDecoder("utf-8").decode(bytes.subarray(3)), charset: "UTF-8" };
  }
  if (bytes.length >= 2 && bytes[0] === 0xff && bytes[1] === 0xfe) {
    return { text: new TextDecoder("utf-16le").decode(bytes.subarray(2)), charset: "UTF-16LE" };
  }
  if (bytes.length >= 2 && bytes[0] === 0xfe && bytes[1] === 0xff) {
    return { text: new TextDecoder("utf-16be").decode(bytes.subarray(2)), charset: "UTF-16BE" };
  }
  if (charset) {
    try {
      return { text: new TextDecoder(charset).decode(bytes), charset };
    } catch {
      // fall through
    }
  }
  try {
    return {
      text: new TextDecoder("utf-8", { fatal: true }).decode(bytes),
      charset: "UTF-8",
    };
  } catch {
    try {
      return { text: new TextDecoder("gb18030").decode(bytes), charset: "GB18030" };
    } catch {
      return { text: new TextDecoder("utf-8").decode(bytes), charset: "UTF-8" };
    }
  }
}

export function decodeTxtBytes(
  bytes: Uint8Array<ArrayBuffer>,
  charset?: string | null,
): { text: string; charset: string } {
  return decodeBytes(bytes, charset);
}


function bookNameFrom(name: string): string {
  return name.replace(/\.[^.]+$/, "").trim() || name;
}

function localId(hash: string): string {
  return `local:${hash}`;
}

function md5(bytes: Uint8Array<ArrayBuffer>): string {
  return SparkMD5.ArrayBuffer.hash(bytes.buffer);
}

async function existingBook(id: string): Promise<Book | null> {
  return (await db.books.get(id)) ?? null;
}

export async function importTxtBytes(
  name: string,
  bytes: Uint8Array<ArrayBuffer>,
  group: number,
  charset?: string | null,
): Promise<LocalImportResult> {
  const hash = md5(bytes);
  const bookUrl = localId(hash);
  const existing = await existingBook(bookUrl);
  if (existing) {
    return {
      path: name,
      name,
      bookUrl,
      duplicate: true,
      error: null,
    };
  }
  const rules = await db.txtTocRules.toArray();
  let analyzed;
  try {
    analyzed = await analyzeTxtInWorker(bytes, charset ?? "", rules, true);
  } catch {
    // A bad/slow optional TOC rule must not make the whole book unimportable.
    analyzed = await analyzeTxtInWorker(bytes, charset ?? "", [], true);
  }
  if (!analyzed.chapters.length) {
    return {
      path: name,
      name,
      bookUrl: null,
      duplicate: false,
      error: "未解析到章节",
    };
  }
  const sourcePath = joinPath(WEB_ROOT, "books", bookUrl, `source.${extensionOf(name) || "txt"}`);
  await writeWebFile(sourcePath, bytes, {
    kind: "book",
    bookUrl,
    mime: "text/plain",
  });
  const book: Book = {
    bookUrl,
    name: bookNameFrom(name),
    author: "未知作者",
    origin: "local",
    originName: name,
    charset: analyzed.charset,
    type: BOOK_TYPE_TEXT | BOOK_TYPE_LOCAL,
    group,
    tocUrl: analyzed.rule,
    intro: analyzed.intro || null,
    totalChapterNum: analyzed.chapters.length,
    canUpdate: true,
    syncTime: nowSeconds(),
    sourcePath,
    sourceHash: hash,
    durChapterIndex: 0,
    durChapterPos: 0,
    durChapterTime: 0,
    order: 0,
  };
  await db.transaction("rw", db.books, db.chapters, db.chapterContents, async () => {
    await db.books.put(book);
    const decoder = new TextDecoder(analyzed.charset);
    const contents = analyzed.chapters.map((chapter) => {
      let text = decoder.decode(bytes.subarray(chapter.start, chapter.end));
      const firstLine = text.split(/\r?\n/, 1)[0]?.trim();
      if (firstLine === chapter.title.trim()) {
        text = text.slice(text.indexOf("\n") + 1);
      }
      return text.replace(/^[ \t]+/gm, "\u3000\u3000");
    });
    const chapters: BookChapter[] = analyzed.chapters.map((chapter, index) => ({
      url: `${bookUrl}#${index}`,
      title: chapter.title,
      index,
      isVolume: chapter.isVolume,
      start: chapter.start,
      end: chapter.end,
      resourceUrl: bookUrl,
    }));
    await db.chapters.bulkPut(
      chapters.map((chapter) => ({
        ...chapter,
        key: chapterKey(bookUrl, chapter.index),
        bookUrl,
      })),
    );
    await db.chapterContents.bulkPut(
      contents.map((content, index) => ({
        key: chapterKey(bookUrl, index),
        bookUrl,
        index,
        content,
      })),
    );
  });
  return {
    path: name,
    name,
    bookUrl,
    duplicate: false,
    error: null,
  };
}

function normalizeEpubPath(path: string): string {
  return path.replace(/^\/+/, "").replace(/\\/g, "/");
}

async function pdfPageText(page: pdfjs.PDFPageProxy): Promise<string> {
  const content = await page.getTextContent();
  const lines: string[] = [];
  let current = "";
  for (const item of content.items) {
    if (!("str" in item)) continue;
    current += item.str;
    if (item.hasEOL) {
      if (current.trim()) lines.push(current.trim());
      current = "";
    }
  }
  if (current.trim()) lines.push(current.trim());
  return lines.join("\n");
}

async function pdfPageImage(
  page: pdfjs.PDFPageProxy,
  scale = 1.6,
): Promise<{ blob: Blob; width: number; height: number } | null> {
  try {
    const viewport = page.getViewport({ scale });
    const canvas = document.createElement("canvas");
    canvas.width = Math.ceil(viewport.width);
    canvas.height = Math.ceil(viewport.height);
    const context = canvas.getContext("2d");
    if (!context) return null;
    await page.render({ canvas, canvasContext: context, viewport }).promise;
    const blob = await new Promise<Blob | null>((resolve) =>
      canvas.toBlob(resolve, "image/jpeg", 0.82),
    );
    if (!blob) return null;
    return { blob, width: canvas.width, height: canvas.height };
  } catch {
    return null;
  }
}

async function pdfCover(page: pdfjs.PDFPageProxy): Promise<Blob | null> {
  return (await pdfPageImage(page, 1.4))?.blob ?? null;
}

export async function importPdfBytes(
  name: string,
  bytes: Uint8Array<ArrayBuffer>,
  group: number,
): Promise<LocalImportResult> {
  const hash = md5(bytes);
  const bookUrl = localId(hash);
  if (await existingBook(bookUrl)) {
    return { path: name, name, bookUrl, duplicate: true, error: null };
  }
  const loadingTask = pdfjs.getDocument({ data: bytes });
  const document = await loadingTask.promise;
  try {
    const chapters: SplitChapter[] = [];
    const chapterBlocks: ReaderBlock[][] = [];
    const pagesPerChapter = 5;
    for (let start = 1; start <= document.numPages; start += pagesPerChapter) {
      const end = Math.min(document.numPages, start + pagesPerChapter - 1);
      const parts: string[] = [];
      const blocks: ReaderBlock[] = [];
      for (let pageNumber = start; pageNumber <= end; pageNumber += 1) {
        const page = await document.getPage(pageNumber);
        const text = await pdfPageText(page);
        const textLength = text.replace(/\s/g, "").length;
        if (textLength < 20) {
          const image = await pdfPageImage(page);
          if (image) {
            const target = joinPath(
              WEB_ROOT,
              "books",
              bookUrl,
              "pdf",
              `page-${pageNumber}.jpg`,
            );
            await writeWebFile(target, image.blob, {
              kind: "image",
              bookUrl,
              mime: "image/jpeg",
            });
            blocks.push({
              type: "image",
              src: target,
              width: image.width,
              height: image.height,
              fullPage: true,
            });
          }
          continue;
        }
        parts.push(text);
        for (const line of text.split("\n")) {
          const value = line.trim();
          if (value) blocks.push({ type: "paragraph", text: value });
        }
      }
      if (!blocks.length) continue;
      chapters.push({
        title:
          start === end ? `第 ${start} 页` : `第 ${start}-${end} 页`,
        content: parts.join("\n\n"),
        isVolume: false,
      });
      chapterBlocks.push(blocks);
    }
    if (!chapters.length) {
      return {
        path: name,
        name,
        bookUrl: null,
        duplicate: false,
        error: "PDF 没有可读取页面",
      };
    }
    const info = (await document.getMetadata().catch(() => null))?.info as
      | { Title?: string; Author?: string }
      | undefined;
    const sourcePath = joinPath(WEB_ROOT, "books", bookUrl, "source.pdf");
    await writeWebFile(sourcePath, bytes, {
      kind: "book",
      bookUrl,
      mime: "application/pdf",
    });
    let coverUrl: string | null = null;
    try {
      const first = await document.getPage(1);
      const cover = await pdfCover(first);
      if (cover) {
        const target = joinPath(WEB_ROOT, "books", bookUrl, "cover.jpg");
        await writeWebFile(target, cover, {
          kind: "cover",
          bookUrl,
          mime: "image/jpeg",
        });
        coverUrl = target;
      }
    } catch {
      // ignore cover failures
    }
    const book: Book = {
      bookUrl,
      name: info?.Title?.trim() || bookNameFrom(name),
      author: info?.Author?.trim() || "未知作者",
      origin: "local",
      originName: name,
      charset: "UTF-8",
      type: BOOK_TYPE_TEXT | BOOK_TYPE_LOCAL,
      group,
      kind: "pdf",
      intro: chapters[0].content.slice(0, 500) || "PDF 文档",
      coverUrl,
      totalChapterNum: chapters.length,
      canUpdate: false,
      syncTime: nowSeconds(),
      sourcePath,
      sourceHash: hash,
      durChapterIndex: 0,
      durChapterPos: 0,
      durChapterTime: 0,
      order: 0,
    };
    await db.transaction(
      "rw",
      db.books,
      db.chapters,
      db.chapterContents,
      db.chapterBlocks,
      async () => {
        await db.books.put(book);
        const bookChapters: BookChapter[] = chapters.map((chapter, index) => ({
          url: `${bookUrl}#${index}`,
          title: chapter.title,
          index,
          isVolume: false,
          start: 0,
          end: 0,
          resourceUrl: bookUrl,
        }));
        await db.chapters.bulkPut(
          bookChapters.map((chapter) => ({
            ...chapter,
            key: chapterKey(bookUrl, chapter.index),
            bookUrl,
          })),
        );
        await db.chapterContents.bulkPut(
          chapters.map((chapter, index) => ({
            key: chapterKey(bookUrl, index),
            bookUrl,
            index,
            content: chapter.content,
          })),
        );
        await db.chapterBlocks.bulkPut(
          chapterBlocks.map((blocks, index) => ({
            key: chapterKey(bookUrl, index),
            bookUrl,
            index,
            blocks,
          })),
        );
      },
    );
    return { path: name, name, bookUrl, duplicate: false, error: null };
  } finally {
    await loadingTask.destroy();
  }
}

async function importUmdBytes(
  name: string,
  bytes: Uint8Array<ArrayBuffer>,
  group: number,
): Promise<LocalImportResult> {
  const hash = md5(bytes);
  const bookUrl = localId(hash);
  if (await existingBook(bookUrl)) {
    return { path: name, name, bookUrl, duplicate: true, error: null };
  }
  const parsed = await parseUmdInWorker(bytes, bookNameFrom(name));
  const chapters = parsed.chapters
    .map((chapter) => ({
      title: chapter.title?.trim() || `第${parsed.chapters.indexOf(chapter) + 1}章`,
      content: chapter.text.trim(),
    }))
    .filter((chapter) => chapter.content);
  if (!chapters.length) {
    return { path: name, name, bookUrl: null, duplicate: false, error: "UMD 没有可读章节" };
  }
  const sourcePath = joinPath(WEB_ROOT, "books", bookUrl, "source.umd");
  await writeWebFile(sourcePath, bytes, { kind: "book", bookUrl, mime: "application/octet-stream" });
  let coverUrl: string | null = null;
  if (parsed.coverBase64) {
    const coverExt = parsed.coverExt || "jpg";
    const target = joinPath(WEB_ROOT, "books", bookUrl, `cover.${coverExt}`);
    await writeWebFile(target, base64Bytes(parsed.coverBase64), {
      kind: "cover",
      bookUrl,
      mime: `image/${coverExt}`,
    });
    coverUrl = target;
  }
  const book: Book = {
    bookUrl,
    name: parsed.title?.trim() || bookNameFrom(name),
    author: parsed.author?.trim() || "未知作者",
    origin: "local",
    originName: name,
    charset: "UTF-16LE",
    type: BOOK_TYPE_TEXT | BOOK_TYPE_LOCAL,
    group,
    kind: "umd",
    intro: chapters[0].content.slice(0, 500),
    coverUrl,
    totalChapterNum: chapters.length,
    canUpdate: false,
    syncTime: nowSeconds(),
    sourcePath,
    sourceHash: hash,
    durChapterIndex: 0,
    durChapterPos: 0,
    durChapterTime: 0,
    order: 0,
  };
  await db.transaction("rw", db.books, db.chapters, db.chapterContents, async () => {
    await db.books.put(book);
    const bookChapters: BookChapter[] = chapters.map((chapter, index) => ({
      url: `${bookUrl}#${index}`,
      title: chapter.title,
      index,
      isVolume: false,
      start: 0,
      end: 0,
      resourceUrl: bookUrl,
    }));
    await db.chapters.bulkPut(
      bookChapters.map((chapter) => ({
        ...chapter,
        key: chapterKey(bookUrl, chapter.index),
        bookUrl,
      })),
    );
    await db.chapterContents.bulkPut(
      chapters.map((chapter, index) => ({
        key: chapterKey(bookUrl, index),
        bookUrl,
        index,
        content: chapter.content,
      })),
    );
  });
  return { path: name, name, bookUrl, duplicate: false, error: null };
}

export interface CoverBytes {
  bytes: Uint8Array<ArrayBuffer>;
  ext: string;
}

export async function extractUmdCoverBytes(
  bytes: Uint8Array<ArrayBuffer>,
): Promise<CoverBytes | null> {
  const parsed = await parseUmdInWorker(bytes, "");
  if (!parsed.coverBase64) return null;
  return {
    bytes: base64Bytes(parsed.coverBase64),
    ext: parsed.coverExt || "jpg",
  };
}

export async function extractPdfCoverBytes(
  bytes: Uint8Array<ArrayBuffer>,
): Promise<CoverBytes | null> {
  const loadingTask = pdfjs.getDocument({ data: bytes });
  const document = await loadingTask.promise;
  try {
    const page = await document.getPage(1);
    const image = await pdfPageImage(page, 1.4);
    if (!image) return null;
    return {
      bytes: new Uint8Array(await image.blob.arrayBuffer()),
      ext: "jpg",
    };
  } finally {
    await loadingTask.destroy();
  }
}

type EbookWasmModule = typeof import("./wasm/ebook/ebook_wasm.js");

let ebookWasmPromise: Promise<EbookWasmModule> | null = null;

async function loadEbookWasm(): Promise<EbookWasmModule> {
  if (!ebookWasmPromise) {
    ebookWasmPromise = import("./wasm/ebook/ebook_wasm.js").then(async (module) => {
      await module.default();
      return module;
    });
  }
  return ebookWasmPromise;
}

function base64Bytes(value: string): Uint8Array<ArrayBuffer> {
  const binary = atob(value);
  const out = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    out[index] = binary.charCodeAt(index);
  }
  return out;
}

export async function extractEbookCoverBytes(
  bytes: Uint8Array<ArrayBuffer>,
  titleFallback: string,
): Promise<CoverBytes | null> {
  const wasm = await loadEbookWasm();
  const parsed = JSON.parse(wasm.parse_ebook_json(bytes, titleFallback)) as {
    cover_base64?: string | null;
    cover_ext?: string | null;
  };
  if (!parsed.cover_base64) return null;
  return {
    bytes: base64Bytes(parsed.cover_base64),
    ext: parsed.cover_ext || "jpg",
  };
}

type RbookWasmModule = typeof import("./wasm/rbook/rbook_wasm.js");

let rbookWasmPromise: Promise<RbookWasmModule> | null = null;

async function loadRbookWasm(): Promise<RbookWasmModule> {
  if (!rbookWasmPromise) {
    rbookWasmPromise = import("./wasm/rbook/rbook_wasm.js").then(async (module) => {
      await module.default();
      return module;
    });
  }
  return rbookWasmPromise;
}

type ParsedBlock =
  | { type: "paragraph"; text: string }
  | {
      type: "image";
      base64: string;
      ext?: string;
      width?: number;
      height?: number;
      alt?: string | null;
    };

interface ParsedSection {
  index: number;
  title: string;
  text: string;
  blocks?: ParsedBlock[];
}

interface ParsedEbook {
  title: string;
  author: string;
  intro: string;
  cover_base64?: string | null;
  cover_ext?: string | null;
  sections: ParsedSection[];
}

async function importParsedEbook(
  name: string,
  bytes: Uint8Array<ArrayBuffer>,
  group: number,
  ext: string,
  parsed: ParsedEbook,
  emptyMessage: string,
): Promise<LocalImportResult> {
  const hash = md5(bytes);
  const bookUrl = localId(hash);
  if (await existingBook(bookUrl)) {
    return { path: name, name, bookUrl, duplicate: true, error: null };
  }
  const writtenImages = new Set<string>();
  const chapters: {
    title: string;
    content: string;
    isVolume: boolean;
    blocks?: ReaderBlock[];
  }[] = [];
  for (const section of parsed.sections) {
    const blocks: ReaderBlock[] = [];
    for (const block of section.blocks ?? []) {
      if (block.type === "paragraph") {
        blocks.push({ type: "paragraph", text: block.text });
        continue;
      }
      const data = base64Bytes(block.base64);
      const rawExt = (block.ext || "jpg").replace(/[^a-z0-9]/gi, "").toLowerCase();
      const ext = rawExt || "jpg";
      const target = joinPath(
        WEB_ROOT,
        "books",
        bookUrl,
        "images",
        `${md5(data)}.${ext}`,
      );
      if (!writtenImages.has(target)) {
        await writeWebFile(target, data, {
          kind: "image",
          bookUrl,
          mime: `image/${ext}`,
        });
        writtenImages.add(target);
      }
      blocks.push({
        type: "image",
        src: target,
        width: block.width ?? 0,
        height: block.height ?? 0,
      });
    }
    const text =
      blocks
        .filter((block): block is { type: "paragraph"; text: string } => block.type === "paragraph")
        .map((block) => block.text)
        .join("\n")
        .trim() || section.text.trim();
    if (!text && !blocks.length) continue;
    chapters.push({
      title: section.title || `第${section.index + 1}章`,
      content: text,
      isVolume: false,
      blocks: blocks.length ? blocks : undefined,
    });
  }
  if (!chapters.length) {
    return { path: name, name, bookUrl: null, duplicate: false, error: emptyMessage };
  }
  const sourcePath = joinPath(WEB_ROOT, "books", bookUrl, `source.${ext}`);
  await writeWebFile(sourcePath, bytes, {
    kind: "book",
    bookUrl,
    mime: "application/octet-stream",
  });
  let coverUrl: string | null = null;
  if (parsed.cover_base64) {
    const coverExt = parsed.cover_ext || "jpg";
    const target = joinPath(WEB_ROOT, "books", bookUrl, `cover.${coverExt}`);
    await writeWebFile(target, base64Bytes(parsed.cover_base64), {
      kind: "cover",
      bookUrl,
      mime: `image/${coverExt}`,
    });
    coverUrl = target;
  }
  const book: Book = {
    bookUrl,
    name: parsed.title?.trim() || bookNameFrom(name),
    author: parsed.author?.trim() || "未知作者",
    origin: "local",
    originName: name,
    charset: "UTF-8",
    type: BOOK_TYPE_TEXT | BOOK_TYPE_LOCAL,
    group,
    kind: ext,
    intro: parsed.intro?.trim() || chapters[0].content.slice(0, 500),
    coverUrl,
    totalChapterNum: chapters.length,
    canUpdate: false,
    syncTime: nowSeconds(),
    sourcePath,
    sourceHash: hash,
    durChapterIndex: 0,
    durChapterPos: 0,
    durChapterTime: 0,
    order: 0,
  };
  await db.transaction(
    "rw",
    db.books,
    db.chapters,
    db.chapterContents,
    db.chapterBlocks,
    async () => {
    await db.books.put(book);
    const bookChapters: BookChapter[] = chapters.map((chapter, index) => ({
      url: `${bookUrl}#${index}`,
      title: chapter.title,
      index,
      isVolume: false,
      start: 0,
      end: 0,
      resourceUrl: bookUrl,
    }));
    await db.chapters.bulkPut(
      bookChapters.map((chapter) => ({
        ...chapter,
        key: chapterKey(bookUrl, chapter.index),
        bookUrl,
      })),
    );
    await db.chapterContents.bulkPut(
      chapters.map((chapter, index) => ({
        key: chapterKey(bookUrl, index),
        bookUrl,
        index,
        content: chapter.content,
      })),
    );
    await db.chapterBlocks.bulkPut(
      chapters
        .map((chapter, index) => ({
          key: chapterKey(bookUrl, index),
          bookUrl,
          index,
          blocks: chapter.blocks ?? [],
        }))
        .filter((row) => row.blocks.length > 0),
    );
  });
  return { path: name, name, bookUrl, duplicate: false, error: null };
}

export async function extractEpubCoverBytes(
  bytes: Uint8Array<ArrayBuffer>,
  titleFallback: string,
): Promise<CoverBytes | null> {
  const wasm = await loadRbookWasm();
  const parsed = JSON.parse(wasm.parse_epub_json(bytes, titleFallback)) as {
    cover_base64?: string | null;
    cover_ext?: string | null;
  };
  if (!parsed.cover_base64) return null;
  return {
    bytes: base64Bytes(parsed.cover_base64),
    ext: parsed.cover_ext || "jpg",
  };
}

export async function importEpubBytes(
  name: string,
  bytes: Uint8Array<ArrayBuffer>,
  group: number,
): Promise<LocalImportResult> {
  const wasm = await loadRbookWasm();
  const parsed = JSON.parse(
    wasm.parse_epub_json(bytes, bookNameFrom(name)),
  ) as ParsedEbook;
  return importParsedEbook(name, bytes, group, "epub", parsed, "EPUB 没有可读章节");
}

export async function importEbookBytes(
  name: string,
  bytes: Uint8Array<ArrayBuffer>,
  group: number,
  ext: string,
): Promise<LocalImportResult> {
  const wasm = await loadEbookWasm();
  const parsed = JSON.parse(
    wasm.parse_ebook_json(bytes, bookNameFrom(name)),
  ) as ParsedEbook;
  return importParsedEbook(name, bytes, group, ext, parsed, "MOBI/AZW3 没有可读章节");
}

export async function importWebPaths(
  paths: string[],
  group: number,
  charset?: string | null,
): Promise<LocalImportResult[]> {
  const results: LocalImportResult[] = [];
  for (const path of paths) {
    const name = baseName(path);
    const ext = extensionOf(path);
    if (!IMPORTABLE.has(ext)) {
      results.push({
        path,
        name,
        bookUrl: null,
        duplicate: false,
        error: `Web 端暂不支持 ${ext || "该"} 格式`,
      });
      continue;
    }
    try {
      const bytes = await readWebFile(path);
      results.push(
        ext === "epub"
          ? await importEpubBytes(name, bytes, group)
          : ext === "pdf"
            ? await importPdfBytes(name, bytes, group)
            : ext === "umd"
              ? await importUmdBytes(name, bytes, group)
              : ext === "mobi" || ext === "azw" || ext === "azw3"
                ? await importEbookBytes(name, bytes, group, ext)
                : await importTxtBytes(name, bytes, group, charset),
      );
    } catch (error) {
      results.push({
        path,
        name,
        bookUrl: null,
        duplicate: false,
        error: String(error),
      });
    }
  }
  return results;
}

export async function archiveList(path: string): Promise<ArchiveEntry[]> {
  const bytes = await readWebFile(path);
  const entries = unzipSync(bytes);
  return Object.entries(entries)
    .filter(([name]) => !name.endsWith("/") && !name.endsWith("\\"))
    .map(([name, data]) => {
      const normalized = normalizeEpubPath(name);
      const ext = extensionOf(normalized);
      return {
        name: normalized,
        size: data.length,
        supported: SUPPORTED.has(ext),
      };
    })
    .sort((left, right) => left.name.localeCompare(right.name, "zh-CN"));
}

export async function importWebArchive(
  path: string,
  names: string[],
  group: number,
  charset?: string | null,
): Promise<LocalImportResult[]> {
  const bytes = await readWebFile(path);
  const rawEntries = unzipSync(bytes);
  const entries: Record<string, Uint8Array> = {};
  for (const [key, value] of Object.entries(rawEntries)) {
    entries[normalizeEpubPath(key)] = value;
  }
  const wanted = new Set(names.map(normalizeEpubPath));
  const results: LocalImportResult[] = [];
  for (const [name, data] of Object.entries(entries)) {
    if (!wanted.has(name)) continue;
    const ext = extensionOf(name);
    if (!IMPORTABLE.has(ext)) {
      results.push({
        path: name,
        name,
        bookUrl: null,
        duplicate: false,
        error: `Web 端暂不支持 ${ext || "该"} 格式`,
      });
      continue;
    }
    try {
      const dataBytes = new Uint8Array(data);
      results.push(
        ext === "epub"
          ? await importEpubBytes(baseName(name), dataBytes, group)
          : ext === "pdf"
            ? await importPdfBytes(baseName(name), dataBytes, group)
            : ext === "umd"
              ? await importUmdBytes(baseName(name), dataBytes, group)
              : ext === "mobi" || ext === "azw" || ext === "azw3"
                ? await importEbookBytes(baseName(name), dataBytes, group, ext)
                : await importTxtBytes(baseName(name), dataBytes, group, charset),
      );
    } catch (error) {
      results.push({
        path: name,
        name,
        bookUrl: null,
        duplicate: false,
        error: String(error),
      });
    }
  }
  return results;
}

export async function deleteImportPaths(paths: string[]): Promise<number> {
  let count = 0;
  for (const path of paths) {
    if (await webFileExists(path)) {
      await db.files.delete(path);
      count += 1;
    }
  }
  return count;
}
