import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { unzipSync } from "fflate";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const version = readFileSync(resolve(root, "VERSION"), "utf8")
  .replace(/^v/, "")
  .trim();
const tag = `release-${version}`;
const assetUrl =
  `https://github.com/2061360308/legado-T/releases/download/${tag}/LegadoT-wasm.zip`;

const headers = process.env.GH_TOKEN
  ? { Authorization: `Bearer ${process.env.GH_TOKEN}` }
  : {};

console.log(`Downloading ${assetUrl}`);
const response = await fetch(assetUrl, {
  headers,
  redirect: "follow",
});

if (!response.ok) {
  throw new Error(
    `Unable to download ${assetUrl}: HTTP ${response.status}. ` +
      `Make sure GitHub Release ${tag} has finished building.`,
  );
}

const archive = new Uint8Array(await response.arrayBuffer());
const files = unzipSync(archive);
const target = resolve(root, "src/adapters/web/wasm");
const allowedPrefixes = ["pkg/", "shelf/", "ebook/", "rbook/", "backup/", "engine/"];
let written = 0;

for (const [name, data] of Object.entries(files)) {
  if (name.endsWith("/")) continue;
  if (!allowedPrefixes.some((prefix) => name.startsWith(prefix))) continue;

  const output = resolve(target, name);
  if (output !== target && !output.startsWith(`${target}\\`) && !output.startsWith(`${target}/`)) {
    throw new Error(`Refusing to extract path outside wasm directory: ${name}`);
  }

  mkdirSync(dirname(output), { recursive: true });
  writeFileSync(output, data);
  written += 1;
}

const required = [
  "pkg/reader_core.js",
  "pkg/reader_core_bg.wasm",
  "shelf/reader_core.js",
  "ebook/ebook_wasm.js",
  "rbook/rbook_wasm.js",
  "backup/backup_wasm.js",
  "engine/engine_js_wasm.wasm",
];

for (const name of required) {
  if (!files[name]) {
    throw new Error(`Release archive is missing ${name}`);
  }
}

console.log(`Extracted ${written} wasm files for ${tag}`);
