import fs from "node:fs";
import path from "node:path";
import { APP_RES, OUT_DESIGN, ROOT, attr, children, readXml } from "./android.mjs";

export const KT_ROOT = path.join(ROOT, "app/src/main/java");
export const GENERATED_LAYOUT_DIR = path.join(OUT_DESIGN, "generated/layout");
export const REPORTS = path.join(OUT_DESIGN, "reports");

export function snake(name) {
  return name
    .replace(/([a-z0-9])([A-Z])/g, "$1_$2")
    .replace(/([A-Z])([A-Z][a-z])/g, "$1_$2")
    .toLowerCase();
}

export function layoutExists(name) {
  return fs.existsSync(path.join(GENERATED_LAYOUT_DIR, `${name}.vue`));
}

const BINDING_PATTERNS = [
  [/viewBinding\(\s*(\w+Binding)\s*::\s*(?:bind|inflate)/g, 10],
  [/<\s*(\w+Binding)\s*>/g, 5],
  [/:\s*(\w+Binding)\b/g, 5],
  [/(\w+Binding)\.inflate\(/g, 3],
  [/\b(\w+Binding)\b/g, 1],
];

// 自身布局探测：viewBinding()/R.layout/泛型 > inflate > 类型声明。
// item_* 为 adapter 条目布局，不能作为类自身布局。
function detectOwnLayout(text) {
  const scores = new Map();
  for (const [regex, weight] of BINDING_PATTERNS) {
    const seen = new Set();
    for (const match of text.matchAll(regex)) {
      const raw = match[1];
      if (!raw) continue;
      const layout = raw.endsWith("Binding")
        ? snake(raw.replace(/Binding$/, ""))
        : raw;
      if (!layout || layout.startsWith("item_") || seen.has(layout)) continue;
      seen.add(layout);
      if (!layoutExists(layout)) continue;
      scores.set(layout, (scores.get(layout) ?? 0) + weight);
    }
  }
  let best = null;
  let bestScore = 0;
  for (const [layout, score] of scores) {
    if (score > bestScore) {
      best = layout;
      bestScore = score;
    }
  }
  return { layout: best, score: bestScore };
}

export function createKtIndex() {
  const ktFiles = [];
  (function walk(dir) {
    for (const entry of fs.readdirSync(dir, { withFileTypes: true })) {
      const full = path.join(dir, entry.name);
      if (entry.isDirectory()) walk(full);
      else if (entry.name.endsWith(".kt")) ktFiles.push(full);
    }
  })(KT_ROOT);

  const fileClass = new Map();
  const classFile = new Map();
  const classOwn = new Map();
  const classOwnScore = new Map();
  const classParent = new Map();

  for (const file of ktFiles) {
    const className = path.basename(file, ".kt");
    fileClass.set(file, className);
    classFile.set(className, file);
    const text = fs.readFileSync(file, "utf8");
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
      const decl = lines[i].match(/(?:open |abstract |sealed |internal )*class (\w+)/);
      if (!decl) continue;
      const header = lines.slice(i, i + 6).join(" ");
      const superMatch = header.slice(header.indexOf(decl[0]) + decl[0].length).match(/:\s*([\w.]+)/);
      if (superMatch) {
        const parent = superMatch[1].split(".").pop();
        if (parent && parent !== decl[1]) classParent.set(decl[1], parent);
      }
      break;
    }
    const own = detectOwnLayout(text);
    if (own.layout) {
      classOwn.set(className, own.layout);
      classOwnScore.set(className, own.score);
    }
  }

  const resolveCache = new Map();
  function resolveLayout(className, seen = new Set()) {
    if (resolveCache.has(className)) return resolveCache.get(className);
    if (seen.has(className)) return null;
    seen.add(className);
    const own = classOwn.get(className) ?? null;
    const ownScore = classOwnScore.get(className) ?? 0;
    const parent = classParent.get(className);
    const parentLayout = parent ? resolveLayout(parent, seen) : null;
    // 继承的显式绑定（viewBinding/泛型/类型）优先于子类中 inflate 的临时弹窗布局
    let result;
    if (parentLayout && ownScore < 5) result = parentLayout;
    else result = own ?? parentLayout;
    resolveCache.set(className, result);
    return result;
  }

  function layoutForClass(className) {
    const resolved = resolveLayout(className);
    if (resolved) return resolved;
    const kindMatch = className.match(/(Activity|Dialog|Fragment)$/);
    if (!kindMatch) return null;
    const kind = kindMatch[1].toLowerCase();
    const layout = `${kind}_${snake(className.replace(/(Activity|Dialog|Fragment)$/, ""))}`;
    return layoutExists(layout) ? layout : null;
  }

  return { ktFiles, fileClass, classFile, classOwn, resolveLayout, layoutForClass };
}

export function loadMenuTitles(bundle) {
  const menuTitles = new Map();
  const menuDir = path.join(APP_RES, "menu");
  function collectMenuItems(items) {
    for (const item of items) {
      const id = attr(item, "id");
      const title = attr(item, "title");
      if (id && title) {
        const key = String(id).replace(/^@\+?id\//, "");
        const titleValue = String(title).startsWith("@string/")
          ? String(bundle.strings[String(title).slice(8)] ?? title)
          : String(title);
        menuTitles.set(key, titleValue);
      }
      collectMenuItems(children(item, "menu").flatMap((menu) => children(menu, "item")));
    }
  }
  if (fs.existsSync(menuDir)) {
    for (const file of fs.readdirSync(menuDir).filter((f) => f.endsWith(".xml")).sort()) {
      const doc = readXml(path.join(menuDir, file));
      const menuNode = Array.isArray(doc.menu) ? doc.menu[0] : doc.menu;
      if (menuNode) collectMenuItems(children(menuNode, "item"));
    }
  }
  return menuTitles;
}
