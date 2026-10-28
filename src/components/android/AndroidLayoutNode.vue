<script setup lang="ts">
import { computed } from "vue";
import Icon from "./Icon.vue";

export interface AndroidNode {
  tag: string;
  attrs: Record<string, string | number | boolean>;
  children: AndroidNode[];
}

const props = defineProps<{ node: AndroidNode; depth?: number }>();

const px = (value: unknown): string | undefined => {
  if (typeof value === "number") return `${value}px`;
  if (typeof value !== "string" || !value) return undefined;
  if (value === "match_parent") return "100%";
  if (value === "wrap_content") return "auto";
  if (/^-?\d+(\.\d+)?$/.test(value)) return `${value}px`;
  return value;
};

const sizeStyle = computed(() => {
  const width = px(props.node.attrs.layout_width);
  const height = px(props.node.attrs.layout_height);
  return {
    width,
    height,
  };
});

const spacingStyle = computed(() => {
  const a = props.node.attrs;
  const style: Record<string, string> = {};
  const margin = (name: string) => px(a[name]);
  const padding = (name: string) => px(a[name]);
  if (margin("layout_margin")) style.margin = margin("layout_margin")!;
  else {
    const top = margin("layout_marginTop") ?? margin("layout_marginVertical");
    const bottom = margin("layout_marginBottom") ?? margin("layout_marginVertical");
    const left = margin("layout_marginLeft") ?? margin("layout_marginStart") ?? margin("layout_marginHorizontal");
    const right = margin("layout_marginRight") ?? margin("layout_marginEnd") ?? margin("layout_marginHorizontal");
    if (top) style.marginTop = top;
    if (bottom) style.marginBottom = bottom;
    if (left) style.marginLeft = left;
    if (right) style.marginRight = right;
  }
  if (padding("padding")) style.padding = padding("padding")!;
  else {
    const top = padding("paddingTop") ?? padding("paddingVertical");
    const bottom = padding("paddingBottom") ?? padding("paddingVertical");
    const left = padding("paddingLeft") ?? padding("paddingStart") ?? padding("paddingHorizontal");
    const right = padding("paddingRight") ?? padding("paddingEnd") ?? padding("paddingHorizontal");
    if (top) style.paddingTop = top;
    if (bottom) style.paddingBottom = bottom;
    if (left) style.paddingLeft = left;
    if (right) style.paddingRight = right;
  }
  return style;
});

const textStyle = computed(() => {
  const a = props.node.attrs;
  const style: Record<string, string> = {};
  if (a.textSize) style.fontSize = px(a.textSize)!;
  if (a.textColor) style.color = String(a.textColor);
  if (String(a.textStyle ?? "").includes("bold")) style.fontWeight = "600";
  const gravity = String(a.gravity ?? a.layout_gravity ?? "");
  if (gravity.includes("center")) style.textAlign = "center";
  else if (gravity.includes("right") || gravity.includes("end")) style.textAlign = "right";
  if (gravity.includes("center_vertical") || gravity.includes("center")) style.alignSelf = "center";
  return style;
});

const background = computed(() => {
  const value = props.node.attrs.background;
  if (typeof value === "string" && (value.startsWith("#") || value.startsWith("rgb"))) {
    return value;
  }
  return undefined;
});

const tag = computed(() => props.node.tag);
const isText = computed(() =>
  [
    "TextView",
    "Button",
    "RadioButton",
    "CheckBox",
    "SwitchCompat",
    "ThemeSwitch",
    "ThemeCheckBox",
    "ThemeRadioButton",
    "AccentTextView",
    "AccentBgTextView",
    "StrokeTextView",
    "BadgeView",
    "BevelLabelView",
  ].includes(tag.value),
);
const isEdit = computed(() => ["EditText", "ThemeEditText"].includes(tag.value));
const isImage = computed(() =>
  ["ImageView", "AppCompatImageView", "ImageButton", "CircleImageView", "CoverImageView"].includes(tag.value),
);
const isSwitch = computed(() =>
  ["SwitchCompat", "ThemeSwitch", "ThemeCheckBox", "ThemeRadioButton", "CheckBox"].includes(tag.value),
);
const isList = computed(() =>
  ["RecyclerView", "FastScrollRecyclerView", "RecyclerViewAtPager2", "ViewPager"].includes(tag.value),
);
const isTitleBar = computed(() => tag.value === "TitleBar");

const text = computed(() => {
  const value = props.node.attrs.text ?? props.node.attrs.hint;
  return value === undefined ? "" : String(value);
});

const iconName = computed(() => {
  const src = props.node.attrs.src;
  if (typeof src === "string" && src) return src;
  return "";
});

const layoutStyle = computed(() => ({
  ...sizeStyle.value,
  ...spacingStyle.value,
  ...(background.value ? { background: background.value } : {}),
  ...(props.node.attrs.layout_weight
    ? { flexGrow: Number(props.node.attrs.layout_weight), flexShrink: 1, flexBasis: "0%" }
    : {}),
}));

const direction = computed(() =>
  String(props.node.attrs.orientation ?? "") === "horizontal" ? "row" : "column",
);

const justifyContent = computed(() => {
  const gravity = String(props.node.attrs.gravity ?? "");
  if (gravity.includes("center")) return "center";
  if (gravity.includes("bottom") || gravity.includes("end")) return "flex-end";
  return "flex-start";
});

const alignItems = computed(() => {
  const gravity = String(props.node.attrs.gravity ?? "");
  if (gravity.includes("center_horizontal") || gravity.includes("center")) return "center";
  if (gravity.includes("right") || gravity.includes("end")) return "flex-end";
  return "stretch";
});
</script>

<template>
  <div
    v-if="node.attrs.visibility !== 'gone'"
    class="android-node"
    :class="{
      text: isText,
      image: isImage,
      list: isList,
      titlebar: isTitleBar,
    }"
    :style="{
      ...layoutStyle,
      ...(isText || isEdit ? textStyle : {}),
      ...(node.attrs.visibility === 'invisible' ? { visibility: 'hidden' } : {}),
    }"
  >
    <template v-if="isTitleBar">
      <span class="titlebar-title">{{ node.attrs.title || "标题" }}</span>
    </template>
    <template v-else-if="isSwitch">
      <span class="node-text">{{ text }}</span>
      <span class="switch-demo" />
    </template>
    <template v-else-if="isEdit">
      <input :placeholder="String(node.attrs.hint ?? '')" />
    </template>
    <template v-else-if="isImage">
      <Icon v-if="iconName" :name="iconName" :size="Number(node.attrs.textSize) || 24" />
      <span v-else class="image-placeholder">图</span>
    </template>
    <template v-else-if="isText">
      {{ text || tag }}
    </template>
    <template v-else-if="isList">
      <div class="list-placeholder">列表项 / 列表项 / 列表项</div>
    </template>
    <template v-else>
      <div
        class="node-children"
        :style="{
          display: 'flex',
          flexDirection: direction,
          justifyContent,
          alignItems,
        }"
      >
        <AndroidLayoutNode
          v-for="(child, index) in node.children"
          :key="index"
          :node="child"
          :depth="(depth ?? 0) + 1"
        />
      </div>
    </template>
  </div>
</template>

<style scoped>
.android-node {
  position: relative;
  min-width: 0;
  min-height: 0;
  box-sizing: border-box;
}
.android-node.text {
  display: inline-flex;
  align-items: center;
  min-height: 18px;
  color: var(--ac-primary-text);
  font-size: 14px;
}
.android-node.titlebar {
  display: flex;
  align-items: center;
  height: 56px;
  padding: 0 16px;
  background: var(--ac-primary);
  color: #fff;
  font-size: 20px;
}
.android-node.image {
  display: flex;
  align-items: center;
  justify-content: center;
  min-width: 24px;
  min-height: 24px;
}
.image-placeholder {
  width: 24px;
  height: 24px;
  border: 1px dashed var(--ac-divider);
  color: var(--ac-secondary-text);
  font-size: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.node-children {
  width: 100%;
  height: 100%;
  min-width: 0;
}
.node-text {
  flex: 1;
  font-size: 14px;
}
.switch-demo {
  width: 36px;
  height: 20px;
  border-radius: 10px;
  background: var(--ac-divider);
  position: relative;
}
.switch-demo::after {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  top: 2px;
  border-radius: 50%;
  background: #fff;
}
input {
  width: 100%;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 6px 8px;
}
.list-placeholder {
  padding: 10px 16px;
  color: var(--ac-secondary-text);
  font-size: 13px;
  border-top: 1px solid var(--ac-divider);
  border-bottom: 1px solid var(--ac-divider);
}
</style>
