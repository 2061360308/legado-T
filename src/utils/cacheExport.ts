import { strToU8, zipSync } from "fflate";
import { chapterList, readChapter } from "@/core/application/books";
import { replaceRules } from "@/core/application/rules";
import type { Book, ReplaceRule } from "@/core/domain";

export interface CacheExportOptions {
  useReplace: boolean;
  noChapterName: boolean;
  exportToWebdav: boolean;
  exportPictures: boolean;
  parallel: boolean;
  type: "txt" | "epub";
  charset: string;
  filenameTemplate: string;
  scope: string;
}

export interface BookChapterContent {
  title: string;
  content: string;
}

export const DEFAULT_CACHE_EXPORT_OPTIONS: CacheExportOptions = {
  useReplace: false,
  noChapterName: false,
  exportToWebdav: false,
  exportPictures: false,
  parallel: false,
  type: "txt",
  charset: "UTF-8",
  filenameTemplate: "{{name}}",
  scope: "",
};

export function sanitizeFileName(value: string): string {
  return value.replace(/[\\/:*?"<>|]/g, "_").trim() || "book";
}

export function formatFileName(template: string, book: Book): string {
  const name = book.name || "book";
  const author = book.author || "";
  return sanitizeFileName(
    (template || "{{name}}")
      .replace(/\{\{\s*name\s*\}\}/gi, name)
      .replace(/\{\{\s*author\s*\}\}/gi, author)
      .replace(/\$\{\s*name\s*\}/gi, name)
      .replace(/\$\{\s*author\s*\}/gi, author),
  );
}

export function parseChapterScope(scope: string, total: number): Set<number> {
  const value = scope.trim();
  if (!value) {
    return new Set(Array.from({ length: total }, (_, index) => index + 1));
  }
  const result = new Set<number>();
  for (const part of value.split(/[,，]/)) {
    const item = part.trim();
    if (!item) continue;
    const range = item.match(/^(\d+)\s*-\s*(\d+)$/);
    if (range) {
      const start = Math.max(1, Number(range[1]));
      const end = Math.min(total, Number(range[2]));
      for (let index = start; index <= end; index++) result.add(index);
    } else if (/^\d+$/.test(item)) {
      const index = Number(item);
      if (index >= 1 && index <= total) result.add(index);
    }
  }
  if (!result.size) {
    throw new Error("章节范围格式无效，示例：1-5,8,10-18");
  }
  return result;
}

function applyRules(text: string, rules: ReplaceRule[]): string {
  let value = text;
  for (const rule of rules) {
    if (rule.isEnabled === false || rule.scopeContent === false) continue;
    if (!rule.pattern) continue;
    const replacement = rule.replacement ?? "";
    try {
      if (rule.isRegex === false) {
        value = value.split(rule.pattern).join(replacement);
      } else {
        value = value.replace(new RegExp(rule.pattern, "g"), replacement);
      }
    } catch {
      // Ignore invalid replace rules.
    }
  }
  return value;
}

export async function loadCachedBook(
  book: Book,
  options: CacheExportOptions,
): Promise<BookChapterContent[]> {
  const chapters = (await chapterList(book.bookUrl)).filter((chapter) => !chapter.isVolume);
  const selected = parseChapterScope(options.scope, chapters.length);
  const rules = options.useReplace ? await replaceRules() : [];
  const result: BookChapterContent[] = [];
  for (const chapter of chapters) {
    if (!selected.has(chapter.index + 1)) continue;
    try {
      let content = await readChapter(book.bookUrl, chapter.index);
      if (!content.trim()) continue;
      if (options.useReplace) content = applyRules(content, rules);
      result.push({ title: chapter.title, content });
    } catch {
      // Skip chapters that are not cached.
    }
  }
  return result;
}

export function buildText(book: Book, chapters: BookChapterContent[], noChapterName: boolean): string {
  const parts = [`${book.name}${book.author ? ` / ${book.author}` : ""}`];
  for (const chapter of chapters) {
    parts.push(noChapterName ? chapter.content : `\n${chapter.title}\n\n${chapter.content}`);
  }
  return parts.join("\n");
}

function escapeXml(value: string): string {
  return value
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function chapterXhtml(title: string, content: string): string {
  const paragraphs = content
    .split(/\r?\n/)
    .map((line) => `<p>${escapeXml(line)}</p>`)
    .join("");
  return `<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head><title>${escapeXml(title)}</title><link rel="stylesheet" type="text/css" href="style.css"/></head>
<body><h1>${escapeXml(title)}</h1>${paragraphs || "<p></p>"}</body>
</html>`;
}

export function buildEpub(book: Book, chapters: BookChapterContent[]): Uint8Array<ArrayBuffer> {
  if (!chapters.length) throw new Error("没有可导出的章节");
  const files: Record<string, Uint8Array> = {};
  files["mimetype"] = strToU8("application/epub+zip");
  files["META-INF/container.xml"] = strToU8(`<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles><rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/></rootfiles>
</container>`);
  files["OEBPS/style.css"] = strToU8(
    "body{line-height:1.8;margin:1em;}h1{font-size:1.4em;margin:0 0 1em;}p{margin:0 0 .6em;text-indent:2em;}",
  );
  const manifest: string[] = [];
  const spine: string[] = [];
  chapters.forEach((chapter, index) => {
    const id = `chapter${index + 1}`;
    files[`OEBPS/${id}.xhtml`] = strToU8(chapterXhtml(chapter.title, chapter.content));
    manifest.push(`<item id="${id}" href="${id}.xhtml" media-type="application/xhtml+xml"/>`);
    spine.push(`<itemref idref="${id}"/>`);
  });
  files["OEBPS/nav.xhtml"] = strToU8(`<?xml version="1.0" encoding="utf-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xmlns:epub="http://www.idpf.org/2007/ops">
<head><title>目录</title></head><body><nav epub:type="toc"><ol>${chapters
    .map((chapter, index) => `<li><a href="chapter${index + 1}.xhtml">${escapeXml(chapter.title)}</a></li>`)
    .join("")}</ol></nav></body></html>`);
  const uuid = `urn:uuid:${Date.now().toString(16)}-${Math.random().toString(16).slice(2)}`;
  files["OEBPS/content.opf"] = strToU8(`<?xml version="1.0" encoding="utf-8"?>
<package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="bookid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="bookid">${uuid}</dc:identifier>
    <dc:title>${escapeXml(book.name)}</dc:title>
    <dc:creator>${escapeXml(book.author || "")}</dc:creator>
    <dc:language>zh-CN</dc:language>
  </metadata>
  <manifest>
    <item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/>
    <item id="css" href="style.css" media-type="text/css"/>
    ${manifest.join("")}
  </manifest>
  <spine>${spine.join("")}</spine>
</package>`);
  return zipSync(files, { level: 0 }) as Uint8Array<ArrayBuffer>;
}
