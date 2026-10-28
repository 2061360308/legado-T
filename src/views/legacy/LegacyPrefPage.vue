<script setup lang="ts">
import { computed } from "vue";
import { useRoute, useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import PreferenceRow from "@/components/android/PreferenceRow.vue";
import PreferenceCategory from "@/components/android/PreferenceCategory.vue";

interface PrefNode {
  tag: string;
  title: string;
  summary: string;
  icon: string;
  key: string;
  defaultValue: string;
  children: PrefNode[];
}

const route = useRoute();
const router = useRouter();
const name = computed(() => String(route.params.name || ""));
const modules = import.meta.glob("../../design/prefs/*.json", {
  eager: true,
  import: "default",
}) as Record<string, PrefNode>;
const tree = computed(() => modules[`../../design/prefs/${name.value}.json`] ?? null);
const nodes = computed(() => tree.value?.children ?? []);
</script>

<template>
  <div class="pref-page">
    <TitleBar :title="`设置：${name}`" @back="router.back()" />
    <div class="banner">原版 Preference 结构壳子 · 功能暂未接入</div>
    <main class="list">
      <template v-for="(node, index) in nodes" :key="index">
        <PreferenceCategory v-if="node.children.length > 0" :title="node.title">
          <PreferenceRow
            v-for="(child, childIndex) in node.children"
            :key="childIndex"
            :label="child.title || child.key"
            :summary="child.summary"
            :icon="child.icon"
            chevron
          />
        </PreferenceCategory>
        <PreferenceRow
          v-else
          :label="node.title || node.key"
          :summary="node.summary"
          :icon="node.icon"
          chevron
        />
      </template>
      <div v-if="nodes.length === 0" class="empty">该 XML 不是可视 Preference 页面</div>
    </main>
  </div>
</template>

<style scoped>
.pref-page {
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
  padding-bottom: 24px;
}
.empty {
  padding: 32px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
