import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function build(crateName, outDir, extraArgs = []) {
  const crate = resolve(root, `src-tauri/crates/${crateName}`);
  execFileSync(
    "cargo",
    [
      "build",
      "--target-dir",
      resolve(crate, "target"),
      ...extraArgs,
      "--target",
      "wasm32-unknown-unknown",
      "--release",
    ],
    {
      cwd: crate,
      stdio: "inherit",
    },
  );
  mkdirSync(outDir, { recursive: true });
  execFileSync(
    "wasm-bindgen",
    [
      resolve(
        crate,
        `target/wasm32-unknown-unknown/release/${crateName.replace(/-/g, "_")}.wasm`,
      ),
      "--out-dir",
      outDir,
      "--target",
      "web",
    ],
    { stdio: "inherit" },
  );
}

function buildEngine() {
  const wasiSdk = process.env.WASI_SDK;
  if (!wasiSdk) {
    const message =
      "WASI_SDK is required to build engine-js-wasm. " +
      "Set WASI_SDK to a wasi-sdk installation, or set " +
      "ALLOW_MISSING_ENGINE_WASM=1 to build a Web bundle without book sources.";
    if (process.env.ALLOW_MISSING_ENGINE_WASM === "1") {
      console.warn(message);
      return;
    }
    throw new Error(message);
  }
  const srcTauri = resolve(root, "src-tauri");
  const env = {
    ...process.env,
    WASI_SDK: wasiSdk,
    LIBCLANG_PATH: resolve(wasiSdk, "bin"),
    PATH: `${resolve(wasiSdk, "bin")};${process.env.PATH ?? ""}`,
  };
  execFileSync(
    "cargo",
    ["build", "-p", "engine-js-wasm", "--target", "wasm32-wasip1", "--release"],
    { cwd: srcTauri, env, stdio: "inherit" },
  );
  const outDir = resolve(root, "src/adapters/web/wasm/engine");
  mkdirSync(outDir, { recursive: true });
  copyFileSync(
    resolve(srcTauri, "target/wasm32-wasip1/release/engine_js_wasm.wasm"),
    resolve(outDir, "engine_js_wasm.wasm"),
  );
}

build("reader-core", resolve(root, "src/adapters/web/wasm/pkg"), [
  "--no-default-features",
  "--features",
  "full",
]);
build("reader-core", resolve(root, "src/adapters/web/wasm/shelf"), [
  "--no-default-features",
  "--features",
  "shelf",
]);
build("ebook-wasm", resolve(root, "src/adapters/web/wasm/ebook"));
build("rbook-wasm", resolve(root, "src/adapters/web/wasm/rbook"));
build("backup-wasm", resolve(root, "src/adapters/web/wasm/backup"));
buildEngine();
