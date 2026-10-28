import fs from "node:fs";
import path from "node:path";
import { OUT_DESIGN, writeFile } from "./android.mjs";
import { REPORTS } from "./kt-scan.mjs";

const APP_WEB = path.join(OUT_DESIGN, "../..");
const contractPath = new URL("./ui-contract.json", import.meta.url);
const contract = JSON.parse(fs.readFileSync(contractPath, "utf8"));
const LAYOUT_DIRS = ["layout", "layout-land"].map((dir) => path.join(OUT_DESIGN, "layouts", dir));
const MENU_DIR = path.join(OUT_DESIGN, "menus");

const failures = [];
const warnings = [];

function fail(message) {
  failures.push(message);
}

function warn(message) {
  warnings.push(message);
}

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

// 菜单项：按 XML 文档顺序展开；静态 visible=false 跳过
const menuCache = new Map();
function menuEntries(menu) {
  if (menuCache.has(menu)) return menuCache.get(menu);
  const file = path.join(MENU_DIR, `${menu}.json`);
  if (!fs.existsSync(file)) {
    menuCache.set(menu, null);
    return null;
  }
  const tree = JSON.parse(fs.readFileSync(file, "utf8"));
  const items = [];
  const walk = (node) => {
    for (const child of node?.children ?? []) {
      if (child.tag === "item") {
        const attrs = child.attrs ?? {};
        if (attrs.visible !== "false") {
          items.push({
            id: attrs.id ?? "",
            title: attrs.title ?? attrs.id ?? "",
            action: attrs.showAsAction ?? "never",
          });
        }
      }
      walk(child);
    }
  };
  walk(tree);
  menuCache.set(menu, items);
  return items;
}

function dataIdsOf(text) {
  return new Set([...text.matchAll(/data-id="([^"]+)"/g)].map((match) => match[1]));
}

function checkSequence(entryKey, targetText, menuKey, titles, optional) {
  let cursor = 0;
  for (const title of titles) {
    if (!title) continue;
    if (optional.has(title) && !targetText.includes(title)) continue;
    const index = targetText.indexOf(title, cursor);
    if (index === -1) {
      if (targetText.includes(title)) {
        fail(`${entryKey}: 菜单项顺序错误「${title}」（${menuKey}）`);
      } else {
        fail(`${entryKey}: 缺少菜单项「${title}」（${menuKey}）`);
      }
    } else {
      cursor = index + title.length;
    }
  }
}

function checkMenu(entryKey, targetText, menuKey, settings = {}) {
  const menu = menuKey.replace(/^menu\//, "").replace(/\.xml$/, "");
  const items = menuEntries(menu);
  if (!items) {
    fail(`${entryKey}: 源菜单不存在 ${menuKey}`);
    return;
  }
  for (const item of items) {
    if (item.action === "always" && item.title && !targetText.includes(item.title)) {
      fail(`${entryKey}: 缺少常驻菜单项「${item.title}」（${menuKey}）`);
    }
  }
  const neverTitles = items.filter((item) => item.action !== "always").map((item) => item.title);
  const modes = Array.isArray(settings.modes) ? settings.modes : null;
  if (modes) {
    for (const mode of modes) {
      const titles = Array.isArray(mode.requiredTitles) ? mode.requiredTitles : neverTitles;
      checkSequence(entryKey, targetText, menuKey, titles, new Set(mode.optionalTitles ?? []));
    }
    return;
  }
  checkSequence(entryKey, targetText, menuKey, neverTitles, new Set(settings.optionalTitles ?? []));
}

let bindingUsage = null;
const bindingFile = path.join(REPORTS, "binding-usage.json");
if (fs.existsSync(bindingFile)) {
  bindingUsage = JSON.parse(fs.readFileSync(bindingFile, "utf8"));
}

const entries = Object.entries(contract.views ?? {});
let idCount = 0;
let menuCount = 0;

for (const [key, entry] of entries) {
  const targetPath = path.join(APP_WEB, entry.target ?? "");
  if (!entry.target || !fs.existsSync(targetPath)) {
    fail(`${key}: 目标视图不存在 ${entry.target ?? "(未设置)"}`);
    continue;
  }
  const text = fs.readFileSync(targetPath, "utf8");

  if (key.startsWith("layout/")) {
    const layout = key.replace(/^layout\//, "").replace(/\.xml$/, "");
    const ids = layoutIds(layout);
    if (!ids) {
      fail(`${key}: 源布局不存在`);
      continue;
    }
    const dataIds = dataIdsOf(text);
    const required = new Set(entry.requiredIds ?? []);
    const exempt = new Map((entry.exemptIds ?? []).map((item) => [item.id, item.reason]));
    idCount += ids.length;

    for (const id of required) {
      if (!ids.includes(id)) fail(`${key}: requiredId 不在源布局中: ${id}`);
      if (exempt.has(id)) fail(`${key}: id 同时出现在 required/exempt: ${id}`);
      if (!dataIds.has(id)) fail(`${key}: 目标缺少 data-id="${id}"（${entry.target}）`);
    }
    for (const [id, reason] of exempt) {
      if (!ids.includes(id)) fail(`${key}: exemptId 不在源布局中: ${id}`);
      if (!String(reason ?? "").trim()) fail(`${key}: exemptId 缺少理由: ${id}`);
    }
    for (const id of ids) {
      if (!required.has(id) && !exempt.has(id)) {
        fail(`${key}: 未记账 id: ${id}（加入 requiredIds 或 exemptIds 并说明理由）`);
      }
    }

    const usage = bindingUsage?.layouts?.[layout];
    if (usage && Array.isArray(usage.refs)) {
      for (const [id] of exempt) {
        if (usage.refs.includes(id)) {
          warn(`${key}: Kotlin 使用但契约豁免了 ${id}（${(usage.classes ?? []).join(", ")}）`);
        }
      }
    }
    for (const menuKey of entry.menus ?? []) {
      checkMenu(key, text, menuKey, entry);
      menuCount += 1;
    }
  }

  if (key.startsWith("menu/")) {
    checkMenu(key, text, key, entry);
    menuCount += 1;
  }
}

const report = {
  generatedAt: new Date().toISOString(),
  entries: entries.length,
  checkedIds: idCount,
  checkedMenus: menuCount,
  failureCount: failures.length,
  warningCount: warnings.length,
  failures,
  warnings,
};
writeFile(path.join(REPORTS, "ui-contract-report.json"), JSON.stringify(report, null, 2));
console.log(
  `ui-contract: entries=${entries.length}, ids=${idCount}, menus=${menuCount}, failures=${failures.length}, warnings=${warnings.length}`,
);
for (const message of failures.slice(0, 20)) console.log(`  FAIL ${message}`);
for (const message of warnings.slice(0, 10)) console.log(`  WARN ${message}`);
if (failures.length) process.exitCode = 1;
