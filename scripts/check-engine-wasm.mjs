import { randomFillSync } from "node:crypto";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const wasmPath = resolve(
  "src/adapters/web/wasm/engine/engine_js_wasm.wasm",
);
const bytes = readFileSync(wasmPath);
let instance;
const encoder = new TextEncoder();
const decoder = new TextDecoder();
const memory = () => new Uint8Array(instance.exports.memory.buffer);
const read = (ptr, len) => decoder.decode(memory().slice(ptr, ptr + len));
const write = (text) => {
  const data = encoder.encode(text);
  const ptr = instance.exports.engine_wasm_alloc(data.length);
  memory().set(data, ptr);
  return { ptr, len: data.length };
};
const last = () =>
  read(
    instance.exports.engine_wasm_last_ptr(),
    instance.exports.engine_wasm_last_len(),
  );

const wasi = {
  random_get(ptr, len) {
    randomFillSync(memory().subarray(ptr, ptr + len));
    return 0;
  },
  environ_get() {
    return 0;
  },
  environ_sizes_get(countPtr, sizePtr) {
    const view = new DataView(instance.exports.memory.buffer);
    view.setUint32(countPtr, 0, true);
    view.setUint32(sizePtr, 0, true);
    return 0;
  },
  clock_time_get(_clock, _precision, outPtr) {
    new DataView(instance.exports.memory.buffer).setBigUint64(
      outPtr,
      BigInt(Date.now()) * 1000000n,
      true,
    );
    return 0;
  },
  poll_oneoff(_inPtr, _outPtr, _count, eventCountPtr) {
    new DataView(instance.exports.memory.buffer).setUint32(
      eventCountPtr,
      0,
      true,
    );
    return 0;
  },
  fd_write(_fd, iovsPtr, iovsLen, writtenPtr) {
    const memoryView = memory();
    const view = new DataView(memoryView.buffer);
    let total = 0;
    for (let index = 0; index < iovsLen; index += 1) {
      total += view.getUint32(iovsPtr + index * 8 + 4, true);
    }
    view.setUint32(writtenPtr, total, true);
    return 0;
  },
  fd_close() {
    return 0;
  },
  fd_fdstat_get() {
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
  proc_exit(code) {
    throw new Error(`engine wasm exited: ${code}`);
  },
};

const imports = {
  wasi_snapshot_preview1: wasi,
  engine_http: {
    engine_http_fetch() {
      const data = encoder.encode(
        JSON.stringify({
          status: 200,
          url: "https://example.com",
          headers: {},
          text: "ok",
        }),
      );
      const ptr = instance.exports.engine_wasm_alloc(data.length + 4);
      new DataView(instance.exports.memory.buffer).setUint32(
        ptr,
        data.length,
        true,
      );
      memory().set(data, ptr + 4);
      return ptr;
    },
  },
};

const module = new WebAssembly.Module(bytes);
const exports = WebAssembly.Module.exports(module).map((item) => item.name);
for (const name of [
  "engine_wasm_ping",
  "engine_wasm_init",
  "engine_wasm_eval",
  "engine_source_search",
  "engine_source_info",
  "engine_source_toc",
  "engine_source_content",
  "engine_source_image",
  "engine_rss_articles",
  "engine_rss_content",
]) {
  if (!exports.includes(name)) throw new Error(`missing wasm export: ${name}`);
}

instance = new WebAssembly.Instance(module, imports);
if (instance.exports.engine_wasm_ping() !== 1) throw new Error("ping failed");
if (instance.exports.engine_wasm_init() !== 0) {
  throw new Error(last() || "engine init failed");
}
const script = write("JSON.stringify({ok:1+1})");
const base = write("https://example.com/");
const result = write("");
const code = instance.exports.engine_wasm_eval(
  script.ptr,
  script.len,
  base.ptr,
  base.len,
  result.ptr,
  result.len,
  0,
  0,
);
if (code !== 0 || last() !== '{"ok":2}') {
  throw new Error(`engine eval failed: ${last()}`);
}
console.log(`engine wasm ok: ${exports.length} exports`);
