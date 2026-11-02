<script setup lang="ts">
import { ref } from "vue";
import Icon from "./Icon.vue";
import PopupMenu, { type PopupMenuItem } from "./PopupMenu.vue";

withDefaults(
  defineProps<{
    count: number;
    total: number;
    mainText?: string;
    menuItems?: PopupMenuItem[];
  }>(),
  {
    mainText: "",
    menuItems: () => [],
  },
);

const emit = defineEmits<{
  selectAll: [value: boolean];
  revert: [];
  main: [];
  menu: [key: string];
}>();

const menuVisible = ref(false);

function onSelectAll(value: boolean) {
  emit("selectAll", value);
}

function onMenu(key: string) {
  menuVisible.value = false;
  emit("menu", key);
}
</script>

<template>
  <div class="action-bar">
    <label class="select-all">
      <input
        type="checkbox"
        :checked="total > 0 && count >= total"
        :disabled="!total"
        @change="onSelectAll(($event.target as HTMLInputElement).checked)"
      />
      <span>
        {{ total > 0 && count >= total ? "取消全选" : "全选" }}（{{ count }}/{{ total }}）
      </span>
    </label>
    <button class="outline" :disabled="!count" @click="emit('revert')">反选</button>
    <button v-if="mainText" class="outline" :disabled="!count" @click="emit('main')">
      {{ mainText }}
    </button>
    <button
      v-if="menuItems.length"
      class="icon"
      aria-label="更多"
      :disabled="!count"
      @click="menuVisible = true"
    >
      <Icon name="ic_more_vert" :size="22" />
    </button>
    <PopupMenu
      v-model="menuVisible"
      :items="menuItems"
      :bottom="58"
      :right="8"
      @select="onMenu"
    />
  </div>
</template>

<style scoped>
.action-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 8px calc(6px + env(safe-area-inset-bottom)) 16px;
  background: var(--ac-background-menu);
  box-shadow: 0 -2px 2px rgba(0, 0, 0, 0.18);
}
.select-all {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  color: var(--ac-primary-text);
}
.select-all input {
  flex: none;
  accent-color: var(--ac-primary);
}
.outline {
  flex: none;
  min-width: 72px;
  border: 1px solid var(--ac-primary);
  border-radius: 2px;
  background: transparent;
  color: var(--ac-primary);
  padding: 6px 10px;
  font-size: 13px;
  cursor: pointer;
}
.icon {
  flex: none;
  width: 36px;
  height: 36px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--ac-primary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.icon:hover {
  background: var(--ac-transparent-10);
}
.outline:disabled,
.icon:disabled {
  opacity: 0.35;
  cursor: default;
}
</style>
