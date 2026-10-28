import fs from "node:fs";
import path from "node:path";
import { APP_RES, ROOT, writeFile } from "./android.mjs";

const groups = [
  { dir: "layout", label: "layout" },
  { dir: "layout-land", label: "layout-land" },
  { dir: "menu", label: "menu" },
  { dir: "xml", label: "xml" },
];

const statusFile = path.join(ROOT, "local-reader/scripts/android-ui/inventory-status.json");
const statusMap = fs.existsSync(statusFile)
  ? JSON.parse(fs.readFileSync(statusFile, "utf8"))
  : {};

const rows = [];
let index = 1;
for (const group of groups) {
  const dir = path.join(APP_RES, group.dir);
  if (!fs.existsSync(dir)) continue;
  const files = fs
    .readdirSync(dir)
    .filter((name) => name.endsWith(".xml"))
    .sort((a, b) => a.localeCompare(b));
  for (const file of files) {
    const key = `${group.dir}/${file}`;
    const meta = statusMap[key] ?? {};
    const generatedTarget = `design/generated/${group.dir}/${file.replace(/\.xml$/, ".vue")}`;
    const shell = { target: generatedTarget, status: "壳子已生成" };
    const nonVisual = [
      "xml/file_paths.xml",
      "xml/network_security_config.xml",
      "xml/spen_remote_actions.xml",
    ].includes(key);
    rows.push({
      index: index++,
      file: key,
      status: meta.status ?? (nonVisual ? "不适用" : shell.status),
      target: meta.target ?? (nonVisual ? "" : shell.target),
      note: meta.note ?? "",
    });
  }
}

const counts = groups.map((group) => {
  const dir = path.join(APP_RES, group.dir);
  const count = fs.existsSync(dir)
    ? fs.readdirSync(dir).filter((name) => name.endsWith(".xml")).length
    : 0;
  return `- ${group.label}: ${count}`;
});

const md = `# Android UI 全量复刻清单

> 生成目录：${path.relative(ROOT, APP_RES).replace(/\\/g, "/")}
> 排序规则：layout → layout-land → menu → xml，组内按文件名升序。
> 状态：待复刻 / 进行中 / 已完成 / 视觉禁用 / 不适用。

${counts.join("\n")}

| # | XML | 目标 Vue/组件 | 状态 | 备注 |
|---:|---|---|---|---|
${rows
  .map(
    (row) =>
      `| ${row.index} | \`${row.file}\` | ${row.target} | ${row.status} | ${row.note} |`,
  )
  .join("\n")}
`;

writeFile(path.join(ROOT, "local-reader/docs/ui-port/00-inventory.md"), md);
writeFile(
  path.join(ROOT, "local-reader/docs/ui-port/00-inventory.json"),
  JSON.stringify(rows, null, 2),
);
console.log(`inventory: ${rows.length} xml files`);
