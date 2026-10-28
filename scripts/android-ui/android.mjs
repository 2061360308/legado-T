import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { XMLParser } from "fast-xml-parser";

export const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
export const APP_RES = path.join(ROOT, "app/src/main/res");
export const OUT_DESIGN = path.join(ROOT, "local-reader/src/design");
export const OUT_DRAWABLE = path.join(ROOT, "local-reader/src/assets/android/drawable");

export const parser = new XMLParser({
  ignoreAttributes: false,
  attributeNamePrefix: "@_",
  textNodeName: "#text",
  parseTagValue: false,
  trimValues: true,
  isArray: (name) =>
    [
      "item",
      "path",
      "group",
      "clip-path",
      "string",
      "color",
      "dimen",
      "style",
      "array",
      "integer",
      "bool",
      "declare-styleable",
      "attr",
      "menu",
      "vector",
    ].includes(name),
});

export function readXml(file) {
  return parser.parse(fs.readFileSync(file, "utf8"));
}

export function attr(node, name) {
  if (!node) return undefined;
  return (
    node[`@_android:${name}`] ??
    node[`@_app:${name}`] ??
    node[`@_${name}`] ??
    undefined
  );
}

export function children(node, name) {
  if (!node) return [];
  const value = node[name];
  if (value === undefined) return [];
  return Array.isArray(value) ? value : [value];
}

export function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

export function writeFile(file, content) {
  ensureDir(path.dirname(file));
  fs.writeFileSync(file, content, "utf8");
}

export function parseValues(file) {
  if (!fs.existsSync(file)) return {};
  return readXml(file);
}

export function resourceMap(file, tag, keyAttr = "name", valueAttr = undefined) {
  const root = parseValues(file);
  const resources = root.resources ?? {};
  const items = children(resources, tag);
  const map = {};
  for (const item of items) {
    const key = item[`@_${keyAttr}`];
    if (!key) continue;
    map[key] = valueAttr ? item[`@_${valueAttr}`] : (item["#text"] ?? "");
  }
  return map;
}

export function resourceArrays(file) {
  if (!fs.existsSync(file)) return {};
  const root = parseValues(file);
  const resources = root.resources ?? {};
  const map = {};
  for (const tag of ["string-array", "array"]) {
    for (const item of children(resources, tag)) {
      const key = item["@_name"];
      if (!key) continue;
      const values = [];
      for (const entry of children(item, "item")) {
        if (typeof entry === "string") values.push(entry);
        else if (entry && typeof entry === "object") values.push(String(entry["#text"] ?? ""));
      }
      map[key] = values;
    }
  }
  return map;
}

export function resolveRefs(map, value, prefix = "@") {
  let current = value;
  let guard = 0;
  while (typeof current === "string" && current.startsWith(prefix) && guard < 20) {
    const key = current
      .replace(/^@color\//, "")
      .replace(/^@dimen\//, "")
      .replace(/^@string\//, "")
      .replace(/^@/, "");
    if (!(key in map)) break;
    current = map[key];
    guard += 1;
  }
  return current;
}

export function loadResourceBundle() {
  const colorsLight = {
    ...resourceMap(path.join(APP_RES, "values/colors_material_design.xml"), "color"),
    ...resourceMap(path.join(APP_RES, "values/colors.xml"), "color"),
  };
  const colorsNight = {
    ...colorsLight,
    ...resourceMap(path.join(APP_RES, "values-night/colors.xml"), "color"),
  };
  const dimens = resourceMap(path.join(APP_RES, "values/dimens.xml"), "dimen");
  const stringsZh = resourceMap(path.join(APP_RES, "values-zh/strings.xml"), "string");
  const stringsBase = resourceMap(path.join(APP_RES, "values/strings.xml"), "string");
  const strings = { ...stringsBase, ...stringsZh };
  for (const map of [colorsLight, colorsNight]) {
    for (const key of Object.keys(map)) {
      map[key] = resolveRefs(map, map[key]);
    }
  }
  const arraysBase = {
    ...resourceArrays(path.join(APP_RES, "values/arrays.xml")),
    ...resourceArrays(path.join(APP_RES, "values/array_values.xml")),
  };
  const arraysZh = resourceArrays(path.join(APP_RES, "values-zh/arrays.xml"));
  const arrays = { ...arraysBase, ...arraysZh };
  for (const key of Object.keys(arrays)) {
    arrays[key] = arrays[key].map((item) => resolveRefs(strings, item));
  }
  return { colorsLight, colorsNight, dimens, strings, arrays };
}

export function dimensionToPx(value) {
  if (typeof value !== "string") return value;
  const match = value.trim().match(/^(-?\d+(?:\.\d+)?)(dp|dip|sp|px|pt|in|mm)?$/);
  if (!match) return value;
  const number = Number(match[1]);
  if (match[2] === "sp" || match[2] === "dp" || match[2] === "dip") return number;
  if (match[2] === "pt") return (number * 4) / 3;
  return number;
}

export function resolveAttrValue(value, bundle) {
  if (typeof value !== "string") return value;
  let out = value;
  if (out.startsWith("@color/")) {
    const key = out.slice(7);
    out = bundle.colorsLight[key] ?? out;
  } else if (out.startsWith("@dimen/")) {
    const key = out.slice(7);
    out = dimensionToPx(bundle.dimens[key] ?? out);
  } else if (out.startsWith("@string/")) {
    const key = out.slice(8);
    out = bundle.strings[key] ?? out;
  } else if (/^-?\d+(\.\d+)?(dp|dip|sp|px|pt)$/.test(out)) {
    out = dimensionToPx(out);
  }
  return out;
}
