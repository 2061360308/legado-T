import fs from "node:fs";
import path from "node:path";
import { APP_RES, attr, children, loadResourceBundle, readXml, writeFile } from "./android.mjs";
import { KT_ROOT, REPORTS, createKtIndex, loadMenuTitles } from "./kt-scan.mjs";

const bundle = loadResourceBundle();

// ---- manifest activities ----

const manifestDoc = readXml(path.join(APP_RES, "../AndroidManifest.xml"));
const manifestActivities = children(manifestDoc.manifest.application, "activity");
const activityClasses = [];
const launcherClasses = [];
for (const activity of manifestActivities) {
  let name = attr(activity, "name");
  if (!name) continue;
  if (name.startsWith(".")) name = `io.legado.app${name}`;
  activityClasses.push(name);
  const filters = children(activity, "intent-filter");
  for (const filter of filters) {
    const actions = children(filter, "action");
    const categories = children(filter, "category");
    if (
      actions.some((a) => attr(a, "name") === "android.intent.action.MAIN") &&
      categories.some((c) => attr(c, "name") === "android.intent.category.LAUNCHER")
    ) {
      launcherClasses.push(name);
    }
  }
}

// ---- kt scan: class -> binding ----

const { ktFiles, fileClass, resolveBinding, layoutForClass } = createKtIndex();

// ---- menu id -> title ----

const menuTitles = loadMenuTitles(bundle);

// ---- edge extraction (line state machine) ----

const edges = [];
const edgeKeys = new Set();
function addEdge(fromClass, toClass, kind, trigger, triggerTitle, file, line) {
  const key = `${fromClass}|${toClass}|${trigger}`;
  if (edgeKeys.has(key)) return;
  edgeKeys.add(key);
  edges.push({ fromClass, toClass, kind, trigger, triggerTitle: triggerTitle || titleForTrigger(trigger) || toClass, source: `${path.relative(KT_ROOT, file).replace(/\\/g, "/")}:${line}` });
}

function titleForTrigger(trigger) {
  if (!trigger) return "";
  const id = trigger.replace(/^@id\//, "");
  return menuTitles.get(id) ?? trigger;
}

// MainActivity 底部导航：菜单 id → 主界面 fragment（源码 onNavigationItemSelected/TabFragmentPageAdapter 语义）
const MAIN_TAB_FRAGMENTS = [
  ["menu_bookshelf", "BookshelfFragment1"],
  ["menu_discovery", "ExploreFragment"],
  ["menu_rss", "RssFragment"],
  ["menu_my_config", "MyFragment"],
];

for (const file of ktFiles) {
  const fromClass = fileClass.get(file);
  if (!/(Activity|Fragment|Dialog|Menu)$/.test(fromClass)) continue;
  const text = fs.readFileSync(file, "utf8");
  const lines = text.split(/\r?\n/);
  let trigger = "";
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    if (/^\s*}\s*(\/\/.*)?$/.test(line)) {
      trigger = "";
      continue;
    }
    const idMatch = line.match(/R\.id\.(\w+)\s*->/);
    const keyMatch = line.match(/"([\w]+)"\s*->/);
    if (idMatch) trigger = `@id/${idMatch[1]}`;
    else if (keyMatch) trigger = keyMatch[1];
    const navMatch = line.match(/startActivity<(\w+)>/);
    if (navMatch) {
      addEdge(fromClass, navMatch[1], "activity", trigger, titleForTrigger(trigger), file, i + 1);
      trigger = "";
      continue;
    }
    const bookMatch = line.match(/startActivityForBook\(/);
    if (bookMatch) {
      addEdge(fromClass, "ReadBookActivity", "activity", trigger, trigger ? titleForTrigger(trigger) : "打开书籍", file, i + 1);
      trigger = "";
      continue;
    }
    const dialogGeneric = line.match(/showDialogFragment<(\w+)>/);
    if (dialogGeneric) {
      addEdge(fromClass, dialogGeneric[1], "dialog", trigger, titleForTrigger(trigger), file, i + 1);
      trigger = "";
      continue;
    }
    const dialogCall = line.match(/showDialogFragment\(\s*(\w+)\(/);
    if (dialogCall) {
      addEdge(fromClass, dialogCall[1], "dialog", trigger, titleForTrigger(trigger), file, i + 1);
      trigger = "";
      continue;
    }
    const dialogShow = line.match(/(\w+Dialog)\(\)\.show\(/);
    if (dialogShow) {
      addEdge(fromClass, dialogShow[1], "dialog", trigger, titleForTrigger(trigger), file, i + 1);
      trigger = "";
    }
  }
}

for (const [menuId, fragmentClass] of MAIN_TAB_FRAGMENTS) {
  const mainFile = ktFiles.find((f) => fileClass.get(f) === "MainActivity");
  if (mainFile) {
    addEdge("MainActivity", fragmentClass, "fragment", `@id/${menuId}`, titleForTrigger(`@id/${menuId}`), mainFile, 0);
  }
}

// ---- nodes ----

const nodeClasses = new Set(activityClasses.map((full) => full.split(".").pop()));
for (const edge of edges) {
  nodeClasses.add(edge.fromClass);
  nodeClasses.add(edge.toClass);
}
const nodes = [];
const activitiesWithoutLayout = [];
const nodeByLayout = new Map();
const manifestSimple = new Set(activityClasses.map((full) => full.split(".").pop()));
const sortedClasses = [...nodeClasses].sort(
  (a, b) => Number(manifestSimple.has(b)) - Number(manifestSimple.has(a)) || a.localeCompare(b),
);
for (const className of sortedClasses) {
  const layout = layoutForClass(className);
  if (layout) {
    if (nodeByLayout.has(layout)) continue;
    const kind = /Dialog$/.test(className) ? "dialog" : /Fragment$/.test(className) ? "fragment" : "activity";
    const node = { id: layout, class: className, kind, route: `/shell/${layout}` };
    nodeByLayout.set(layout, node);
    nodes.push(node);
  } else if (activityClasses.some((full) => full.split(".").pop() === className)) {
    activitiesWithoutLayout.push(className);
  }
}
const nodeIds = new Set(nodes.map((node) => node.id));

const resolvedEdges = [];
const edgesWithoutTarget = [];
for (const edge of edges) {
  const fromId = layoutForClass(edge.fromClass);
  const toId = layoutForClass(edge.toClass);
  const enriched = {
    ...edge,
    triggerTitle: edge.triggerTitle || edge.trigger,
    from: fromId,
    target: toId,
    targetRoute: toId ? `/shell/${toId}` : null,
  };
  if (fromId && toId) resolvedEdges.push(enriched);
  else if (fromId) edgesWithoutTarget.push(enriched);
}

const launcherClass =
  launcherClasses.find((name) => layoutForClass(name.split(".").pop())) ??
  launcherClasses[0] ??
  null;
const startLayout = launcherClass ? layoutForClass(launcherClass.split(".").pop()) : null;
const graph = {
  generatedAt: new Date().toISOString(),
  start: startLayout,
  launcherClass,
  nodes,
  edges: resolvedEdges,
  stats: {
    manifestActivities: activityClasses.length,
    nodes: nodes.length,
    edges: resolvedEdges.length,
    dialogEdges: resolvedEdges.filter((e) => e.kind === "dialog").length,
  },
  unresolved: {
    activitiesWithoutLayout,
    edgesWithoutTarget,
  },
};

writeFile(path.join(REPORTS, "navigation-graph.json"), JSON.stringify(graph, null, 2));
console.log(
  `navigation: nodes=${nodes.length}, edges=${resolvedEdges.length}, activities-without-layout=${activitiesWithoutLayout.length}, edges-without-target=${edgesWithoutTarget.length}, start=${startLayout ?? "none"}`,
);
if (!startLayout || resolvedEdges.length === 0) {
  process.exitCode = 1;
}
