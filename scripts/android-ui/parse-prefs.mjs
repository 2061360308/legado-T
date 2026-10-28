import fs from "node:fs";
import path from "node:path";
import {
  APP_RES,
  OUT_DESIGN,
  loadResourceBundle,
  readXml,
  resolveAttrValue,
  writeFile,
} from "./android.mjs";

const bundle = loadResourceBundle();
const dir = path.join(APP_RES, "xml");
const out = path.join(OUT_DESIGN, "prefs");

function tagOf(name) {
  return name.split(".").pop();
}

function value(raw) {
  if (raw === undefined) return undefined;
  let result = resolveAttrValue(raw, bundle);
  if (typeof result === "string" && result.startsWith("@drawable/")) {
    result = result.slice(10);
  }
  return result;
}

function arrayValue(raw) {
  if (typeof raw !== "string" || !raw.startsWith("@array/")) return undefined;
  return bundle.arrays[raw.slice(7)] ?? [];
}

function normalize(node, name) {
  const attrs = node ?? {};
  const title = value(attrs["@_android:title"] ?? attrs["@_app:title"]);
  const summary = value(attrs["@_android:summary"] ?? attrs["@_app:summary"]);
  const icon = value(attrs["@_android:icon"]);
  const key = attrs["@_android:key"];
  const defaultValue = value(attrs["@_android:defaultValue"]);
  const dependency = attrs["@_android:dependency"] ?? "";
  const entries = arrayValue(attrs["@_android:entries"]) ?? [];
  const entryValues = arrayValue(attrs["@_android:entryValues"]) ?? [];
  const children = [];
  for (const [childName, childValue] of Object.entries(attrs)) {
    if (
      childName.startsWith("@_") ||
      childName.startsWith("#") ||
      childName.startsWith("?")
    ) {
      continue;
    }
    const list = Array.isArray(childValue) ? childValue : [childValue];
    for (const child of list) {
      if (typeof child !== "object" || child === null) continue;
      children.push(normalize(child, childName));
    }
  }
  return {
    tag: tagOf(name),
    title: typeof title === "string" ? title : "",
    summary: typeof summary === "string" ? summary : "",
    icon: typeof icon === "string" ? icon : "",
    key: typeof key === "string" ? key : "",
    defaultValue: defaultValue === undefined ? "" : String(defaultValue),
    dependency: typeof dependency === "string" ? dependency : "",
    entries,
    entryValues,
    children,
  };
}

fs.mkdirSync(out, { recursive: true });
const index = [];
const files = fs
  .readdirSync(dir)
  .filter((name) => name.endsWith(".xml"))
  .sort((a, b) => a.localeCompare(b));
for (const file of files) {
  const doc = readXml(path.join(dir, file));
  const key = Object.keys(doc).find((item) => !item.startsWith("?"));
  const rootRaw = doc[key];
  const root = Array.isArray(rootRaw) ? rootRaw[0] : rootRaw;
  const name = path.basename(file, ".xml");
  const rootTag = tagOf(key);
  const tree =
    rootTag === "PreferenceScreen"
      ? normalize(root, key)
      : {
          tag: rootTag,
          title: "",
          summary: "",
          icon: "",
          key: "",
          defaultValue: "",
          dependency: "",
          entries: [],
          entryValues: [],
          children: [],
        };
  writeFile(path.join(out, `${name}.json`), JSON.stringify(tree, null, 2));
  index.push({ name, file: `xml/${file}` });
}
writeFile(path.join(out, "index.json"), JSON.stringify(index, null, 2));
console.log(`parsed ${index.length} preference files`);
