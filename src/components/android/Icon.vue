<script setup lang="ts">
import { computed } from "vue";

const props = withDefaults(
  defineProps<{
    name: string;
    size?: number;
    checked?: boolean;
    color?: string;
  }>(),
  {
    size: 24,
    checked: false,
    color: "currentColor",
  },
);

const modules = import.meta.glob("../../assets/android/drawable/*.{svg,png,jpg,jpeg,webp}", {
  eager: true,
  query: "?url",
  import: "default",
}) as Record<string, string>;

function find(name: string): string | undefined {
  for (const [key, url] of Object.entries(modules)) {
    if (key.endsWith(`/${name}.svg`) || key.endsWith(`/${name}.png`) || key.endsWith(`/${name}.jpg`) || key.endsWith(`/${name}.webp`)) {
      return url;
    }
  }
  return undefined;
}

const url = computed(() => {
  const candidates = props.checked
    ? [`${props.name}-checked`, `${props.name}_s`, `${props.name}`]
    : [`${props.name}`, `${props.name}_e`];
  for (const candidate of candidates) {
    const found = find(candidate);
    if (found) return found;
  }
  return "";
});
</script>

<template>
  <span
    class="android-icon"
    :style="{
      width: `${size}px`,
      height: `${size}px`,
      backgroundColor: color,
      '--icon-mask': url ? `url(${url})` : 'none',
    }"
  />
</template>

<style scoped>
.android-icon {
  display: inline-block;
  flex: none;
  -webkit-mask: var(--icon-mask) center / contain no-repeat;
  mask: var(--icon-mask) center / contain no-repeat;
}
</style>
