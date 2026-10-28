<script setup lang="ts">
import { computed, defineAsyncComponent, type Component } from "vue";
import { useRoute, useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import manifest from "@/design/generated/manifest.json";

interface ManifestEntry {
  source: string;
  kind: string;
  name: string;
  component: string;
  nodeCount: number;
  attrCount: number;
  status: string;
}

const route = useRoute();
const router = useRouter();
const entries = manifest as ManifestEntry[];

const modules = import.meta.glob("../../design/generated/**/*.vue") as Record<
  string,
  () => Promise<{ default: Component }>
>;

const components = Object.fromEntries(
  Object.entries(modules).map(([key, loader]) => [key, defineAsyncComponent(loader)]),
);

const kind = computed(() => String(route.params.kind || "layout"));
const name = computed(() => String(route.params.name || ""));
const entry = computed(
  () => entries.find((item) => item.kind === kind.value && item.name === name.value) ?? null,
);
const componentKey = computed(() => `../../design/generated/${kind.value}/${name.value}.vue`);
const component = computed(() => components[componentKey.value] ?? null);
const title = computed(() => `${kind.value} / ${name.value}`);
</script>

<template>
  <div class="generated-page">
    <TitleBar :title="title" @back="router.back()">
      <template #actions>
        <span class="meta">
          {{ entry?.nodeCount ?? 0 }} nodes · {{ entry?.attrCount ?? 0 }} attrs
        </span>
      </template>
    </TitleBar>

    <main class="body">
      <component :is="component" v-if="component" />
      <div v-else class="missing">
        <p>未找到生成组件：{{ kind }}/{{ name }}</p>
        <p v-if="entry?.status === 'not-applicable'">该 XML 为非视觉配置，已标记为不适用。</p>
      </div>
    </main>
  </div>
</template>

<style scoped>
.generated-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.meta {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.75);
  padding: 0 8px;
}
.body {
  flex: 1;
  min-height: 0;
  overflow: auto;
}
.missing {
  padding: 32px;
  text-align: center;
  color: var(--ac-secondary-text);
}
</style>
