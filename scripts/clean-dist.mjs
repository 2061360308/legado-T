import { rmSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");

for (const name of ["dist", "dist-ssr"]) {
  rmSync(resolve(root, name), { recursive: true, force: true });
  console.log(`removed ${name}`);
}
