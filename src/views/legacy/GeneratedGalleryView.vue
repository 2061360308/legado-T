<script setup lang="ts">
import { computed } from "vue";
import { useRouter } from "vue-router";
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

const router = useRouter();
const entries = manifest as ManifestEntry[];
const groups = computed(() => {
  const map = new Map<string, ManifestEntry[]>();
  for (const entry of entries) {
    const list = map.get(entry.kind) ?? [];
    list.push(entry);
    map.set(entry.kind, list);
  }
  return [...map.entries()];
});

function open(entry: ManifestEntry) {
  if (entry.status !== "generated") return;
  router.push(`/generated/${entry.kind}/${entry.name}`);
}
</script>

<template>
  <div class="generated-gallery">
    <TitleBar title="生成组件清单" @back="router.back()" />
    <main class="list">
      <section v-for="[kind, items] in groups" :key="kind">
        <h2>{{ kind }} · {{ items.length }}</h2>
        <button
          v-for="entry in items"
          :key="entry.source"
          :disabled="entry.status !== 'generated'"
          @click="open(entry)"
        >
          <span>{{ entry.name }}</span>
          <span class="meta">
            {{ entry.status === "generated" ? `${entry.nodeCount} / ${entry.attrCount}` : "不适用" }}
          </span>
        </button>
      </section>
    </main>
  </div>
</template>

<style scoped>
.generated-gallery {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.list {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 24px;
}
h2 {
  margin: 0;
  padding: 10px 20px 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--ac-primary);
}
button {
  width: 100%;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 11px 20px;
  font-size: 13px;
  text-align: left;
  cursor: pointer;
}
button:hover {
  background: var(--ac-transparent-10);
}
button:disabled {
  color: var(--ac-secondary-text);
  cursor: default;
}
.meta {
  font-size: 11px;
  color: var(--ac-secondary-text);
}
</style>
