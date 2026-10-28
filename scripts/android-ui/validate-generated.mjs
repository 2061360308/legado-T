import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { APP_RES, OUT_DESIGN, writeFile } from "./android.mjs";

const GENERATED = path.join(OUT_DESIGN, "generated");
const REPORTS = path.join(OUT_DESIGN, "reports");
const manifest = JSON.parse(fs.readFileSync(path.join(GENERATED, "manifest.json"), "utf8"));
const report = JSON.parse(fs.readFileSync(path.join(REPORTS, "generate-report.json"), "utf8"));

const sourceFiles = [];
for (const dir of ["layout", "layout-land", "menu", "xml"]) {
  const full = path.join(APP_RES, dir);
  if (!fs.existsSync(full)) continue;
  for (const file of fs.readdirSync(full).filter((item) => item.endsWith(".xml")).sort()) {
    sourceFiles.push(`${dir}/${file}`);
  }
}

const failures = [];
const warnings = [];
const allowedStatus = new Set(["mapped", "ignored", "unsupported", "unresolved"]);

function fail(message) {
  failures.push(message);
}

if (sourceFiles.length !== 286) {
  fail(`source XML count mismatch: ${sourceFiles.length} != 286`);
}

const manifestSources = manifest.map((item) => item.source);
const uniqueManifestSources = new Set(manifestSources);
if (manifestSources.length !== uniqueManifestSources.size) {
  fail("manifest contains duplicate source entries");
}
for (const source of sourceFiles) {
  if (!uniqueManifestSources.has(source)) {
    fail(`manifest missing source: ${source}`);
  }
}
for (const source of uniqueManifestSources) {
  if (!sourceFiles.includes(source)) {
    fail(`manifest contains unknown source: ${source}`);
  }
}

const generatedEntries = manifest.filter((item) => item.status === "generated");
const notApplicable = manifest.filter((item) => item.status === "not-applicable");
if (generatedEntries.length !== 283) {
  fail(`generated component count mismatch: ${generatedEntries.length} != 283`);
}
if (notApplicable.length !== 3) {
  fail(`not-applicable count mismatch: ${notApplicable.length} != 3`);
}

for (const entry of generatedEntries) {
  const sourcePath = path.join(APP_RES, entry.source);
  if (!fs.existsSync(sourcePath)) {
    fail(`${entry.source}: source missing`);
    continue;
  }
  const vuePath = path.join(GENERATED, entry.kind, `${entry.name}.vue`);
  const metaPath = path.join(GENERATED, entry.kind, `${entry.name}.meta.json`);
  if (!fs.existsSync(vuePath)) {
    fail(`${entry.source}: generated .vue missing`);
    continue;
  }
  if (!fs.existsSync(metaPath)) {
    fail(`${entry.source}: generated .meta.json missing`);
    continue;
  }
  const vue = fs.readFileSync(vuePath, "utf8");
  const meta = JSON.parse(fs.readFileSync(metaPath, "utf8"));
  if (!vue.includes("<template>") || !vue.includes("<script setup")) {
    fail(`${entry.source}: generated Vue SFC structure invalid`);
  }
  const hash = crypto.createHash("sha1").update(fs.readFileSync(sourcePath)).digest("hex");
  if (meta.hash !== hash) {
    fail(`${entry.source}: source hash mismatch`);
  }
  if (meta.nodeCount !== entry.nodeCount) {
    fail(`${entry.source}: manifest node count mismatch`);
  }
  if (meta.attrCount !== entry.attrCount) {
    fail(`${entry.source}: manifest attr count mismatch`);
  }
  if (!Array.isArray(meta.meta)) {
    fail(`${entry.source}: attr ledger missing`);
    continue;
  }
  if (meta.meta.length !== meta.attrCount) {
    fail(
      `${entry.source}: attr ledger count mismatch: ${meta.meta.length} != ${meta.attrCount}`,
    );
  }
  for (const attr of meta.meta) {
    if (!allowedStatus.has(attr.status)) {
      fail(`${entry.source}: invalid attr status ${attr.status} for ${attr.name}`);
    }
    if (attr.status === "unresolved") {
      fail(`${entry.source}: unresolved attr ${attr.name}: ${attr.reason ?? ""}`);
    }
  }
  const expandedMeta = Array.isArray(meta.expandedMeta) ? meta.expandedMeta : [];
  for (const attr of expandedMeta) {
    if (!allowedStatus.has(attr.status)) {
      fail(`${entry.source}: invalid expanded attr status ${attr.status} for ${attr.name}`);
    }
    if (attr.status === "unresolved") {
      fail(`${entry.source}: unresolved expanded attr ${attr.name}: ${attr.reason ?? ""}`);
    }
  }
  if (meta.unresolved?.length) {
    fail(`${entry.source}: unresolved list is not empty`);
  }
  if (meta.unsupported?.length) {
    warnings.push(`${entry.source}: unsupported attrs: ${meta.unsupported.map((item) => item.name).join(", ")}`);
  }
}

const vueFiles = [];
for (const dir of ["layout", "layout-land", "menu", "xml"]) {
  const full = path.join(GENERATED, dir);
  if (!fs.existsSync(full)) continue;
  for (const file of fs.readdirSync(full).filter((item) => item.endsWith(".vue"))) {
    vueFiles.push(`${dir}/${file}`);
  }
}
if (vueFiles.length !== 283) {
  fail(`generated Vue file count mismatch: ${vueFiles.length} != 283`);
}

const routerFile = path.join(process.cwd(), "src/router/index.ts");
if (!fs.existsSync(routerFile)) {
  fail("router file missing");
} else {
  const router = fs.readFileSync(routerFile, "utf8");
  if (!router.includes('path: "/generated/:kind/:name"')) {
    fail("generated route is not registered");
  }
  if (!router.includes('path: "/generated"')) {
    fail("generated gallery route is not registered");
  }
  if (!router.includes('path: "/shell/:name"')) {
    fail("shell route is not registered");
  }
}

const navigationFile = path.join(GENERATED, "../reports/navigation-graph.json");
let navigationStats = null;
if (!fs.existsSync(navigationFile)) {
  fail("navigation graph is missing (run extract-navigation.mjs)");
} else {
  const graph = JSON.parse(fs.readFileSync(navigationFile, "utf8"));
  const nodeIds = new Set(graph.nodes.map((node) => node.id));
  if (!graph.start || !nodeIds.has(graph.start)) {
    fail("navigation graph has no resolvable start node");
  }
  if (!Array.isArray(graph.edges) || graph.edges.length === 0) {
    fail("navigation graph has no edges");
  }
  for (const edge of graph.edges) {
    if (!edge.from || !nodeIds.has(edge.from)) {
      fail(`navigation edge has unknown source: ${edge.fromClass} -> ${edge.toClass}`);
    }
    if (!edge.target || !nodeIds.has(edge.target)) {
      fail(`navigation edge has unknown target: ${edge.fromClass} -> ${edge.toClass}`);
    }
    if (graph.nodes.some((node) => node.id === edge.target && node.route !== edge.targetRoute)) {
      fail(`navigation edge route mismatch: ${edge.fromClass} -> ${edge.toClass}`);
    }
  }
  navigationStats = {
    nodes: graph.nodes.length,
    edges: graph.edges.length,
    start: graph.start,
    activitiesWithoutLayout: graph.unresolved?.activitiesWithoutLayout?.length ?? 0,
    edgesWithoutTarget: graph.unresolved?.edgesWithoutTarget?.length ?? 0,
  };
}

const prefsIndexFile = path.join(OUT_DESIGN, "prefs/index.json");
let prefStats = null;
if (!fs.existsSync(prefsIndexFile)) {
  fail("prefs index missing (run parse-prefs.mjs)");
} else {
  const prefIndex = JSON.parse(fs.readFileSync(prefsIndexFile, "utf8"));
  const empty = [];
  let prefScreens = 0;
  for (const entry of prefIndex) {
    const file = path.join(OUT_DESIGN, "prefs", `${entry.name}.json`);
    if (!fs.existsSync(file)) {
      fail(`pref file missing: ${entry.name}`);
      continue;
    }
    const tree = JSON.parse(fs.readFileSync(file, "utf8"));
    if (tree.tag !== "PreferenceScreen") continue;
    prefScreens += 1;
    if (!Array.isArray(tree.children) || tree.children.length === 0) {
      empty.push(entry.name);
    }
  }
  if (empty.length) {
    fail(`pref trees empty (parser regression?): ${empty.join(", ")}`);
  }
  if (prefScreens !== 9) {
    fail(`PreferenceScreen count mismatch: ${prefScreens} != 9`);
  }
  prefStats = { files: prefIndex.length, screens: prefScreens, empty: empty.length };
}

const summary = {
  generatedAt: new Date().toISOString(),
  sourceXmlCount: sourceFiles.length,
  manifestCount: manifest.length,
  generatedCount: generatedEntries.length,
  notApplicableCount: notApplicable.length,
  generatedVueCount: vueFiles.length,
  navigation: navigationStats,
  prefs: prefStats,
  unresolvedCount: failures.filter((item) => item.includes("unresolved")).length,
  failureCount: failures.length,
  warningCount: warnings.length,
  failures,
  warnings,
};

writeFile(path.join(REPORTS, "validate-report.json"), JSON.stringify(summary, null, 2));
console.log(
  `source=${sourceFiles.length}, generated=${generatedEntries.length}, vue=${vueFiles.length}, failures=${failures.length}, warnings=${warnings.length}`,
);
if (failures.length) {
  process.exitCode = 1;
}
