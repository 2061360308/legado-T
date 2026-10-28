import fs from "node:fs";
import path from "node:path";
import {
  APP_RES,
  OUT_DESIGN,
  attr,
  dimensionToPx,
  loadResourceBundle,
  readXml,
  resolveAttrValue,
  writeFile,
} from "./android.mjs";

const bundle = loadResourceBundle();
const OUT_LAYOUT = path.join(OUT_DESIGN, "layouts");

const GROUPS = [
  { dir: "layout", out: "layout" },
  { dir: "layout-land", out: "layout-land" },
];

const KEEP = [
  "id",
  "text",
  "hint",
  "title",
  "subtitle",
  "textSize",
  "textColor",
  "textStyle",
  "background",
  "orientation",
  "gravity",
  "layout_width",
  "layout_height",
  "layout_weight",
  "layout_margin",
  "layout_marginLeft",
  "layout_marginTop",
  "layout_marginRight",
  "layout_marginBottom",
  "layout_marginStart",
  "layout_marginEnd",
  "padding",
  "paddingLeft",
  "paddingTop",
  "paddingRight",
  "paddingBottom",
  "paddingStart",
  "paddingEnd",
  "layout_constraintTop_toTopOf",
  "layout_constraintTop_toBottomOf",
  "layout_constraintBottom_toTopOf",
  "layout_constraintBottom_toBottomOf",
  "layout_constraintLeft_toLeftOf",
  "layout_constraintLeft_toRightOf",
  "layout_constraintRight_toLeftOf",
  "layout_constraintRight_toRightOf",
  "layout_constraintStart_toStartOf",
  "layout_constraintStart_toEndOf",
  "layout_constraintEnd_toStartOf",
  "layout_constraintEnd_toEndOf",
  "layout_constraintHorizontal_bias",
  "layout_constraintVertical_bias",
  "layout_constraintDimensionRatio",
  "layout_constraintWidth_percent",
  "layout_constraintHeight_percent",
  "src",
  "tint",
  "visibility",
  "maxLines",
  "lines",
  "singleLine",
  "ellipsize",
  "scaleType",
  "minHeight",
  "contentDescription",
  "style",
  "checked",
  "layout_behavior",
  "layout_span",
  "layout_row",
  "layout_column",
  "layout_columnSpan",
  "layout_columnWeight",
  "layout_gravity",
  "layout_centerInParent",
  "layout_alignParentTop",
  "layout_alignParentBottom",
  "layout_alignParentLeft",
  "layout_alignParentRight",
  "layout_alignParentStart",
  "layout_alignParentEnd",
  "layout_toRightOf",
  "layout_toLeftOf",
  "layout_below",
  "layout_above",
  "layout_alignTop",
  "layout_alignBottom",
  "layout_alignLeft",
  "layout_alignRight",
  "layout_alignStart",
  "layout_alignEnd",
  "layout_alignBaseline",
  "layout_centerHorizontal",
  "layout_centerVertical",
];

function normalizeValue(key, value) {
  if (value === undefined || value === null) return undefined;
  if (key === "id") return String(value).replace("@+id/", "").replace("@id/", "");
  if (key === "layout_width" || key === "layout_height") {
    if (value === "match_parent" || value === "wrap_content") return value;
    const px = dimensionToPx(value);
    return typeof px === "number" ? px : value;
  }
  if (
    key.startsWith("layout_margin") ||
    key.startsWith("padding") ||
    key === "textSize" ||
    key === "minHeight"
  ) {
    const px = dimensionToPx(value);
    return typeof px === "number" ? px : resolveAttrValue(value, bundle);
  }
  if (key === "src" || key === "background") {
    if (typeof value === "string" && value.startsWith("@drawable/")) {
      return value.slice(10);
    }
    return value;
  }
  return resolveAttrValue(value, bundle);
}

function tagOf(name) {
  return name.split(".").pop();
}

function normalizeNode(node, name) {
  const attrs = {};
  for (const key of Object.keys(node)) {
    if (!key.startsWith("@_")) continue;
    const raw = key.replace("@_android:", "").replace("@_app:", "").replace("@_", "");
    if (!KEEP.includes(raw)) continue;
    const value = normalizeValue(raw, node[key]);
    if (value !== undefined) attrs[raw] = value;
  }
  const children = [];
  for (const [key, value] of Object.entries(node)) {
    if (key.startsWith("@_") || key.startsWith("#") || key.startsWith("?")) continue;
    const list = Array.isArray(value) ? value : [value];
    for (const child of list) {
      if (typeof child !== "object" || child === null) continue;
      children.push(normalizeNode(child, key));
    }
  }
  return { tag: tagOf(name), attrs, children };
}

function parseLayout(file) {
  const doc = readXml(file);
  const rootKey = Object.keys(doc).find((key) => !key.startsWith("?"));
  const rootRaw = doc[rootKey];
  const root = Array.isArray(rootRaw) ? rootRaw[0] : rootRaw;
  return normalizeNode(root, rootKey);
}

let count = 0;
const index = [];
for (const group of GROUPS) {
  const dir = path.join(APP_RES, group.dir);
  if (!fs.existsSync(dir)) continue;
  const files = fs
    .readdirSync(dir)
    .filter((name) => name.endsWith(".xml"))
    .sort((a, b) => a.localeCompare(b));
  for (const file of files) {
    const name = path.basename(file, ".xml");
    const tree = parseLayout(path.join(dir, file));
    writeFile(
      path.join(OUT_LAYOUT, group.out, `${name}.json`),
      JSON.stringify(tree, null, 2),
    );
    index.push({ group: group.out, name, file: `${group.dir}/${file}` });
    count += 1;
  }
}
writeFile(path.join(OUT_LAYOUT, "index.json"), JSON.stringify(index, null, 2));
console.log(`parsed ${count} layouts`);
