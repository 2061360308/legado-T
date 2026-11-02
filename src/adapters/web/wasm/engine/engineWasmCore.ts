type WasmExports = {
  memory: WebAssembly.Memory;
  engine_wasm_ping(): number;
  engine_wasm_init(): number;
  engine_wasm_eval(
    scriptPtr: number,
    scriptLen: number,
    baseUrlPtr: number,
    baseUrlLen: number,
    resultPtr: number,
    resultLen: number,
    sourcePtr: number,
    sourceLen: number,
  ): number;
  engine_wasm_last_ptr(): number;
  engine_wasm_last_len(): number;
  engine_wasm_alloc(len: number): number;
  engine_wasm_free(ptr: number, cap: number): void;
  engine_wasm_get_cookie(urlPtr: number, urlLen: number): number;
  engine_wasm_set_cookie(
    urlPtr: number,
    urlLen: number,
    cookiePtr: number,
    cookieLen: number,
  ): number;
  engine_wasm_remove_cookie(urlPtr: number, urlLen: number): number;
  engine_wasm_set_variables(
    sourceKeyPtr: number,
    sourceKeyLen: number,
    variablesPtr: number,
    variablesLen: number,
  ): number;
  engine_source_search(
    sourcePtr: number,
    sourceLen: number,
    keyPtr: number,
    keyLen: number,
    page: number,
  ): number;
  engine_source_explore(
    sourcePtr: number,
    sourceLen: number,
    urlPtr: number,
    urlLen: number,
    page: number,
  ): number;
  engine_source_info(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
  ): number;
  engine_source_toc(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
  ): number;
  engine_source_content(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
    chapterPtr: number,
    chapterLen: number,
    nextUrlPtr: number,
    nextUrlLen: number,
  ): number;
  engine_source_reviews(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
    chapterPtr: number,
    chapterLen: number,
  ): number;
  engine_source_download_urls(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
  ): number;
  engine_source_pay(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
    chapterPtr: number,
    chapterLen: number,
  ): number;
  engine_source_resource(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
    chapterPtr: number,
    chapterLen: number,
    nextUrlPtr: number,
    nextUrlLen: number,
  ): number;
  engine_source_image(
    sourcePtr: number,
    sourceLen: number,
    bookPtr: number,
    bookLen: number,
    chapterPtr: number,
    chapterLen: number,
    urlPtr: number,
    urlLen: number,
  ): number;
  engine_rss_articles(
    sourcePtr: number,
    sourceLen: number,
    sortNamePtr: number,
    sortNameLen: number,
    sortUrlPtr: number,
    sortUrlLen: number,
    page: number,
  ): number;
  engine_rss_content(
    sourcePtr: number,
    sourceLen: number,
    articlePtr: number,
    articleLen: number,
    rulePtr: number,
    ruleLen: number,
  ): number;
};

export interface EngineEvalInput {
  script: string;
  baseUrl: string;
  result?: string;
  sourceJson?: string | null;
}

const encoder = new TextEncoder();
const decoder = new TextDecoder();
let instance: WebAssembly.Instance | null = null;
let initPromise: Promise<WasmExports> | null = null;
let queue: Promise<unknown> = Promise.resolve();

function exportsOf(value: WebAssembly.Instance): WasmExports {
  return value.exports as unknown as WasmExports;
}

function bytes(exports: WasmExports): Uint8Array {
  return new Uint8Array(exports.memory.buffer);
}

function readText(exports: WasmExports, ptr: number, len: number): string {
  if (!len) return "";
  return decoder.decode(bytes(exports).slice(ptr, ptr + len));
}

function writeText(exports: WasmExports, text: string): { ptr: number; len: number } {
  const data = encoder.encode(text);
  const ptr = exports.engine_wasm_alloc(data.length);
  bytes(exports).set(data, ptr);
  return { ptr, len: data.length };
}

function readLast(exports: WasmExports): string {
  return readText(exports, exports.engine_wasm_last_ptr(), exports.engine_wasm_last_len());
}

function wasiImports(exports: () => WasmExports) {
  return {
    random_get(ptr: number, len: number) {
      const target = bytes(exports()).subarray(ptr, ptr + len);
      if (globalThis.crypto?.getRandomValues) {
        globalThis.crypto.getRandomValues(target);
      } else {
        for (let index = 0; index < target.length; index += 1) {
          target[index] = Math.floor(Math.random() * 256);
        }
      }
      return 0;
    },
    environ_get() {
      return 0;
    },
    environ_sizes_get(countPtr: number, sizePtr: number) {
      const view = new DataView(exports().memory.buffer);
      view.setUint32(countPtr, 0, true);
      view.setUint32(sizePtr, 0, true);
      return 0;
    },
    clock_time_get(_clock: number, _precision: bigint, outPtr: number) {
      const view = new DataView(exports().memory.buffer);
      view.setBigUint64(outPtr, BigInt(Date.now()) * 1000000n, true);
      return 0;
    },
    poll_oneoff(_inPtr: number, _outPtr: number, _count: number, eventCountPtr: number) {
      const view = new DataView(exports().memory.buffer);
      view.setUint32(eventCountPtr, 0, true);
      return 0;
    },
    fd_write(_fd: number, iovsPtr: number, iovsLen: number, writtenPtr: number) {
      const memory = bytes(exports());
      const view = new DataView(memory.buffer);
      let total = 0;
      let text = "";
      for (let index = 0; index < iovsLen; index += 1) {
        const ptr = view.getUint32(iovsPtr + index * 8, true);
        const len = view.getUint32(iovsPtr + index * 8 + 4, true);
        text += decoder.decode(memory.slice(ptr, ptr + len));
        total += len;
      }
      if (text) console.warn(text);
      new DataView(exports().memory.buffer).setUint32(writtenPtr, total, true);
      return 0;
    },
    fd_close() {
      return 0;
    },
    fd_fdstat_get(_fd: number, _statPtr: number) {
      return 0;
    },
    fd_prestat_get() {
      return 8;
    },
    fd_prestat_dir_name() {
      return 8;
    },
    fd_seek() {
      return 8;
    },
    sched_yield() {
      return 0;
    },
    proc_exit(code: number) {
      throw new Error(`engine wasm exited: ${code}`);
    },
  };
}

function httpImports(exports: () => WasmExports) {
  return {
    engine_http_fetch(urlPtr: number, urlLen: number, initPtr: number, initLen: number): number {
      const url = readText(exports(), urlPtr, urlLen);
      const init = JSON.parse(readText(exports(), initPtr, initLen) || "{}") as {
        method?: string;
        headers?: Record<string, string>;
        body?: string | null;
      };
      let response: Record<string, unknown>;
      try {
        const xhr = new XMLHttpRequest();
        xhr.open(init.method || "GET", "/api/engine/fetch", false);
        xhr.setRequestHeader("X-Remote-Target", url);
        for (const [key, value] of Object.entries(init.headers || {})) {
          if (key.toLowerCase() !== "host" && key.toLowerCase() !== "content-length") {
            try {
              xhr.setRequestHeader(key, value);
            } catch {
              // ignore forbidden browser headers
            }
          }
        }
        xhr.send(init.body == null ? null : init.body);
        const headers: Record<string, string> = {};
        for (const line of (xhr.getAllResponseHeaders() || "").trim().split(/[\r\n]+/)) {
          const index = line.indexOf(":");
          if (index > 0) {
            headers[line.slice(0, index).trim().toLowerCase()] = line.slice(index + 1).trim();
          }
        }
        response = {
          status: xhr.status,
          url: xhr.responseURL || url,
          headers,
          text: xhr.responseText || "",
        };
      } catch (error) {
        response = {
          status: 502,
          url,
          headers: {},
          text: error instanceof Error ? error.message : String(error),
        };
      }
      const data = encoder.encode(JSON.stringify(response));
      const ptr = exports().engine_wasm_alloc(data.length + 4);
      const memory = bytes(exports());
      new DataView(memory.buffer).setUint32(ptr, data.length, true);
      memory.set(data, ptr + 4);
      return ptr;
    },
  };
}

async function load(): Promise<WasmExports> {
  const response = await fetch(new URL("./engine_js_wasm.wasm", import.meta.url));
  if (!response.ok) throw new Error(`引擎 wasm 加载失败：${response.status}`);
  const module = await WebAssembly.instantiate(await response.arrayBuffer(), {
    wasi_snapshot_preview1: wasiImports(() => exportsOf(instance!)),
    engine_http: httpImports(() => exportsOf(instance!)),
  });
  instance = module.instance;
  const loaded = exportsOf(instance);
  if (loaded.engine_wasm_ping() !== 1) throw new Error("引擎 wasm 导出异常");
  const initCode = loaded.engine_wasm_init();
  if (initCode !== 0) throw new Error(readLast(loaded) || "引擎 wasm 初始化失败");
  return loaded;
}

function ensureLoaded(): Promise<WasmExports> {
  if (!initPromise) {
    initPromise = load().catch((error) => {
      initPromise = null;
      instance = null;
      throw error;
    });
  }
  return initPromise;
}

function runSerialized<T>(task: (exports: WasmExports) => T): Promise<T> {
  const next = queue.then(async () => task(await ensureLoaded()));
  queue = next.catch(() => undefined);
  return next;
}

export function evalEngine(input: EngineEvalInput): Promise<string> {
  return runSerialized((exports) => {
    const script = writeText(exports, input.script);
    const baseUrl = writeText(exports, input.baseUrl);
    const result = writeText(exports, input.result ?? "");
    const source = input.sourceJson ? writeText(exports, input.sourceJson) : null;
    try {
      const code = exports.engine_wasm_eval(
        script.ptr,
        script.len,
        baseUrl.ptr,
        baseUrl.len,
        result.ptr,
        result.len,
        source?.ptr ?? 0,
        source?.len ?? 0,
      );
      const output = readLast(exports);
      if (code !== 0) throw new Error(output || "引擎 wasm 执行失败");
      return output;
    } finally {
      exports.engine_wasm_free(script.ptr, script.len);
      exports.engine_wasm_free(baseUrl.ptr, baseUrl.len);
      exports.engine_wasm_free(result.ptr, result.len);
      if (source) exports.engine_wasm_free(source.ptr, source.len);
    }
  });
}

function callWithText<T>(
  parts: string[],
  invoke: (exports: WasmExports, written: { ptr: number; len: number }[]) => number,
  parse: (value: string) => T,
): Promise<T> {
  return runSerialized((exports) => {
    const written = parts.map((part) => writeText(exports, part));
    try {
      const code = invoke(exports, written);
      const output = readLast(exports);
      if (code !== 0) throw new Error(output || "引擎 wasm 执行失败");
      return parse(output);
    } finally {
      for (const item of written) exports.engine_wasm_free(item.ptr, item.len);
    }
  });
}

function parseJson<T>(value: string): T {
  return JSON.parse(value) as T;
}

export function searchSource(
  sourceJson: string,
  keyword: string,
  page = 1,
): Promise<unknown[]> {
  return callWithText<unknown[]>(
    [sourceJson, keyword],
    (exports, [source, key]) =>
      exports.engine_source_search(source.ptr, source.len, key.ptr, key.len, page),
    parseJson,
  );
}

export function exploreSource(
  sourceJson: string,
  url: string,
  page = 1,
): Promise<unknown[]> {
  return callWithText<unknown[]>(
    [sourceJson, url],
    (exports, [source, target]) =>
      exports.engine_source_explore(source.ptr, source.len, target.ptr, target.len, page),
    parseJson,
  );
}

export function sourceInfo(sourceJson: string, bookJson: string): Promise<unknown> {
  return callWithText(
    [sourceJson, bookJson],
    (exports, [source, book]) =>
      exports.engine_source_info(source.ptr, source.len, book.ptr, book.len),
    parseJson,
  );
}

export function sourceToc(sourceJson: string, bookJson: string): Promise<unknown[]> {
  return callWithText<unknown[]>(
    [sourceJson, bookJson],
    (exports, [source, book]) =>
      exports.engine_source_toc(source.ptr, source.len, book.ptr, book.len),
    parseJson,
  );
}

export function sourceContent(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
  nextUrl?: string | null,
): Promise<string> {
  return callWithText(
    [sourceJson, bookJson, chapterJson, nextUrl ?? ""],
    (exports, [source, book, chapter, next]) =>
      exports.engine_source_content(
        source.ptr,
        source.len,
        book.ptr,
        book.len,
        chapter.ptr,
        chapter.len,
        next.ptr,
        next.len,
      ),
    (value) => value,
  );
}

export function rssArticles(
  sourceJson: string,
  sortName: string,
  sortUrl: string,
  page = 1,
): Promise<unknown> {
  return callWithText(
    [sourceJson, sortName, sortUrl],
    (exports, [source, name, url]) =>
      exports.engine_rss_articles(
        source.ptr,
        source.len,
        name.ptr,
        name.len,
        url.ptr,
        url.len,
        page,
      ),
    parseJson,
  );
}

export function rssContent(
  sourceJson: string,
  articleJson: string,
  ruleContent: string,
): Promise<string> {
  return callWithText(
    [sourceJson, articleJson, ruleContent],
    (exports, [source, article, rule]) =>
      exports.engine_rss_content(
        source.ptr,
        source.len,
        article.ptr,
        article.len,
        rule.ptr,
        rule.len,
      ),
    (value) => value,
  );
}

export function sourceReviews(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
): Promise<unknown[]> {
  return callWithText<unknown[]>(
    [sourceJson, bookJson, chapterJson],
    (exports, [source, book, chapter]) =>
      exports.engine_source_reviews(
        source.ptr,
        source.len,
        book.ptr,
        book.len,
        chapter.ptr,
        chapter.len,
      ),
    parseJson,
  );
}

export function sourceDownloadUrls(
  sourceJson: string,
  bookJson: string,
): Promise<unknown[]> {
  return callWithText<unknown[]>(
    [sourceJson, bookJson],
    (exports, [source, book]) =>
      exports.engine_source_download_urls(source.ptr, source.len, book.ptr, book.len),
    parseJson,
  );
}

export function sourcePay(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
): Promise<unknown> {
  return callWithText(
    [sourceJson, bookJson, chapterJson],
    (exports, [source, book, chapter]) =>
      exports.engine_source_pay(
        source.ptr,
        source.len,
        book.ptr,
        book.len,
        chapter.ptr,
        chapter.len,
      ),
    parseJson,
  );
}

export function sourceResource(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
  nextUrl?: string | null,
): Promise<string> {
  return callWithText(
    [sourceJson, bookJson, chapterJson, nextUrl ?? ""],
    (exports, [source, book, chapter, next]) =>
      exports.engine_source_resource(
        source.ptr,
        source.len,
        book.ptr,
        book.len,
        chapter.ptr,
        chapter.len,
        next.ptr,
        next.len,
      ),
    (value) => value,
  );
}

export function sourceImage(
  sourceJson: string,
  bookJson: string,
  chapterJson: string,
  url?: string | null,
): Promise<string> {
  return callWithText(
    [sourceJson, bookJson, chapterJson, url ?? ""],
    (exports, [source, book, chapter, target]) =>
      exports.engine_source_image(
        source.ptr,
        source.len,
        book.ptr,
        book.len,
        chapter.ptr,
        chapter.len,
        target.ptr,
        target.len,
      ),
    (value) => value,
  );
}

export function setSourceVariables(
  sourceKey: string,
  variables: Record<string, string>,
): Promise<void> {
  return callWithText(
    [sourceKey, JSON.stringify(variables)],
    (exports, [key, value]) =>
      exports.engine_wasm_set_variables(key.ptr, key.len, value.ptr, value.len),
    () => undefined,
  );
}

export function setEngineCookie(url: string, cookie: string): Promise<void> {
  return callWithText(
    [url, cookie],
    (exports, [target, value]) =>
      exports.engine_wasm_set_cookie(target.ptr, target.len, value.ptr, value.len),
    () => undefined,
  );
}

export function removeEngineCookie(url: string): Promise<void> {
  return callWithText(
    [url],
    (exports, [target]) => exports.engine_wasm_remove_cookie(target.ptr, target.len),
    () => undefined,
  );
}

export function getEngineCookie(url: string): Promise<string> {
  return callWithText(
    [url],
    (exports, [target]) => exports.engine_wasm_get_cookie(target.ptr, target.len),
    (value) => value,
  );
}

export function warmupEngine(): Promise<void> {
  return runSerialized(() => undefined);
}
