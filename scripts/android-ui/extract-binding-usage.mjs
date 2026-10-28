import fs from "node:fs";
import path from "node:path";
import { OUT_DESIGN, writeFile } from "./android.mjs";
import { REPORTS, createKtIndex, snake } from "./kt-scan.mjs";

const index = createKtIndex();
const LAYOUT_DIRS = ["layout", "layout-land"].map((dir) =>
  path.join(OUT_DESIGN, "layouts", dir),
);

const layoutIdCache = new Map();
function layoutIds(layout) {
  if (layoutIdCache.has(layout)) return layoutIdCache.get(layout);
  for (const dir of LAYOUT_DIRS) {
    const file = path.join(dir, `${layout}.json`);
    if (!fs.existsSync(file)) continue;
    const ids = [];
    const walk = (node) => {
      const id = node?.attrs?.id;
      if (id && !ids.includes(id)) ids.push(id);
      for (const child of node?.children ?? []) walk(child);
    };
    walk(JSON.parse(fs.readFileSync(file, "utf8")));
    layoutIdCache.set(layout, ids);
    return ids;
  }
  layoutIdCache.set(layout, null);
  return null;
}

function uniqueMatches(text, regex) {
  const out = [];
  for (const match of text.matchAll(regex)) {
    const value = match[1];
    if (value && !out.includes(value)) out.push(value);
  }
  return out;
}

// ViewBinding 生成的 root / Kotlin 作用域函数不是布局 id
const NON_ID_BINDING = new Set(["root", "run", "apply", "let", "also", "with"]);

const stats = {
  ktFiles: index.ktFiles.length,
  classesWithLayout: 0,
  bindingRefs: 0,
  menuReferences: 0,
  clickBindings: 0,
  unmatchedRefs: 0,
};
const layouts = {};
const menus = {};
const unmatched = [];

for (const file of index.ktFiles) {
  const className = index.fileClass.get(file);
  if (!/(Activity|Fragment|Dialog|View|Menu)$/.test(className)) continue;
  const layout = index.layoutForClass(className);
  if (!layout) continue;
  stats.classesWithLayout += 1;

  const text = fs.readFileSync(file, "utf8");
  const refs = uniqueMatches(text, /\bbinding\.(\w+)/g);
  const clickRefs = uniqueMatches(
    text,
    /\bbinding\.(\w+)\.setOn\w*(?:ClickListener|Listener)/g,
  );
  const menuRefs = uniqueMatches(text, /R\.menu\.(\w+)/g);
  const ids = layoutIds(layout) ?? [];

  const resolved = [];
  const refUnmatched = [];
  for (const ref of refs) {
    if (NON_ID_BINDING.has(ref)) continue;
    const id = snake(ref);
    if (ids.includes(id)) resolved.push(id);
    else refUnmatched.push(id);
  }
  const clickIds = clickRefs.filter((ref) => !NON_ID_BINDING.has(ref)).map((ref) => snake(ref));

  stats.bindingRefs += resolved.length;
  stats.menuReferences += menuRefs.length;
  stats.clickBindings += clickIds.length;
  stats.unmatchedRefs += refUnmatched.length;
  if (refUnmatched.length) {
    unmatched.push({
      class: className,
      layout,
      refs: refUnmatched,
    });
  }

  const entry = layouts[layout] ?? {
    layout,
    classes: [],
    refs: [],
    clickIds: [],
  };
  if (!entry.classes.includes(className)) entry.classes.push(className);
  for (const id of resolved) if (!entry.refs.includes(id)) entry.refs.push(id);
  for (const id of clickIds) if (!entry.clickIds.includes(id)) entry.clickIds.push(id);
  layouts[layout] = entry;

  for (const menu of menuRefs) {
    const menuEntry = menus[menu] ?? { menu, classes: [], sources: [] };
    if (!menuEntry.classes.includes(className)) menuEntry.classes.push(className);
    const source = `${path.relative(path.join(OUT_DESIGN, "../../.."), file).replace(/\\/g, "/")}`;
    if (!menuEntry.sources.includes(source)) menuEntry.sources.push(source);
    menus[menu] = menuEntry;
  }
}

const report = {
  generatedAt: new Date().toISOString(),
  stats,
  layouts,
  menus,
  unmatched,
};

writeFile(path.join(REPORTS, "binding-usage.json"), JSON.stringify(report, null, 2));
console.log(
  `binding-usage: kt=${stats.ktFiles}, classes-with-layout=${stats.classesWithLayout}, layouts=${Object.keys(layouts).length}, menus=${Object.keys(menus).length}, refs=${stats.bindingRefs}, click=${stats.clickBindings}, unmatched=${stats.unmatchedRefs}`,
);
if (stats.unmatchedRefs > 0) {
  console.log("  unmatched refs (not layout ids):");
  for (const item of unmatched.slice(0, 10)) {
    console.log(`  - ${item.class} (${item.layout}): ${item.refs.join(", ")}`);
  }
}
