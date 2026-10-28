import fs from "node:fs";
import path from "node:path";
import {
  APP_RES,
  OUT_DRAWABLE,
  attr,
  children,
  ensureDir,
  loadResourceBundle,
  readXml,
  resolveRefs,
  writeFile,
} from "./android.mjs";

const DRAWABLE_DIR = path.join(APP_RES, "drawable");
const bundle = loadResourceBundle();
const written = new Set();

function color(value, fallback) {
  if (!value) return fallback;
  if (value.startsWith("@color/")) {
    return bundle.colorsLight[value.slice(7)] ?? fallback;
  }
  return value;
}

function opacity(value) {
  if (value === undefined) return undefined;
  const n = Number(value);
  return Number.isFinite(n) ? n : undefined;
}

function svgWrap(content, vw, vh, width, height) {
  return `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 ${vw} ${vh}" width="${width}" height="${height}" fill="none">${content}</svg>\n`;
}

function convertGroup(group) {
  const tx = Number(attr(group, "translateX") ?? 0);
  const ty = Number(attr(group, "translateY") ?? 0);
  const sx = Number(attr(group, "scaleX") ?? 1);
  const sy = Number(attr(group, "scaleY") ?? 1);
  const rotation = Number(attr(group, "rotation") ?? 0);
  const px = Number(attr(group, "pivotX") ?? 0);
  const py = Number(attr(group, "pivotY") ?? 0);
  const parts = [];
  if (tx || ty) parts.push(`translate(${tx} ${ty})`);
  if (rotation) parts.push(`rotate(${rotation} ${px} ${py})`);
  if (sx !== 1 || sy !== 1) parts.push(`scale(${sx} ${sy})`);
  const transform = parts.length ? ` transform="${parts.join(" ")}"` : "";
  const inner = renderNodes(group);
  return `<g${transform}>${inner}</g>`;
}

function renderNodes(node) {
  const out = [];
  for (const group of children(node, "group")) out.push(convertGroup(group));
  for (const clip of children(node, "clip-path")) {
    const d = attr(clip, "pathData");
    if (d) out.push(`<clipPath><path d="${d}"/></clipPath>`);
  }
  for (const p of children(node, "path")) out.push(convertPath(p));
  return out.join("");
}

function convertPath(p) {
  const d = attr(p, "pathData");
  if (!d) return "";
  const fillColor = color(attr(p, "fillColor"), "#000000");
  const strokeColor = color(attr(p, "strokeColor"), undefined);
  const fillAlpha = opacity(attr(p, "fillAlpha"));
  const strokeAlpha = opacity(attr(p, "strokeAlpha"));
  const strokeWidth = attr(p, "strokeWidth");
  const fillType = attr(p, "fillType");
  const transparent = /^#0{8}$/i.test(fillColor) || fillColor === "transparent";
  const attrs = [`d="${d.replace(/\s+/g, " ")}"`];
  attrs.push(`fill="${transparent ? "none" : "currentColor"}"`);
  if (fillAlpha !== undefined) attrs.push(`fill-opacity="${fillAlpha}"`);
  if (fillType === "evenOdd") attrs.push('fill-rule="evenodd"');
  if (strokeColor) {
    attrs.push('stroke="currentColor"');
    if (strokeWidth !== undefined) attrs.push(`stroke-width="${strokeWidth}"`);
    if (strokeAlpha !== undefined) attrs.push(`stroke-opacity="${strokeAlpha}"`);
  }
  return `<path ${attrs.join(" ")}/>`;
}

function convertVector(root, name) {
  const vw = attr(root, "viewportWidth") ?? 24;
  const vh = attr(root, "viewportHeight") ?? 24;
  const width = attr(root, "width") ?? `${vw}dp`;
  const height = attr(root, "height") ?? `${vh}dp`;
  const body = renderNodes(root);
  if (!body) return null;
  return svgWrap(body, vw, vh, String(width).replace("dp", ""), String(height).replace("dp", ""));
}

function convertShape(root, name) {
  const shape = attr(root, "shape") ?? "rectangle";
  const solid = children(root, "solid")[0];
  const stroke = children(root, "stroke")[0];
  const corners = children(root, "corners")[0];
  const fill = solid ? color(attr(solid, "color"), "#000000") : "none";
  const strokeColor = stroke ? color(attr(stroke, "color"), "none") : "none";
  const strokeWidth = stroke ? attr(stroke, "width") ?? 1 : 0;
  const radius = corners ? attr(corners, "radius") ?? 0 : 0;
  const fillValue = fill === "transparent" ? "none" : fill;
  let body = "";
  if (shape === "oval") {
    body = `<ellipse cx="50" cy="50" rx="48" ry="48" fill="${fillValue}" stroke="${strokeColor}" stroke-width="${strokeWidth}"/>`;
  } else if (shape === "line") {
    body = `<line x1="0" y1="50" x2="100" y2="50" stroke="${strokeColor}" stroke-width="${strokeWidth || 1}"/>`;
  } else {
    body = `<rect x="1" y="1" width="98" height="98" rx="${radius}" fill="${fillValue}" stroke="${strokeColor}" stroke-width="${strokeWidth}"/>`;
  }
  return svgWrap(body, 100, 100, 100, 100);
}

function resolveDrawableRef(ref) {
  if (!ref) return null;
  const name = ref.replace("@drawable/", "");
  return name;
}

function convertFile(name, variant = "") {
  const baseName = variant ? `${name}-${variant}` : name;
  if (written.has(baseName)) return [baseName];
  written.add(baseName);
  const xmlPath = path.join(DRAWABLE_DIR, `${name}.xml`);
  if (!fs.existsSync(xmlPath)) {
    for (const ext of ["png", "jpg", "jpeg", "webp"]) {
      const bitmap = path.join(DRAWABLE_DIR, `${name}.${ext}`);
      if (fs.existsSync(bitmap)) {
        ensureDir(OUT_DRAWABLE);
        fs.copyFileSync(bitmap, path.join(OUT_DRAWABLE, `${baseName}.${ext}`));
        return [baseName];
      }
    }
    return [];
  }
  const doc = readXml(xmlPath);
  const rootKey = Object.keys(doc).find((key) => !key.startsWith("?")) ?? Object.keys(doc)[0];
  const rootRaw = doc[rootKey];
  const root = Array.isArray(rootRaw) ? rootRaw[0] : rootRaw;
  const tag = rootKey.split(".").pop();
  if (tag === "vector") {
    const svg = convertVector(root, name);
    if (svg) writeFile(path.join(OUT_DRAWABLE, `${baseName}.svg`), svg);
    return svg ? [baseName] : [];
  }
  if (tag === "selector") {
    const items = children(root, "item");
    const pick = (predicate) => items.find(predicate);
    const isChecked = (item) => attr(item, "state_checked") === "true";
    const isEnabled = (item) => attr(item, "state_enabled") === "false";
    const defaults = items.filter(
      (item) =>
        !Object.keys(item).some((key) => key.includes("state_")),
    );
    const defaultItem = defaults[0] ?? pick((item) => !isChecked(item)) ?? items[0];
    const checkedItem = pick(isChecked);
    const disabledItem = pick(isEnabled);
    const result = [];
    const handle = (item, suffix) => {
      const rawDrawable = attr(item, "drawable");
      if (typeof rawDrawable === "string" && rawDrawable.startsWith("@color/")) {
        const fill = color(rawDrawable, "#000000");
        const targetName = suffix ? `${name}-${suffix}` : name;
        const svg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100" width="100" height="100"><rect width="100" height="100" fill="${fill}"/></svg>\n`;
        writeFile(path.join(OUT_DRAWABLE, `${targetName}.svg`), svg);
        result.push(targetName);
        return;
      }
      const ref = resolveDrawableRef(rawDrawable);
      if (!ref) return;
      const fileRef = path.join(DRAWABLE_DIR, `${ref}.xml`);
      if (!fs.existsSync(fileRef)) {
        const bitmap = ["png", "jpg", "jpeg", "webp"]
          .map((ext) => path.join(DRAWABLE_DIR, `${ref}.${ext}`))
          .find((p) => fs.existsSync(p));
        if (bitmap) {
          const target = suffix ? `${name}-${suffix}.${path.extname(bitmap).slice(1)}` : `${name}.${path.extname(bitmap).slice(1)}`;
          fs.copyFileSync(bitmap, path.join(OUT_DRAWABLE, target));
          result.push(target.replace(/\.[^.]+$/, ""));
          return;
        }
      }
      if (!fs.existsSync(fileRef)) return;
      const doc2 = readXml(fileRef);
      const key2 = Object.keys(doc2).find((key) => !key.startsWith("?")) ?? Object.keys(doc2)[0];
      const tag2 = key2.split(".").pop();
      const root2Raw = doc2[key2];
      const root2 = Array.isArray(root2Raw) ? root2Raw[0] : root2Raw;
      const targetName = suffix ? `${name}-${suffix}` : name;
      let svg = null;
      if (tag2 === "vector") svg = convertVector(root2, targetName);
      else if (tag2 === "shape") svg = convertShape(root2, targetName);
      if (svg) {
        writeFile(path.join(OUT_DRAWABLE, `${targetName}.svg`), svg);
        result.push(targetName);
      }
    };
    handle(defaultItem, "");
    handle(checkedItem, "checked");
    handle(disabledItem, "disabled");
    return result;
  }
  if (tag === "shape") {
    const svg = convertShape(root, name);
    if (svg) {
      writeFile(path.join(OUT_DRAWABLE, `${baseName}.svg`), svg);
      return [baseName];
    }
  }
  if (tag === "ripple") {
    const item = children(root, "item")[0];
    const ref = resolveDrawableRef(attr(item, "drawable"));
    if (ref) return convertFile(ref, variant);
  }
  return [];
}

ensureDir(OUT_DRAWABLE);
const manifest = {};
const files = fs.readdirSync(DRAWABLE_DIR);
for (const file of files) {
  const ext = path.extname(file).toLowerCase();
  const name = path.basename(file, ext);
  if (ext === ".xml") {
    try {
      const result = convertFile(name);
      if (result.length) manifest[name] = result;
    } catch (e) {
      console.warn(`skip ${file}: ${e.message}`);
    }
  } else if ([".png", ".jpg", ".jpeg", ".webp"].includes(ext)) {
    fs.copyFileSync(path.join(DRAWABLE_DIR, file), path.join(OUT_DRAWABLE, file));
    manifest[name] = [name];
  }
}
writeFile(path.join(OUT_DRAWABLE, "manifest.json"), JSON.stringify(manifest, null, 2));
console.log(`converted ${Object.keys(manifest).length} drawables`);


