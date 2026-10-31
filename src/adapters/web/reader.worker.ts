import type { LayoutStyle, ReaderBlock } from "@/core/domain";
import initFull, {
  analyze_txt_with_rule_json,
  layout_blocks_json,
  parse_umd_json,
  process_chapter_json,
  process_content_json,
  search_text_json,
} from "./wasm/pkg/reader_core.js";
import initShelf, { group_books_json } from "./wasm/shelf/reader_core.js";

let fullReady: Promise<unknown> | null = null;
let shelfReady: Promise<unknown> | null = null;

function ensureFull(): Promise<unknown> {
  if (!fullReady) fullReady = initFull();
  return fullReady;
}

function ensureShelf(): Promise<unknown> {
  if (!shelfReady) shelfReady = initShelf();
  return shelfReady;
}

type WorkerRequest =
  | {
      id: number;
      kind: "layout";
      title: string;
      blocks: ReaderBlock[];
      style: LayoutStyle;
    }
  | {
      id: number;
      kind: "analyzeTxt";
      bytes: Uint8Array;
      charset: string;
      rules: unknown[];
      splitLong: boolean;
      ruleExpr: string;
    }
  | {
      id: number;
      kind: "search";
      chapters: unknown[];
      keyword: string;
      limit: number;
    }
  | {
      id: number;
      kind: "processContent";
      request: unknown;
    }
  | {
      id: number;
      kind: "shelfBooks";
      books: unknown[];
      groupId: number;
      sort: number;
    }
  | {
      id: number;
      kind: "chapter";
      request: unknown;
    }
  | {
      id: number;
      kind: "parseUmd";
      bytes: Uint8Array;
      titleFallback: string;
    };

self.onmessage = async (event: MessageEvent) => {
  const data = event.data as WorkerRequest;
  try {
    let result: unknown;
    switch (data.kind) {
      case "shelfBooks": {
        await ensureShelf();
        result = JSON.parse(
          group_books_json(
            JSON.stringify(data.books),
            data.groupId,
            data.sort,
          ),
        );
        break;
      }
      case "layout": {
        await ensureFull();
        result = JSON.parse(
          layout_blocks_json(
            data.title,
            JSON.stringify(data.blocks),
            JSON.stringify(data.style),
          ),
        );
        break;
      }
      case "analyzeTxt": {
        await ensureFull();
        result = JSON.parse(
          analyze_txt_with_rule_json(
            data.bytes,
            data.charset,
            JSON.stringify(data.rules),
            data.ruleExpr,
            data.splitLong,
          ),
        );
        break;
      }
      case "processContent": {
        await ensureFull();
        result = JSON.parse(process_content_json(JSON.stringify(data.request)));
        break;
      }
      case "chapter": {
        await ensureFull();
        result = JSON.parse(process_chapter_json(JSON.stringify(data.request)));
        break;
      }
      case "parseUmd": {
        await ensureFull();
        result = JSON.parse(parse_umd_json(data.bytes, data.titleFallback));
        break;
      }
      case "search": {
        await ensureFull();
        result = JSON.parse(
          search_text_json(
            JSON.stringify(data.chapters),
            data.keyword,
            data.limit,
          ),
        );
        break;
      }
    }
    self.postMessage({ id: data.id, result });
  } catch (error) {
    self.postMessage({ id: data.id, error: String(error) });
  }
};
