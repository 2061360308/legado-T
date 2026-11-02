import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const isWindows = process.platform === "win32";
let failures = 0;

function ok(message) {
  console.log(`[ok]   ${message}`);
}

function warn(message) {
  console.log(`[warn] ${message}`);
}

function fail(message) {
  failures += 1;
  console.log(`[fail] ${message}`);
}

function command(name, args = ["--version"]) {
  const result = spawnSync(name, args, {
    encoding: "utf8",
    shell: false,
  });
  if (result.error || result.status !== 0) return null;
  return `${result.stdout || ""}${result.stderr || ""}`.trim();
}

function checkCommand(label, name, args) {
  const version = command(name, args);
  if (version) ok(`${label}: ${version.split(/\r?\n/)[0]}`);
  else fail(`${label} not found`);
}

function checkPath(label, path, required = false) {
  if (!path) {
    (required ? fail : warn)(`${label} is not set`);
    return;
  }
  if (existsSync(path)) ok(`${label}: ${path}`);
  else (required ? fail : warn)(`${label} path does not exist: ${path}`);
}

checkCommand("Node", "node", ["--version"]);
checkCommand("Cargo", "cargo", ["--version"]);
checkCommand("Rust", "rustc", ["--version"]);
checkCommand("wasm-bindgen", "wasm-bindgen", ["--version"]);

const targets = command("rustup", ["target", "list", "--installed"]);
if (targets) {
  for (const target of [
    "wasm32-unknown-unknown",
    "wasm32-wasip1",
    "aarch64-linux-android",
    "x86_64-linux-android",
  ]) {
    if (targets.includes(target)) ok(`Rust target: ${target}`);
    else warn(`Rust target missing: rustup target add ${target}`);
  }
} else {
  warn("rustup not found; cannot inspect Rust targets");
}

checkPath("WASI_SDK", process.env.WASI_SDK);
checkPath("JAVA_HOME", process.env.JAVA_HOME);
checkPath("ANDROID_HOME", process.env.ANDROID_HOME || process.env.ANDROID_SDK_ROOT);
checkPath(
  "Android NDK",
  process.env.ANDROID_NDK_HOME || process.env.NDK_HOME,
);
checkPath("LIBCLANG_PATH", process.env.LIBCLANG_PATH);

const versionFile = resolve(root, "..", "VERSION");
if (existsSync(versionFile)) {
  const version = readFileSync(versionFile, "utf8").trim();
  if (/^\d+\.\d+\.\d+(?:[-+].*)?$/.test(version)) ok(`VERSION: ${version}`);
  else fail(`VERSION is not semver: ${version}`);
} else {
  fail(`VERSION not found: ${versionFile}`);
}

const keyProperties = resolve(root, "src-tauri/gen/android/app/key.properties");
if (existsSync(keyProperties)) ok("Android key.properties found");
else warn("Android key.properties missing; release APK signing unavailable");

for (const file of [
  "src-tauri/resources/pdfium/pdfium.dll",
  "src-tauri/resources/pdfium/libpdfium.so",
]) {
  if (existsSync(resolve(root, file))) ok(`PDFium resource: ${file}`);
  else warn(`PDFium resource missing: ${file}`);
}

if (existsSync(resolve(root, "node_modules/.bin", isWindows ? "tauri.cmd" : "tauri"))) {
  ok("Tauri CLI installed");
} else {
  fail("Tauri CLI missing; run npm install");
}

if (failures) {
  console.log(`\n${failures} required check(s) failed`);
  process.exitCode = 1;
} else {
  console.log("\nEnvironment looks usable");
}
