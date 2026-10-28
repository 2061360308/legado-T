<script setup lang="ts">
import Icon from "./Icon.vue";

export type MainTab = "bookshelf" | "discover" | "rss" | "config";

defineProps<{ modelValue: MainTab }>();
const emit = defineEmits<{ "update:modelValue": [value: MainTab] }>();

const items = [
  { key: "bookshelf", icon: "ic_bottom_books", label: "书架", enabled: true },
  { key: "discover", icon: "ic_bottom_explore", label: "发现", enabled: true },
  { key: "rss", icon: "ic_bottom_rss_feed", label: "订阅", enabled: true },
  { key: "config", icon: "ic_bottom_person", label: "设置", enabled: true },
] as const;

function select(item: (typeof items)[number]) {
  if (!item.enabled) return;
  emit("update:modelValue", item.key as MainTab);
}
</script>

<template>
  <nav class="bottom-nav">
    <button
      v-for="item in items"
      :key="item.key"
      class="nav-item"
      :class="{ active: modelValue === item.key, disabled: !item.enabled }"
      :aria-label="item.label"
      @click="select(item)"
    >
      <Icon :name="item.icon" :checked="modelValue === item.key" :size="24" />
    </button>
  </nav>
</template><style scoped>
.bottom-nav {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  z-index: 40;
  display: flex;
  align-items: stretch;
  height: calc(50px + env(safe-area-inset-bottom));
  padding-bottom: env(safe-area-inset-bottom);
  background: var(--ac-background);
  border-top: 1px solid var(--ac-divider);
}
.nav-item {
  flex: 1;
  border: none;
  background: transparent;
  color: var(--ac-summary-text);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
}
.nav-item.active {
  color: var(--ac-primary);
}
.nav-item.disabled {
  opacity: 0.35;
  cursor: default;
}
</style>

