import fs from "node:fs";
import path from "node:path";
import { XMLParser } from "fast-xml-parser";
import {
  APP_RES,
  OUT_DESIGN,
  loadResourceBundle,
  resolveAttrValue,
  writeFile,
} from "./android.mjs";

const bundle = loadResourceBundle();
const dir = path.join(APP_RES, "menu");
const out = path.join(OUT_DESIGN, "menus");

// preserveOrder：保留 <group>/<item> 的真实文档顺序
const orderedParser = new XMLParser({
  ignoreAttributes: false,
  attributeNamePrefix: "@_",
  preserveOrder: true,
  parseTagValue: false,
  trimValues: true,
});

function tagOf(name) {
  return name.split(".").pop();
}

function normalizeAttrs(attrs) {
  const out = {};
  for (const [key, value] of Object.entries(attrs ?? {})) {
    if (!key.startsWith("@_")) continue;
    const raw = key.replace("@_android:", "").replace("@_app:", "").replace("@_", "");
    let result = value;
    if (raw === "id") result = String(result).replace("@+id/", "").replace("@id/", "");
    if (raw === "title" || raw === "icon") {
      result = resolveAttrValue(result, bundle);
      if (raw === "icon" && typeof result === "string" && result.startsWith("@drawable/")) {
        result = result.slice(10);
      }
    }
    out[raw] = result;
  }
  return out;
}

function convert(nodes) {
  const out = [];
  for (const node of nodes) {
    for (const [key, value] of Object.entries(node)) {
      if (key === ":@" || key.startsWith("?") || key.startsWith("#")) continue;
      out.push({
        tag: tagOf(key),
        attrs: node[":@"] ?? {},
        children: Array.isArray(value) ? convert(value) : [],
      });
    }
  }
  return out;
}

function normalizeTree(node) {
  return {
    tag: node.tag,
    attrs: normalizeAttrs(node.attrs),
    children: node.children
      .filter((child) => ["item", "group", "menu"].includes(child.tag))
      .map(normalizeTree),
  };
}

fs.mkdirSync(out, { recursive: true });
const index = [];
const files = fs
  .readdirSync(dir)
  .filter((name) => name.endsWith(".xml"))
  .sort((a, b) => a.localeCompare(b));
for (const file of files) {
  const doc = orderedParser.parse(fs.readFileSync(path.join(dir, file), "utf8"));
  const roots = convert(doc);
  const root = roots[0] ?? { tag: "menu", attrs: {}, children: [] };
  const tree = normalizeTree(root);
  const name = path.basename(file, ".xml");
  writeFile(path.join(out, `${name}.json`), JSON.stringify(tree, null, 2));
  index.push({ name, file: `menu/${file}` });
}
writeFile(path.join(out, "index.json"), JSON.stringify(index, null, 2));
console.log(`parsed ${index.length} menus`);
