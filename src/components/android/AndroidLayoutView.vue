<script setup lang="ts">
import { computed } from "vue";
import AndroidLayoutNode, { type AndroidNode } from "./AndroidLayoutNode.vue";

const props = withDefaults(
  defineProps<{
    name: string;
    group?: string;
  }>(),
  { group: "layout" },
);

const modules = import.meta.glob("../../design/layouts/**/*.json", {
  eager: true,
  import: "default",
}) as Record<string, AndroidNode>;

const tree = computed<AndroidNode | null>(() => {
  const key = `../../design/layouts/${props.group}/${props.name}.json`;
  return modules[key] ?? null;
});
</script>

<template>
  <div class="android-layout">
    <AndroidLayoutNode v-if="tree" :node="tree" />
    <div v-else class="missing">未找到布局：{{ group }}/{{ name }}</div>
  </div>
</template>

<style scoped>
.android-layout {
  width: 100%;
  min-height: 100%;
  background: var(--ac-background);
  color: var(--ac-primary-text);
  overflow: auto;
}
.missing {
  padding: 24px;
  text-align: center;
  color: var(--ac-secondary-text);
}
</style>
