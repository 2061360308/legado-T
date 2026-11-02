import { execFileSync } from "node:child_process";
import { existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const tauri = resolve(root, "node_modules", "@tauri-apps", "cli", "tauri.js");

if (!existsSync(tauri)) {
  throw new Error("Tauri CLI not found. Run npm install first.");
}

execFileSync(process.execPath, [resolve(root, "scripts/apply-version.mjs")], {
  cwd: root,
  stdio: "inherit",
});

const args = ["build", "--ci"];
if (process.platform === "win32") {
  args.push("--no-bundle");
} else if (process.platform === "linux") {
  args.push("--bundles", "appimage,deb");
}

execFileSync(process.execPath, [tauri, ...args], {
  cwd: root,
  stdio: "inherit",
});
