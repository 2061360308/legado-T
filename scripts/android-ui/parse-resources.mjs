import path from "node:path";
import {
  APP_RES,
  OUT_DESIGN,
  dimensionToPx,
  loadResourceBundle,
  writeFile,
} from "./android.mjs";

const bundle = loadResourceBundle();

function cssColor(value) {
  if (typeof value !== "string") return value;
  const match = value.match(/^#([0-9a-fA-F]{8})$/);
  if (!match) return value;
  const argb = match[1];
  return `#${argb.slice(2)}${argb.slice(0, 2)}`;
}

function colorBlock(map) {
  const lines = [];
  for (const [key, value] of Object.entries(map)) {
    lines.push(`  --ac-${key}: ${cssColor(value)};`);
  }
  return lines.join("\n");
}

function aliasBlock(map) {
  const aliases = {
    primary: "primary",
    primaryDark: "primaryDark",
    accent: "accent",
    background: "background",
    "background-card": "background_card",
    "background-menu": "background_menu",
    "primary-text": "primaryText",
    "secondary-text": "secondaryText",
    "summary-text": "tv_text_summary",
    divider: "divider",
    error: "error",
    success: "success",
    white: "white",
    black: "black",
    transparent: "transparent",
    "transparent-10": "transparent10",
    "transparent-20": "transparent20",
    "transparent-30": "transparent30",
    "transparent-50": "transparent50",
  };
  return Object.entries(aliases)
    .filter(([alias, source]) => alias !== source && map[source] !== undefined)
    .map(([alias, source]) => `  --ac-${alias}: var(--ac-${source});`)
    .join("\n");
}

const dimenLines = Object.entries(bundle.dimens)
  .map(([key, value]) => {
    const px = dimensionToPx(value);
    return typeof px === "number" ? `  --d-${key}: ${px}px;` : `  --d-${key}: ${value};`;
  })
  .join("\n");

const css = `/* Generated from app/src/main/res. Do not edit by hand. */
:root {
  --android-design-width: 360px;
  --android-design-height: 800px;
  --android-density: 1;
${colorBlock(bundle.colorsLight)}
${aliasBlock(bundle.colorsLight)}
${dimenLines}
}

html.theme-night,
html.dark {
${colorBlock(bundle.colorsNight)}
${aliasBlock(bundle.colorsNight)}
}
`;

writeFile(path.join(OUT_DESIGN, "tokens.css"), css);
writeFile(path.join(OUT_DESIGN, "strings.json"), JSON.stringify(bundle.strings, null, 2));
writeFile(
  path.join(OUT_DESIGN, "resources.json"),
  JSON.stringify(
    {
      colorsLight: bundle.colorsLight,
      colorsNight: bundle.colorsNight,
      dimens: bundle.dimens,
      strings: bundle.strings,
      arrays: bundle.arrays,
    },
    null,
    2,
  ),
);
console.log(`tokens: ${Object.keys(bundle.colorsLight).length} colors, ${Object.keys(bundle.dimens).length} dimens`);
