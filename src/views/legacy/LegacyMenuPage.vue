<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

interface MenuNode {
  tag: string;
  attrs: Record<string, string>;
  children: MenuNode[];
}

const route = useRoute();
const router = useRouter();
const name = computed(() => String(route.params.name || ""));
const modules = import.meta.glob("../../design/menus/*.json", {
  eager: true,
  import: "default",
}) as Record<string, MenuNode>;
const tree = computed(() => modules[`../../design/menus/${name.value}.json`] ?? null);
const items = computed(() => tree.value?.children ?? []);
</script>

<template>
  <div class="menu-page">
    <TitleBar :title="`菜单：${name}`" @back="router.back()" />
    <div class="banner">原版菜单结构壳子 · 功能暂未接入</div>
    <main class="list">
      <div v-for="(item, index) in items" :key="index" class="menu-row">
        <Icon v-if="item.attrs.icon" :name="item.attrs.icon" :size="22" />
        <span class="text">{{ item.attrs.title || item.attrs.id || "菜单项" }}</span>
        <span class="mode">{{ item.attrs.showAsAction || "never" }}</span>
      </div>
      <div v-if="items.length === 0" class="empty">无可视菜单项</div>
    </main>
  </div>
</template>

<style scoped>
.menu-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.banner {
  flex: none;
  padding: 6px 16px;
  font-size: 12px;
  color: var(--ac-secondary-text);
  background: var(--ac-background-card);
  border-bottom: 1px solid var(--ac-divider);
}
.list {
  flex: 1;
  overflow-y: auto;
}
.menu-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--ac-divider);
  color: var(--ac-primary-text);
  font-size: 14px;
}
.text {
  flex: 1;
}
.mode {
  font-size: 11px;
  color: var(--ac-secondary-text);
}
.empty {
  padding: 32px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
