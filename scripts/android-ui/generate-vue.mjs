import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { XMLParser } from "fast-xml-parser";
import {
  APP_RES,
  OUT_DESIGN,
  dimensionToPx,
  loadResourceBundle,
  writeFile,
} from "./android.mjs";

const bundle = loadResourceBundle();
const OUT_GENERATED = path.join(OUT_DESIGN, "generated");
const OUT_REPORTS = path.join(OUT_DESIGN, "reports");
const DRAWABLE_DIR = path.join(
  process.cwd(),
  "src/assets/android/drawable",
);

const parser = new XMLParser({
  ignoreAttributes: false,
  attributeNamePrefix: "@_",
  parseTagValue: false,
  trimValues: true,
  isArray: (name) =>
    [
      "item",
      "path",
      "group",
      "string",
      "color",
      "dimen",
      "style",
      "array",
      "declare-styleable",
      "attr",
      "menu",
    ].includes(name),
});

const IGNORED_EXACT = new Set([
  "xmlns:android",
  "xmlns:app",
  "xmlns:tools",
  "xmlns:tool",
  "scrollbars",
  "layoutManager",
  "transitionName",
  "importantForAccessibility",
  "descendantFocusability",
  "overScrollMode",
  "clipToPadding",
  "focusable",
  "clickable",
  "tooltipText",
  "contentInsetStartWithNavigation",
  "contentInsetRight",
  "displayHomeAsUp",
  "attachToActivity",
  "fitStatusBar",
  "fitNavigationBar",
  "themeMode",
  "theme",
  "popupTheme",
  "titleTextAppearance",
  "textAppearance",
  "layout_behavior",
  "layout_span",
  "layout_row",
  "layout_column",
  "layout_columnSpan",
  "layout_columnWeight",
  "layout_constrainedWidth",
  "layout_constraintWidth",
  "layout_constraintHeight",
  "tabMode",
  "tabBackground",
  "tabRippleColor",
  "itemActiveIndicatorStyle",
  "labelVisibilityMode",
  "menu",
  "includeFontPadding",
  "importantForAutofill",
  "textCursorDrawable",
  "inputType",
  "searchIcon",
  "queryBackground",
  "submitBackground",
  "searchHintIcon",
  "iconifiedByDefault",
  "checkMark",
  "textOff",
  "textOn",
  "endIconCheckable",
  "endIconContentDescription",
  "endIconDrawable",
  "endIconMode",
  "helperText",
  "progressBackgroundTint",
  "indicatorColor",
  "indicatorSize",
  "trackCornerRadius",
  "trackThickness",
  "animationResolution",
  "indeterminateBehavior",
  "layout_editor_absoluteX",
  "layout_editor_absoluteY",
]);

const IGNORED_PREFIX = ["tools:", "tool:", "xmlns:"];

const THEME_ATTR_MAP = {
  "?attr/colorPrimary": "var(--ac-primary)",
  "?attr/colorAccent": "var(--ac-accent)",
  "?attr/colorBackground": "var(--ac-background)",
  "?android:attr/colorBackground": "var(--ac-background)",
  "?attr/colorControlNormal": "var(--ac-secondary-text)",
  "?attr/colorControlActivated": "var(--ac-primary)",
  "?attr/selectableItemBackground": "transparent",
  "?android:attr/selectableItemBackground": "transparent",
  "?attr/selectableItemBackgroundBorderless": "transparent",
  "?android:attr/selectableItemBackgroundBorderless": "transparent",
  "?attr/actionBarSize": "56px",
  "?attr/actionBarStyle": "",
  "?attr/titleBarStyle": "",
  "?attr/actionOverflowMenuStyle": "",
  "?attr/popupMenuStyle": "",
  "?attr/colorPrimaryDark": "var(--ac-primaryDark)",
  "?attr/textColorPrimary": "var(--ac-primary-text)",
  "?attr/textColorSecondary": "var(--ac-secondary-text)",
  "?android:attr/textColorSecondary": "var(--ac-secondary-text)",
  "?android:attr/textColorAlertDialogListItem": "var(--ac-primary-text)",
  "?android:attr/listPreferredItemHeight": "56px",
  "?android:attr/progressBarStyleHorizontal": "",
  "?android:attr/actionButtonStyle": "",
};

const ATTR_ALIASES = {
  "android:layout_width": "layout_width",
  "android:layout_height": "layout_height",
};

function unwrapRoot(doc) {
  const key = Object.keys(doc).find((item) => !item.startsWith("?"));
  const value = doc[key];
  return { tag: key, node: Array.isArray(value) ? value[0] : value };
}

function normalizeName(raw) {
  if (raw.startsWith("@_android:")) return raw.slice(10);
  if (raw.startsWith("@_app:")) return `app:${raw.slice(6)}`;
  if (raw.startsWith("@_tools:")) return `tools:${raw.slice(8)}`;
  if (raw.startsWith("@_tool:")) return `tool:${raw.slice(7)}`;
  if (raw.startsWith("@_xmlns:")) return `xmlns:${raw.slice(8)}`;
  if (raw.startsWith("@_")) return raw.slice(2);
  return raw;
}

function isIgnored(name) {
  if (IGNORED_EXACT.has(name)) return true;
  return IGNORED_PREFIX.some((prefix) => name.startsWith(prefix));
}

function cssColor(value) {
  if (typeof value !== "string") return value;
  const match = value.match(/^#([0-9a-fA-F]{8})$/);
  if (!match) return value;
  const argb = match[1];
  return `#${argb.slice(2)}${argb.slice(0, 2)}`;
}

function resolveDimension(value) {
  const px = dimensionToPx(value);
  if (typeof px === "number") return `${px}px`;
  return null;
}

function drawableAsset(name) {
  for (const ext of ["svg", "png", "jpg", "jpeg", "webp"]) {
    const rel = `assets/android/drawable/${name}.${ext}`;
    if (fs.existsSync(path.join(process.cwd(), "src", rel))) return rel;
  }
  return null;
}

function iconName(raw) {
  if (typeof raw !== "string") return "";
  if (raw.startsWith("@drawable/")) return raw.slice(10);
  if (raw.startsWith("@android:drawable/")) return raw.slice(18);
  return "";
}

function drawableFile(name) {
  for (const ext of ["svg", "png", "jpg", "jpeg", "webp"]) {
    const file = path.join(DRAWABLE_DIR, `${name}.${ext}`);
    if (fs.existsSync(file)) return file;
  }
  return null;
}

const LAYOUT_CACHE = new Map();

function loadLayoutNode(ref) {
  const name = String(ref).replace(/^@layout\//, "");
  if (LAYOUT_CACHE.has(name)) return LAYOUT_CACHE.get(name);
  let entry = null;
  for (const dir of ["layout", "layout-land"]) {
    const file = path.join(APP_RES, dir, `${name}.xml`);
    if (!fs.existsSync(file)) continue;
    const doc = parser.parse(fs.readFileSync(file, "utf8"));
    const { tag, node } = unwrapRoot(doc);
    entry = { name, tag, node };
    break;
  }
  LAYOUT_CACHE.set(name, entry);
  return entry;
}

const TAG_LAYOUT_EXPANSIONS = new Map([
  ["SelectActionBar", "@layout/view_select_action_bar"],
]);

function renderLayoutRef(ref, state, depth) {
  const entry = loadLayoutNode(ref);
  if (!entry) {
    state.expandedMeta.push({
      name: `layout:${ref}`,
      raw: String(ref),
      status: "unresolved",
      reason: `缺少 layout: ${String(ref).replace(/^@layout\//, "")}`,
    });
    return null;
  }
  if (state.layoutChain.includes(entry.name)) {
    state.expandedMeta.push({
      name: `layout:${ref}`,
      raw: String(ref),
      status: "unresolved",
      reason: `布局引用循环: ${state.layoutChain.concat(entry.name).join(" -> ")}`,
    });
    return null;
  }
  const outerMeta = state.meta;
  const outerPath = state.path;
  state.meta = state.expandedMeta;
  state.path = `${outerPath}@${entry.name}`;
  state.layoutChain.push(entry.name);
  try {
    return renderNode(entry.tag, entry.node, state, depth);
  } finally {
    state.layoutChain.pop();
    state.path = outerPath;
    state.meta = outerMeta;
  }
}

function resolveValue(raw, nodeAttrs) {
  if (raw === undefined || raw === null) return { value: raw, status: "mapped" };
  const value = String(raw);
  if (value.startsWith("@dimen/")) {
    const key = value.slice(7);
    const dimen = bundle.dimens[key];
    if (dimen === undefined) {
      return { status: "unresolved", reason: `缂哄皯 dimen: ${key}` };
    }
    const px = dimensionToPx(dimen);
    return { value: typeof px === "number" ? `${px}px` : dimen, status: "mapped" };
  }
  if (value.startsWith("@color/")) {
    const key = value.slice(7);
    const color = bundle.colorsLight[key];
    if (color === undefined) {
      return { status: "unresolved", reason: `缂哄皯 color: ${key}` };
    }
    return { value: cssColor(color), status: "mapped" };
  }
  if (value.startsWith("@string/")) {
    const key = value.slice(8);
    const text = bundle.strings[key];
    if (text === undefined) {
      return { status: "unresolved", reason: `缂哄皯 string: ${key}` };
    }
    return { value: text, status: "mapped" };
  }
  if (value.startsWith("@drawable/")) {
    const key = value.slice(10);
    const file = drawableFile(key);
    if (!file) {
      return { status: "unresolved", reason: `缂哄皯 drawable: ${key}` };
    }
    return { value: key, status: "mapped", drawable: key };
  }
  if (value.startsWith("@android:color/")) {
    const key = value.slice(15);
    const map = { transparent: "transparent", white: "#ffffff", black: "#000000" };
    if (map[key]) return { value: map[key], status: "mapped" };
    return { status: "unresolved", reason: `缂哄皯 android color: ${key}` };
  }
  if (value.startsWith("@style/")) {
    return { value, status: "mapped", style: value.slice(7) };
  }
  if (value.startsWith("@array/")) {
    const key = value.slice(7);
    const items = arrays.get(key);
    if (!items) {
      return { status: "unresolved", reason: `缂哄皯 array: ${key}` };
    }
    return { value: items, status: "mapped", array: true };
  }
  if (value.startsWith("@android:")) {
    return { value, status: "ignored", reason: "android framework resource" };
  }
  if (value.startsWith("?")) {
    const mapped = THEME_ATTR_MAP[value];
    if (mapped !== undefined) {
      return mapped === ""
        ? { value: "", status: "ignored", reason: "theme style" }
        : { value: mapped, status: "mapped" };
    }
    if (/Style|Theme|attr\/actionBar|popupMenu/i.test(value)) {
      return { value: "", status: "ignored", reason: "theme style" };
    }
    return { status: "unresolved", reason: `鏈煡涓婚灞炴€? ${value}` };
  }
  const dimension = resolveDimension(value);
  if (dimension) return { value: dimension, status: "mapped" };
  if (value.startsWith("#")) {
    return { value: cssColor(value), status: "mapped" };
  }
  return { value, status: "mapped" };
}

function loadStyles() {
  const map = new Map();
  for (const dir of ["values", "values-night"]) {
    const file = path.join(APP_RES, dir, "styles.xml");
    if (!fs.existsSync(file)) continue;
    const doc = parser.parse(fs.readFileSync(file, "utf8"));
    const resources = doc.resources ?? {};
    const styles = resources.style ?? [];
    for (const style of Array.isArray(styles) ? styles : [styles]) {
      const name = style["@_name"];
      if (!name) continue;
      const items = {};
      const list = style.item ?? [];
      for (const item of Array.isArray(list) ? list : [list]) {
        const key = item["@_name"];
        if (key) items[key] = item["#text"] ?? "";
      }
      map.set(name, { parent: style["@_parent"], items });
    }
  }
  const resolved = new Map();
  const resolve = (name, guard = 0) => {
    if (resolved.has(name)) return resolved.get(name);
    if (guard > 20) return {};
    const entry = map.get(name);
    if (!entry) return {};
    const parent = entry.parent
      ? resolve(entry.parent.replace(/^@style\//, ""), guard + 1)
      : {};
    const merged = { ...parent, ...entry.items };
    resolved.set(name, merged);
    return merged;
  };
  for (const name of map.keys()) resolve(name);
  return resolved;
}

function loadArrays() {
  const map = new Map();
  for (const file of ["values/arrays.xml", "values/array_values.xml"]) {
    const full = path.join(APP_RES, file);
    if (!fs.existsSync(full)) continue;
    const doc = parser.parse(fs.readFileSync(full, "utf8"));
    const resources = doc.resources ?? {};
    const arrays = resources["string-array"] ?? [];
    for (const array of Array.isArray(arrays) ? arrays : [arrays]) {
      const name = array["@_name"];
      if (!name) continue;
      const items = (array.item ?? []).map((item) =>
        typeof item === "object" ? item["#text"] ?? "" : String(item),
      );
      map.set(name, items);
    }
  }
  return map;
}

const arrays = loadArrays();

const FRAMEWORK_STYLES = new Map([
  ["Widget.Material3.CircularProgressIndicator.Small", "width:36px;height:36px"],
  ["Widget.AppCompat.CompoundButton.Switch", ""],
  ["Widget.Material3.Button.TonalButton", "background:var(--ac-background-menu);color:var(--ac-primary-text)"],
  ["Widget.AppCompat.ProgressBar", ""],
]);

const styles = loadStyles();

const TEXT_TAGS = new Set([
  "TextView",
  "Button",
  "AppCompatTextView",
  "AccentTextView",
  "AccentBgTextView",
  "AccentStrokeTextView",
  "StrokeTextView",
  "BadgeView",
  "BevelLabelView",
  "MultilineTextView",
  "ScrollTextView",
  "AppCompatButton",
  "CheckedTextView",
  "RadioButton",
  "CheckBox",
  "SwitchCompat",
  "ThemeSwitch",
  "ThemeCheckBox",
  "ThemeRadioButton",
  "ThemeRadioNoButton",
  "TextFontWeightConverter",
  "ChineseConverter",
]);

const EDIT_TAGS = new Set([
  "EditText",
  "ThemeEditText",
  "AutoCompleteTextView",
  "TextInputLayout",
]);

const IMAGE_TAGS = new Set([
  "ImageView",
  "AppCompatImageView",
  "ImageButton",
  "CircleImageView",
  "CoverImageView",
  "PhotoView",
  "ArcView",
  "ImageFilterView",
  "FilletImageView",
]);

const LIST_TAGS = new Set([
  "RecyclerView",
  "FastScrollRecyclerView",
  "RecyclerViewAtPager2",
  "ViewPager",
  "FlexboxLayout",
]);

const TITLE_TAGS = new Set(["TitleBar", "Toolbar"]);

const IGNORE_TAGS = new Set(["ViewStub", "View"]);

const CONTAINER_TAGS = new Set([
  "LinearLayout",
  "LinearLayoutCompat",
  "FrameLayout",
  "ConstraintLayout",
  "RelativeLayout",
  "ScrollView",
  "HorizontalScrollView",
  "NestedScrollView",
  "NoChildScrollNestedScrollView",
  "SwipeRefreshLayout",
  "CardView",
  "DynamicFrameLayout",
  "WebtoonFrame",
  "merge",
]);

function resolveNodeAttrs(node) {
  const attrs = {};
  const rawAttrs = {};
  for (const [rawKey, rawValue] of Object.entries(node)) {
    if (!rawKey.startsWith("@_")) continue;
    const name = normalizeName(rawKey);
    attrs[name] = rawValue;
    rawAttrs[name] = rawValue;
  }
  const styleName = attrs.style;
  if (typeof styleName === "string" && styleName.startsWith("@style/")) {
    const styleItems = styles.get(styleName.slice(7));
    if (styleItems) {
      for (const [key, value] of Object.entries(styleItems)) {
        const normalized =
          key.replace(/^android:/, "").replace(/^app:/, "");
        if (!(normalized in attrs)) attrs[normalized] = value;
      }
    }
  }
  return { attrs, rawAttrs };
}

function cssFromAttrs(attrs, state, customTag, rawAttrs) {
  const css = [];
  const meta = [];
  const add = (name, raw) => {
    if (!(name in rawAttrs)) {
      return { value: raw, status: "mapped" };
    }
    const result = resolveValue(raw, attrs);
    meta.push({ name, raw, path: state.path, ...result });
    return result;
  };
  const width = add("layout_width", attrs.layout_width);
  const height = add("layout_height", attrs.layout_height);
  if (width.value) {
    const v = width.value;
    if (v === "match_parent") css.push("width:100%");
    else if (v === "wrap_content") css.push("width:auto");
    else css.push(`width:${v}`);
  }
  if (height.value) {
    const v = height.value;
    if (v === "match_parent") css.push("height:100%");
    else if (v === "wrap_content") css.push("height:auto");
    else css.push(`height:${v}`);
  }
  const margins = {
    layout_margin: "margin",
    layout_marginTop: "margin-top",
    layout_marginBottom: "margin-bottom",
    layout_marginLeft: "margin-left",
    layout_marginStart: "margin-left",
    layout_marginRight: "margin-right",
    layout_marginEnd: "margin-right",
    layout_marginHorizontal: "margin-inline",
    layout_marginVertical: "margin-block",
  };
  for (const [key, prop] of Object.entries(margins)) {
    if (attrs[key] === undefined) continue;
    const r = add(key, attrs[key]);
    if (r.value) css.push(`${prop}:${r.value}`);
  }
  const paddings = {
    padding: "padding",
    paddingTop: "padding-top",
    paddingBottom: "padding-bottom",
    paddingLeft: "padding-left",
    paddingStart: "padding-left",
    paddingRight: "padding-right",
    paddingEnd: "padding-right",
    paddingHorizontal: "padding-inline",
    paddingVertical: "padding-block",
  };
  for (const [key, prop] of Object.entries(paddings)) {
    if (attrs[key] === undefined) continue;
    const r = add(key, attrs[key]);
    if (r.value) css.push(`${prop}:${r.value}`);
  }
  if (attrs.layout_weight) {
    css.push(`flex-grow:${attrs.layout_weight}`, "flex-shrink:1", "flex-basis:0%");
    meta.push({ name: "layout_weight", raw: attrs.layout_weight, value: attrs.layout_weight, status: "mapped", target: "flex" });
  }
  if (attrs.orientation) {
    css.push(`flex-direction:${attrs.orientation === "horizontal" ? "row" : "column"}`);
    meta.push({ name: "orientation", raw: attrs.orientation, value: attrs.orientation, status: "mapped", target: "flex-direction" });
  }
  const gravityRawName =
    attrs.gravity !== undefined ? "gravity" : attrs.layout_gravity !== undefined ? "layout_gravity" : "";
  const gravityValue =
    attrs.gravity !== undefined ? attrs.gravity : attrs.layout_gravity !== undefined ? attrs.layout_gravity : "";
  if (gravityValue) {
    if (String(gravityValue).includes("center")) css.push("justify-content:center", "align-items:center");
    else if (String(gravityValue).includes("bottom") || String(gravityValue).includes("end")) css.push("justify-content:flex-end");
    else if (String(gravityValue).includes("right")) css.push("align-items:flex-end");
    if (String(gravityValue).includes("center_horizontal")) css.push("align-items:center");
    meta.push({ name: gravityRawName, raw: gravityValue, value: gravityValue, status: "mapped", target: "flex alignment" });
  }
  if (attrs.background !== undefined) {
    const r = add("background", attrs.background);
    if (r.drawable) {
      const asset = drawableAsset(r.drawable);
      if (asset) {
        state.drawables.push(asset);
        state.nodeDrawableClass = `bg-${state.drawables.length - 1}`;
      }
    } else if (r.value && typeof r.value === "string" && r.value.startsWith("#")) {
      css.push(`background:${r.value}`);
    } else if (r.value && typeof r.value === "string" && r.value) {
      css.push(`background:${r.value}`);
    }
  }
  if (attrs.backgroundTint) {
    const r = add("backgroundTint", attrs.backgroundTint);
    if (r.value) css.push(`background-color:${r.value}`);
  }
  if (attrs.textColor !== undefined) {
    const r = add("textColor", attrs.textColor);
    if (r.value) css.push(`color:${r.value}`);
  }
  if (attrs.textSize !== undefined) {
    const r = add("textSize", attrs.textSize);
    if (r.value) css.push(`font-size:${r.value}`);
  }
  if (attrs.textStyle !== undefined) {
    const value = String(attrs.textStyle);
    css.push(value.includes("bold") ? "font-weight:600" : "font-weight:400");
    meta.push({ name: "textStyle", raw: value, value, status: "mapped", target: "font-weight" });
  }
  if (attrs.visibility !== undefined) {
    const value = String(attrs.visibility);
    if (value === "gone") css.push("display:none");
    else if (value === "invisible") css.push("visibility:hidden");
    meta.push({ name: "visibility", raw: value, value, status: "mapped", target: "display/visibility" });
  }
  if (attrs.alpha !== undefined) {
    css.push(`opacity:${attrs.alpha}`);
    meta.push({ name: "alpha", raw: attrs.alpha, value: attrs.alpha, status: "mapped", target: "opacity" });
  }
  if (attrs.elevation !== undefined) {
    const r = add("elevation", attrs.elevation);
    if (r.value) css.push(`box-shadow:0 ${r.value} ${r.value} rgba(0,0,0,0.18)`);
  }
  if (attrs.cardElevation !== undefined) {
    const r = add("cardElevation", attrs.cardElevation);
    if (r.value) css.push(`box-shadow:0 ${r.value} ${r.value} rgba(0,0,0,0.18)`);
  }
  for (const key of ["radius", "cardCornerRadius", "cornerRadius"]) {
    if (attrs[key] !== undefined) {
      const r = add(key, attrs[key]);
      if (r.value) css.push(`border-radius:${r.value}`);
    }
  }
  if (attrs.civ_border_width) {
    const width = add("civ_border_width", attrs.civ_border_width);
    const color = attrs.civ_border_color
      ? add("civ_border_color", attrs.civ_border_color)
      : { value: "var(--ac-divider)" };
    if (width.value) css.push(`border:${width.value} solid ${color.value}`);
  }
  if (attrs.maxLines) {
    css.push(`-webkit-line-clamp:${attrs.maxLines}`, "display:-webkit-box", "-webkit-box-orient:vertical", "overflow:hidden");
    meta.push({ name: "maxLines", raw: attrs.maxLines, value: attrs.maxLines, status: "mapped", target: "line-clamp" });
  }
  if (attrs.ellipsize) {
    css.push("text-overflow:ellipsis", "white-space:nowrap", "overflow:hidden");
    meta.push({ name: "ellipsize", raw: attrs.ellipsize, value: attrs.ellipsize, status: "mapped", target: "text-overflow" });
  }
  if (attrs.scaleType) {
    css.push(`object-fit:${attrs.scaleType === "centerCrop" ? "cover" : "contain"}`);
    meta.push({ name: "scaleType", raw: attrs.scaleType, value: attrs.scaleType, status: "mapped", target: "object-fit" });
  }
  for (const key of ["minWidth", "maxWidth", "minHeight", "maxHeight"]) {
    if (attrs[key] === undefined) continue;
    const r = add(key, attrs[key]);
    if (r.value) css.push(`${key.replace(/[A-Z]/g, (m) => `-${m.toLowerCase()}`)}:${r.value}`);
  }
  if (attrs.layout_constraintWidth_percent !== undefined) {
    const percent = Number(attrs.layout_constraintWidth_percent) * 100;
    css.push(`width:${percent}%`);
    meta.push({ name: "layout_constraintWidth_percent", raw: attrs.layout_constraintWidth_percent, value: attrs.layout_constraintWidth_percent, status: "mapped", target: "width" });
  }
  if (attrs.layout_constraintHeight_percent !== undefined) {
    const percent = Number(attrs.layout_constraintHeight_percent) * 100;
    css.push(`height:${percent}%`);
    meta.push({ name: "layout_constraintHeight_percent", raw: attrs.layout_constraintHeight_percent, value: attrs.layout_constraintHeight_percent, status: "mapped", target: "height" });
  }
  if (attrs.layout_constraintDimensionRatio !== undefined) {
    css.push(`aspect-ratio:${attrs.layout_constraintDimensionRatio}`);
    meta.push({ name: "layout_constraintDimensionRatio", raw: attrs.layout_constraintDimensionRatio, value: attrs.layout_constraintDimensionRatio, status: "mapped", target: "aspect-ratio" });
  }
  if (attrs.layout_constraintHorizontal_bias !== undefined) {
    const bias = Number(attrs.layout_constraintHorizontal_bias);
    css.push(bias <= 0.5 ? "align-self:flex-start" : bias >= 1 ? "align-self:flex-end" : "align-self:center");
    meta.push({ name: "layout_constraintHorizontal_bias", raw: attrs.layout_constraintHorizontal_bias, value: attrs.layout_constraintHorizontal_bias, status: "mapped", target: "align-self" });
  }
  if (attrs.layout_constraintVertical_bias !== undefined) {
    const bias = Number(attrs.layout_constraintVertical_bias);
    css.push(bias <= 0.5 ? "justify-content:flex-start" : bias >= 1 ? "justify-content:flex-end" : "justify-content:center");
    meta.push({ name: "layout_constraintVertical_bias", raw: attrs.layout_constraintVertical_bias, value: attrs.layout_constraintVertical_bias, status: "mapped", target: "justify-content" });
  }
  if (attrs.layout_constraintHorizontal_chainStyle !== undefined) {
    const style = String(attrs.layout_constraintHorizontal_chainStyle);
    css.push(style === "spread_inside" ? "justify-content:space-between" : style === "packed" ? "justify-content:center" : "justify-content:space-between");
    meta.push({ name: "layout_constraintHorizontal_chainStyle", raw: attrs.layout_constraintHorizontal_chainStyle, value: attrs.layout_constraintHorizontal_chainStyle, status: "mapped", target: "justify-content" });
  }
  if (attrs.enabled === "false") {
    css.push("opacity:0.5");
    meta.push({ name: "enabled", raw: attrs.enabled, value: attrs.enabled, status: "mapped", target: "opacity" });
  }
  for (const key of Object.keys(rawAttrs)) {
    if (meta.some((item) => item.name === key)) continue;
    if (key === "style") {
      const styleName = String(rawAttrs[key]);
      if (styleName.startsWith("?")) {
        const mapped = THEME_ATTR_MAP[styleName];
        meta.push({
          name: key,
          raw: rawAttrs[key],
          value: mapped ?? "",
          status: mapped === undefined ? "unresolved" : "mapped",
          target: "framework style",
          reason: mapped === undefined ? `缂哄皯 style: ${styleName}` : undefined,
        });
        continue;
      }
      const localStyleName = styleName.replace(/^@style\//, "");
      const styleItems = styles.get(localStyleName);
      const frameworkStyle = FRAMEWORK_STYLES.get(localStyleName);
      const resolved = Boolean(styleItems || frameworkStyle !== undefined);
      meta.push({
        name: key,
        raw: rawAttrs[key],
        value: styleItems
          ? Object.keys(styleItems).length
          : frameworkStyle ?? 0,
        status: resolved ? "mapped" : "unresolved",
        target: "style items",
        reason: resolved ? undefined : `缂哄皯 style: ${styleName}`,
      });
      continue;
    }
    if (isIgnored(key)) {
      meta.push({ name: key, raw: rawAttrs[key], value: rawAttrs[key], status: "ignored", reason: "non-visual/platform attribute" });
      continue;
    }
    if (key.startsWith("layout_constraint") || key.startsWith("layout_align") || key.startsWith("layout_to") || key.startsWith("layout_below") || key.startsWith("layout_above") || key.startsWith("layout_center")) {
      meta.push({ name: key, raw: rawAttrs[key], value: rawAttrs[key], status: "ignored", reason: "constraint mapped to flex/grid intent" });
      continue;
    }
    if (key === "app:contentLayout" || key === "contentLayout") {
      const layoutName = String(rawAttrs[key]).replace(/^@layout\//, "");
      const exists = loadLayoutNode(rawAttrs[key]) !== null;
      meta.push({
        name: key,
        raw: rawAttrs[key],
        value: rawAttrs[key],
        status: exists ? "mapped" : "unresolved",
        target: "expanded child layout",
        ...(exists ? {} : { reason: `缺少 layout: ${layoutName}` }),
      });
      continue;
    }
    if (key.startsWith("app:") || key === "id" || key === "contentDescription" || key === "name" || key === "text" || key === "hint" || key === "src" || key === "tint" || key === "checked" || key === "singleLine" || key === "lines" || key === "inputType" || key === "maxLength" || key === "button" || key === "entries" || key === "digits" || key === "textAlignment" || key === "textIsSelectable" || key === "autoLink" || key === "rotation" || key === "rotationX" || key === "backgroundTint" || key === "pressedTranslationZ" || key === "fabSize" || key === "spanCount" || key === "dividerDrawable" || key === "showDivider" || key === "layout" || key === "menu" || key === "subtitle" || key === "title") {
      meta.push({ name: key, raw: rawAttrs[key], value: rawAttrs[key], status: "mapped", target: "component prop" });
      continue;
    }
    const resolved = resolveValue(rawAttrs[key], attrs);
    if (resolved.status === "unresolved") {
      meta.push({ name: key, raw: rawAttrs[key], ...resolved });
    } else {
      meta.push({ name: key, raw: rawAttrs[key], ...resolved, status: "ignored", reason: "no direct CSS mapping" });
    }
  }
  return { css, meta };
}

function escapeHtml(value) {
  return String(value)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function attrValue(attrs, name) {
  return attrs[name];
}

function renderNode(tag, node, state, depth = 0) {
  const { attrs, rawAttrs } = resolveNodeAttrs(node);
  const { css, meta } = cssFromAttrs(attrs, state, tag, rawAttrs);
  state.meta.push(...meta);
  const path = state.path;
  const text = attrs.text !== undefined ? attrs.text : attrs.hint;
  const style = css.join(";");
  const styleAttr = style ? ` style="${escapeHtml(style)}"` : "";
  const classes = [];
  if (state.nodeDrawableClass) {
    classes.push(state.nodeDrawableClass);
    state.nodeDrawableClass = null;
  }
  const aria = attrs.contentDescription ? ` aria-label="${escapeHtml(attrs.contentDescription)}"` : "";
  const dataTag = ` data-tag="${escapeHtml(tag)}"`;
  const dataPath = ` data-path="${escapeHtml(path)}"`;
  const common = `${styleAttr}${aria}${dataTag}${dataPath}`;

  const childList = wrapChildren(node);

  const renderChildren = () =>
    childList
      .map((child, index) => {
        const childTag = child.__tag ?? "View";
        const childNode = child.__node ?? child;
        state.path = `${path}/${index}`;
        return renderNode(childTag, childNode, state, depth + 1);
      })
      .join("");

  const typeTag = tag.includes(".") ? tag.split(".").pop() : tag;

  if (TITLE_TAGS.has(typeTag)) {
    const title = attrs["app:title"] ?? attrs.title ?? "";
    const subtitle = attrs["app:subtitle"] ?? attrs.subtitle ?? "";
    const contentRef = attrs["app:contentLayout"] ?? attrs.contentLayout;
    let contentHtml = "";
    if (contentRef) {
      const rendered = renderLayoutRef(contentRef, state, depth);
      if (rendered !== null) contentHtml = `<div class="titlebar-content">${rendered}</div>`;
    }
    return `<div class="titlebar ${classes.join(" ")}"${common}><span class="titlebar-title">{{ ${JSON.stringify(title)} }}</span>${subtitle ? `<span class="titlebar-subtitle">{{ ${JSON.stringify(subtitle)} }}</span>` : ""}${contentHtml}${renderChildren()}</div>`;
  }

  if (typeTag === "SearchView") {
    const hintRaw = attrs["app:defaultQueryHint"] ?? attrs.defaultQueryHint;
    let placeholder = "";
    if (hintRaw !== undefined) {
      const hintResult = resolveValue(hintRaw, attrs);
      if (hintResult.status !== "unresolved") placeholder = String(hintResult.value ?? "");
    }
    return `<label class="search ${classes.join(" ")}"${common}><input class="search-input" type="search" placeholder="${escapeHtml(placeholder)}" />${renderChildren()}</label>`;
  }

  if (TEXT_TAGS.has(typeTag)) {
    const value = text === undefined ? "" : String(text);
    const icon = attrs.src && IMAGE_TAGS.has(typeTag)
      ? `<AndroidIcon name="${escapeHtml(attrs.src)}" :size="24" />`
      : "";
    const checked = attrs.checked === true || attrs.checked === "true";
    const switchLike = ["ThemeSwitch", "ThemeCheckBox", "SwitchCompat", "CheckBox", "ThemeRadioButton", "ThemeRadioNoButton", "RadioButton"].includes(typeTag);
    if (switchLike) {
      return `<label class="switch-row ${classes.join(" ")}"${common}>${icon}<input type="checkbox"${checked ? " checked" : ""} /><span>{{ ${JSON.stringify(value)} }}</span></label>`;
    }
    return `<span class="text ${classes.join(" ")}"${common}>${icon}{{ ${JSON.stringify(value)} }}</span>`;
  }

  if (EDIT_TAGS.has(typeTag)) {
    const placeholder = attrs.hint !== undefined ? attrs.hint : "";
    const multiline = String(attrs.inputType ?? "").includes("MultiLine") || String(attrs.lines ?? "") !== "";
    if (typeTag === "TextInputLayout") {
      return `<div class="input-wrap ${classes.join(" ")}"${common}>${renderChildren()}</div>`;
    }
    return multiline
      ? `<textarea class="edit ${classes.join(" ")}" placeholder="${escapeHtml(placeholder)}"${common}></textarea>`
      : `<input class="edit ${classes.join(" ")}" placeholder="${escapeHtml(placeholder)}"${common} />`;
  }

  if (IMAGE_TAGS.has(typeTag)) {
    const src = attrs.src ? iconName(attrs.src) : "";
    if (src) {
      return `<span class="image ${classes.join(" ")}"${common}><AndroidIcon name="${escapeHtml(src)}" :size="24" /></span>`;
    }
    return `<span class="image-placeholder ${classes.join(" ")}"${common}>${escapeHtml(typeTag)}</span>`;
  }

  if (LIST_TAGS.has(typeTag)) {
    const childHtml = renderChildren();
    if (childHtml) {
      return `<div class="list ${classes.join(" ")}"${common}>${childHtml}</div>`;
    }
    return `<div class="list ${classes.join(" ")}"${common}><div class="list-row">列表项</div><div class="list-row">列表项</div><div class="list-row">列表项</div></div>`;
  }

  if (IGNORE_TAGS.has(typeTag)) {
    return `<div class="box ${classes.join(" ")}"${common}></div>`;
  }

  if (CONTAINER_TAGS.has(typeTag) || typeTag === "merge") {
    return `<div class="box ${classes.join(" ")}"${common}>${renderChildren()}</div>`;
  }

  if (typeTag === "Space") {
    return `<span class="space ${classes.join(" ")}"${common}></span>`;
  }

  if (typeTag === "VerticalSeekBarWrapper") {
    return `<div class="seek-wrap ${classes.join(" ")}"${common}>${renderChildren()}</div>`;
  }

  if (typeTag === "ThemeSeekBar" || typeTag === "DetailSeekBar" || typeTag === "AppCompatSeekBar" || typeTag === "VerticalSeekBar") {
    return `<input class="seek ${classes.join(" ")}" type="range"${common} />`;
  }

  if (typeTag === "ThemeProgressBar" || typeTag === "ProgressBar" || typeTag === "ContentLoadingProgressBar" || typeTag === "RefreshProgressBar" || typeTag === "RotateLoading") {
    return `<div class="progress ${classes.join(" ")}"${common}>加载中</div>`;
  }

  if (typeTag === "FloatingActionButton" || typeTag === "ImageButton") {
    return `<button class="fab ${classes.join(" ")}"${common}>${attrs.src ? `<AndroidIcon name="${escapeHtml(attrs.src)}" :size="22" />` : ""}</button>`;
  }

  if (typeTag === "SelectActionBar") {
    const expansionRef = TAG_LAYOUT_EXPANSIONS.get(typeTag);
    const expanded = expansionRef ? renderLayoutRef(expansionRef, state, depth) : null;
    if (expanded !== null) return expanded;
    return `<div class="select-bar ${classes.join(" ")}"${common}>选择操作栏</div>`;
  }

  if (typeTag === "Barrier") {
    return "";
  }

  if (typeTag === "TabLayout") {
    return `<div class="tabs ${classes.join(" ")}"${common}><span>Tab 1</span><span>Tab 2</span><span>Tab 3</span></div>`;
  }

  if (typeTag === "ThemeBottomNavigationVIew") {
    return `<div class="bottom-nav ${classes.join(" ")}"${common}>底部导航</div>`;
  }

  return `<div class="custom ${classes.join(" ")}"${common} data-custom="true">${escapeHtml(typeTag)}${renderChildren()}</div>`;
}

function wrapChildren(node) {
  const out = [];
  for (const [key, value] of Object.entries(node)) {
    if (
      key.startsWith("@_") ||
      key.startsWith("#") ||
      key.startsWith("?") ||
      key.startsWith("__")
    ) {
      continue;
    }
    const list = Array.isArray(value) ? value : [value];
    for (const child of list) {
      if (typeof child !== "object" || child === null) continue;
      const keyName = key.split(".").pop();
      child.__tag = keyName;
      child.__node = child;
      out.push(child);
    }
  }
  return out;
}

function compileLayout(file, group, name) {
  const doc = parser.parse(fs.readFileSync(file, "utf8"));
  const { tag, node } = unwrapRoot(doc);
  const state = { path: "$", drawables: [], nodeDrawableClass: null, unresolved: [], unsupported: [], meta: [], expandedMeta: [], layoutChain: [] };
  node.__tag = tag;
  node.__node = node;
  const rootAttrs = resolveNodeAttrs(node);
  rootAttrs.__tag = tag;
  const rootTag = tag;
  const html = renderNode(rootTag, node, state);
  const nodeCount = countNodes(node);
  const attrCount = countAttrs(node);
  const meta = state.meta;
  const expandedMeta = state.expandedMeta;
  const unresolved = [...meta, ...expandedMeta].filter((item) => item.status === "unresolved");
  const unsupported = [...meta, ...expandedMeta].filter((item) => item.status === "unsupported");
  const hash = crypto.createHash("sha1").update(fs.readFileSync(file)).digest("hex");
  const bgCss = state.drawables
    .map((rel, index) => `.bg-${index}{background-image:url("/src/${rel}");background-size:cover;background-position:center;}`)
    .join("\n");
  const vue = `<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-root">
${html}
  </div>
</template>

<style scoped>
.generated-root { width:100%; min-height:100%; background:var(--ac-background); color:var(--ac-primary-text); }
.box { display:flex; flex-direction:column; box-sizing:border-box; min-width:0; min-height:0; }
.text { display:inline-flex; align-items:center; box-sizing:border-box; min-width:0; }
.switch-row { display:flex; align-items:center; gap:6px; box-sizing:border-box; }
.edit { border:1px solid var(--ac-divider); background:var(--ac-background); color:var(--ac-primary-text); padding:6px 8px; }
.image { display:inline-flex; align-items:center; justify-content:center; }
.image-placeholder { display:inline-flex; align-items:center; justify-content:center; border:1px dashed var(--ac-divider); color:var(--ac-secondary-text); font-size:10px; min-width:24px; min-height:24px; }
.list { display:flex; flex-direction:column; border-top:1px solid var(--ac-divider); border-bottom:1px solid var(--ac-divider); }
.list-row { padding:10px 16px; border-bottom:1px solid var(--ac-divider); color:var(--ac-secondary-text); font-size:13px; }
.titlebar { display:flex; align-items:center; gap:8px; min-height:56px; padding:0 16px; background:var(--ac-primary); color:#fff; font-size:20px; }
.titlebar-subtitle { font-size:12px; opacity:.8; }
.seek { width:100%; }
.seek-wrap { display:flex; align-items:center; justify-content:center; width:100%; height:100%; }
.progress { padding:12px; color:var(--ac-secondary-text); text-align:center; font-size:12px; }
.fab { width:56px; height:56px; border-radius:50%; border:none; background:var(--ac-primary); color:#fff; display:inline-flex; align-items:center; justify-content:center; }
.select-bar { padding:8px 16px; background:var(--ac-background-menu); color:var(--ac-secondary-text); font-size:12px; border-top:1px solid var(--ac-divider); }
.tabs { display:flex; gap:8px; padding:0 12px; border-bottom:1px solid var(--ac-divider); }
.tabs span { padding:10px 8px; font-size:13px; color:var(--ac-secondary-text); }
.bottom-nav { display:flex; justify-content:center; align-items:center; height:50px; background:var(--ac-background); border-top:1px solid var(--ac-divider); color:var(--ac-secondary-text); }
.space { flex:1; }
.custom { display:flex; box-sizing:border-box; }
.titlebar-content { flex:1; min-width:0; display:flex; align-items:center; }
.search { display:flex; align-items:center; width:100%; height:100%; box-sizing:border-box; }
.search-input { flex:1; min-width:0; border:none; outline:none; background:transparent; color:inherit; font-size:13px; padding:0 12px; }
${bgCss}
</style>
`;
  return { vue, meta, expandedMeta, unresolved, unsupported, nodeCount, attrCount, hash };
}

function countNodes(node) {
  let count = 1;
  for (const [key, value] of Object.entries(node)) {
    if (key.startsWith("@_") || key.startsWith("#") || key.startsWith("?") || key.startsWith("__")) continue;
    const list = Array.isArray(value) ? value : [value];
    for (const item of list) {
      if (item && typeof item === "object") count += countNodes(item);
    }
  }
  return count;
}

function countAttrs(node) {
  let count = 0;
  for (const [key, value] of Object.entries(node)) {
    if (key.startsWith("@_")) {
      count += 1;
      continue;
    }
    if (key.startsWith("#") || key.startsWith("?") || key.startsWith("__")) continue;
    const list = Array.isArray(value) ? value : [value];
    for (const item of list) {
      if (item && typeof item === "object") count += countAttrs(item);
    }
  }
  return count;
}

function xmlChildren(node) {
  const out = [];
  for (const [key, value] of Object.entries(node)) {
    if (
      key.startsWith("@_") ||
      key.startsWith("#") ||
      key.startsWith("?") ||
      key.startsWith("__")
    ) {
      continue;
    }
    const list = Array.isArray(value) ? value : [value];
    for (const child of list) {
      if (typeof child !== "object" || child === null) continue;
      out.push({ tag: key.split(".").pop(), node: child });
    }
  }
  return out;
}

function rawAttrsOf(node) {
  const attrs = {};
  for (const [rawKey, rawValue] of Object.entries(node)) {
    if (!rawKey.startsWith("@_")) continue;
    attrs[normalizeName(rawKey)] = rawValue;
  }
  return attrs;
}

function recordAttrs(attrs, state) {
  const meta = [];
  for (const [name, raw] of Object.entries(attrs)) {
    if (isIgnored(name)) {
      meta.push({ name, raw, value: raw, status: "ignored", reason: "non-visual/platform attribute" });
      continue;
    }
    const resolved = resolveValue(raw, attrs);
    if (resolved.status === "unresolved") {
      meta.push({ name, raw, ...resolved });
    } else {
      meta.push({ name, raw, ...resolved, status: "mapped", target: "component prop" });
    }
  }
  state.meta.push(...meta);
  return meta;
}

function renderMenuNode(tag, node, state, depth = 0) {
  const attrs = rawAttrsOf(node);
  recordAttrs(attrs, state);
  const children = xmlChildren(node)
    .map((child) => renderMenuNode(child.tag, child.node, state, depth + 1))
    .join("\n");
  if (tag !== "item") {
    return `<div class="menu-group">${children}</div>`;
  }
  const title = attrs.title ? resolveValue(attrs.title, attrs).value ?? "" : "";
  const icon = attrs.icon ? iconName(attrs.icon) : "";
  const showAsAction = attrs.showAsAction ?? "never";
  const checkable = attrs.checkable === "true" || attrs.checkable === true;
  const checked = attrs.checked === "true" || attrs.checked === true;
  return `<div class="menu-item" data-depth="${depth}">
  ${icon ? `<AndroidIcon name="${escapeHtml(icon)}" :size="20" />` : ""}
  <span class="menu-title">{{ ${JSON.stringify(String(title))} }}</span>
  <span class="menu-meta">${escapeHtml(showAsAction)}${checkable ? " · checkable" : ""}${checked ? " · checked" : ""}</span>
  ${children ? `<div class="menu-children">${children}</div>` : ""}
</div>`;
}

function compileMenu(file, name) {
  const doc = parser.parse(fs.readFileSync(file, "utf8"));
  const { tag, node } = unwrapRoot(doc);
  const state = { meta: [] };
  const items = renderMenuNode(tag, node, state);
  const nodeCount = countNodes(node);
  const attrCount = countAttrs(node);
  const unresolved = state.meta.filter((item) => item.status === "unresolved");
  const hash = crypto.createHash("sha1").update(fs.readFileSync(file)).digest("hex");
  const vue = `<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-menu">
${items}
  </div>
</template>

<style scoped>
.generated-menu { width:100%; min-height:100%; background:var(--ac-background); color:var(--ac-primary-text); }
.menu-item { display:flex; align-items:center; gap:10px; padding:11px 16px; border-bottom:1px solid var(--ac-divider); }
.menu-title { flex:1; font-size:14px; }
.menu-meta { font-size:11px; color:var(--ac-secondary-text); }
.menu-group,.menu-children { display:flex; flex-direction:column; width:100%; }
.menu-children { padding-left:20px; }
</style>
`;
  return { vue, meta: state.meta, unresolved, unsupported: [], nodeCount, attrCount, hash };
}

function renderPrefNode(tag, node, state, depth = 0) {
  const attrs = rawAttrsOf(node);
  recordAttrs(attrs, state);
  const title = attrs.title ? resolveValue(attrs.title, attrs).value ?? "" : "";
  const summary = attrs.summary ? resolveValue(attrs.summary, attrs).value ?? "" : "";
  const icon = attrs.icon ? iconName(attrs.icon) : "";
  const key = attrs.key ?? "";
  const defaultValue = attrs.defaultValue ?? "";
  const entries = attrs.entries ? resolveValue(attrs.entries, attrs).value : [];
  const children = xmlChildren(node)
    .map((child) => renderPrefNode(child.tag, child.node, state, depth + 1))
    .join("\n");
  if (tag === "PreferenceCategory" || tag === "PreferenceScreen") {
    return `<section class="pref-group">
${title ? `<h2>{{ ${JSON.stringify(String(title))} }}</h2>` : ""}
${children}
</section>`;
  }
  const isSwitch = tag === "SwitchPreference";
  return `<div class="pref-row" data-depth="${depth}">
  ${icon ? `<AndroidIcon name="${escapeHtml(icon)}" :size="22" />` : ""}
  <div class="pref-text">
    <span class="pref-title">{{ ${JSON.stringify(String(title))} }}</span>
    ${summary ? `<span class="pref-summary">{{ ${JSON.stringify(String(summary))} }}</span>` : ""}
    ${key ? `<span class="pref-key">{{ ${JSON.stringify(String(key))} }}</span>` : ""}
    ${defaultValue ? `<span class="pref-default">榛樿锛歿{ ${JSON.stringify(String(defaultValue))} }}</span>` : ""}
    ${Array.isArray(entries) && entries.length ? `<span class="pref-entries">{{ ${JSON.stringify(entries.join(" / "))} }}</span>` : ""}
  </div>
  ${isSwitch ? `<input type="checkbox" />` : ""}
</div>`;
}

function compilePref(file, name) {
  const doc = parser.parse(fs.readFileSync(file, "utf8"));
  const { tag, node } = unwrapRoot(doc);
  const state = { meta: [] };
  const html = renderPrefNode(tag, node, state);
  const nodeCount = countNodes(node);
  const attrCount = countAttrs(node);
  const unresolved = state.meta.filter((item) => item.status === "unresolved");
  const hash = crypto.createHash("sha1").update(fs.readFileSync(file)).digest("hex");
  const vue = `<script setup lang="ts">
import AndroidIcon from "@/components/android/Icon.vue";
</script>

<template>
  <div class="generated-pref">
${html}
  </div>
</template>

<style scoped>
.generated-pref { width:100%; min-height:100%; background:var(--ac-background); color:var(--ac-primary-text); }
.pref-group { display:flex; flex-direction:column; }
.pref-group h2 { margin:0; padding:10px 20px 6px; font-size:13px; color:var(--ac-primary); }
.pref-row { display:flex; align-items:center; gap:12px; min-height:52px; padding:10px 20px; border-bottom:1px solid var(--ac-divider); }
.pref-text { flex:1; min-width:0; display:flex; flex-direction:column; gap:3px; }
.pref-title { font-size:15px; }
.pref-summary,.pref-key,.pref-default,.pref-entries { font-size:12px; color:var(--ac-secondary-text); }
</style>
`;
  return { vue, meta: state.meta, unresolved, unsupported: [], nodeCount, attrCount, hash };
}

fs.mkdirSync(OUT_GENERATED, { recursive: true });
fs.mkdirSync(OUT_REPORTS, { recursive: true });

const report = {
  generatedAt: new Date().toISOString(),
  layouts: [],
  menus: [],
  prefs: [],
  skipped: [],
  unresolved: [],
  unsupported: [],
};
const manifest = [];

for (const group of ["layout", "layout-land"]) {
  const dir = path.join(APP_RES, group);
  if (!fs.existsSync(dir)) continue;
  const files = fs.readdirSync(dir).filter((file) => file.endsWith(".xml")).sort();
  for (const file of files) {
    const name = path.basename(file, ".xml");
    const result = compileLayout(path.join(dir, file), group, name);
    const targetDir = path.join(OUT_GENERATED, group);
    writeFile(path.join(targetDir, `${name}.vue`), result.vue);
    writeFile(
      path.join(targetDir, `${name}.meta.json`),
      JSON.stringify(
        {
          source: `${group}/${file}`,
          hash: result.hash,
          nodeCount: result.nodeCount,
          attrCount: result.attrCount,
          meta: result.meta,
          expandedMeta: result.expandedMeta,
          unresolved: result.unresolved,
          unsupported: result.unsupported,
        },
        null,
        2,
      ),
    );
    report.layouts.push({
      group,
      name,
      nodeCount: result.nodeCount,
      attrCount: result.attrCount,
      unresolved: result.unresolved.length,
      unsupported: result.unsupported.length,
    });
    manifest.push({
      source: `${group}/${file}`,
      kind: group,
      name,
      component: `@/design/generated/${group}/${name}.vue`,
      nodeCount: result.nodeCount,
      attrCount: result.attrCount,
      status: "generated",
    });
    if (result.unresolved.length) {
      report.unresolved.push({ file: `${group}/${file}`, items: result.unresolved });
    }
    if (result.unsupported.length) {
      report.unsupported.push({ file: `${group}/${file}`, items: result.unsupported });
    }
  }
}

for (const group of [
  { dir: "menu", kind: "menus", compile: compileMenu },
  { dir: "xml", kind: "prefs", compile: compilePref },
]) {
  const dir = path.join(APP_RES, group.dir);
  if (!fs.existsSync(dir)) continue;
  const files = fs.readdirSync(dir)
    .filter((file) => file.endsWith(".xml"))
    .sort();
  for (const file of files) {
    const name = path.basename(file, ".xml");
    if (["file_paths.xml", "network_security_config.xml", "spen_remote_actions.xml"].includes(file)) {
      report.skipped.push({
        source: `${group.dir}/${file}`,
        reason: "闈炶瑙夐厤缃?XML",
      });
      manifest.push({
        source: `${group.dir}/${file}`,
        kind: group.dir,
        name,
        component: "",
        nodeCount: 0,
        attrCount: 0,
        status: "not-applicable",
      });
      continue;
    }
    const result = group.compile(path.join(dir, file), name);
    const targetDir = path.join(OUT_GENERATED, group.dir);
    writeFile(path.join(targetDir, `${name}.vue`), result.vue);
    writeFile(
      path.join(targetDir, `${name}.meta.json`),
      JSON.stringify(
        {
          source: `${group.dir}/${file}`,
          hash: result.hash,
          nodeCount: result.nodeCount,
          attrCount: result.attrCount,
          meta: result.meta,
          unresolved: result.unresolved,
          unsupported: result.unsupported,
        },
        null,
        2,
      ),
    );
    report[group.kind].push({
      name,
      nodeCount: result.nodeCount,
      attrCount: result.attrCount,
      unresolved: result.unresolved.length,
      unsupported: result.unsupported.length,
    });
    manifest.push({
      source: `${group.dir}/${file}`,
      kind: group.dir,
      name,
      component: `@/design/generated/${group.dir}/${name}.vue`,
      nodeCount: result.nodeCount,
      attrCount: result.attrCount,
      status: "generated",
    });
    if (result.unresolved.length) {
      report.unresolved.push({ file: `${group.dir}/${file}`, items: result.unresolved });
    }
    if (result.unsupported.length) {
      report.unsupported.push({ file: `${group.dir}/${file}`, items: result.unsupported });
    }
  }
}

writeFile(path.join(OUT_GENERATED, "manifest.json"), JSON.stringify(manifest, null, 2));
writeFile(path.join(OUT_REPORTS, "generate-report.json"), JSON.stringify(report, null, 2));
console.log(
  `generated layouts: ${report.layouts.length}, menus: ${report.menus.length}, prefs: ${report.prefs.length}, skipped: ${report.skipped.length}, unresolved files: ${report.unresolved.length}, unsupported files: ${report.unsupported.length}`,
);
if (report.unresolved.length) {
  process.exitCode = 1;
}

