import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

function build(crateName, outDir, extraArgs = []) {
  const crate = resolve(root, `src-tauri/crates/${crateName}`);
  execFileSync(
    "cargo",
    ["build", ...extraArgs, "--target", "wasm32-unknown-unknown", "--release"],
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
