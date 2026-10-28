<script setup lang="ts">
import { computed, defineAsyncComponent, ref, type Component } from "vue";
import { useRoute, useRouter } from "vue-router";
import navigationGraph from "@/design/reports/navigation-graph.json";

interface GraphNode {
  id: string;
  class: string;
  kind: "activity" | "fragment" | "dialog";
  route: string;
}

interface GraphEdge {
  from: string;
  target: string;
  kind: "activity" | "fragment" | "dialog";
  trigger: string;
  triggerTitle: string;
  targetRoute: string;
  source: string;
}

const graph = navigationGraph as unknown as {
  start: string;
  nodes: GraphNode[];
  edges: GraphEdge[];
  stats: Record<string, number>;
};

const modules = import.meta.glob("@/design/generated/layout/*.vue");
const componentCache = new Map<string, Component>();

const route = useRoute();
const router = useRouter();
const browserVisible = ref(false);
const browserKeyword = ref("");

const name = computed(() => String(route.params.name ?? ""));
const node = computed(() => graph.nodes.find((item) => item.id === name.value) ?? null);
const outgoing = computed(() => graph.edges.filter((edge) => edge.from === name.value));

const currentComponent = computed(() => {
  if (!node.value) return null;
  const cached = componentCache.get(node.value.id);
  if (cached) return cached;
  const entry = Object.entries(modules).find(([path]) => path.endsWith(`/${node.value!.id}.vue`));
  if (!entry) return null;
  const component = defineAsyncComponent(entry[1] as () => Promise<{ default: Component }>);
  componentCache.set(node.value.id, component);
  return component;
});

const groupedNodes = computed(() => {
  const keyword = browserKeyword.value.trim().toLowerCase();
  const matched = graph.nodes.filter(
    (item) =>
      !keyword ||
      item.id.toLowerCase().includes(keyword) ||
      item.class.toLowerCase().includes(keyword),
  );
  const groups: { kind: string; label: string; nodes: GraphNode[] }[] = [
    { kind: "activity", label: "Activity", nodes: [] },
    { kind: "fragment", label: "Fragment", nodes: [] },
    { kind: "dialog", label: "Dialog", nodes: [] },
  ];
  for (const item of matched) {
    groups.find((group) => group.kind === item.kind)?.nodes.push(item);
  }
  return groups;
});

function shortName(id: string) {
  return id.replace(/^(activity|fragment|dialog)_/, "");
}

function go(id: string) {
  router.push(`/shell/${id}`);
}
</script>

<template>
  <div class="shell">
    <header class="shell-bar">
      <button class="back" @click="router.back()">‹ 返回</button>
      <div class="title">
        <span class="name">{{ name }}</span>
        <span v-if="node" class="kind" :data-kind="node.kind">{{ node.kind }}</span>
        <span class="class-name" v-if="node">{{ node.class }}</span>
      </div>
      <button class="browser-toggle" @click="browserVisible = !browserVisible">
        {{ browserVisible ? "关闭目录" : `目录 ${graph.nodes.length}` }}
      </button>
    </header>

    <main class="shell-body">
      <component :is="currentComponent" v-if="currentComponent" />
      <div v-else-if="node" class="missing">布局 {{ name }} 无生成组件</div>
      <div v-else class="missing">
        <p>未知页面：{{ name }}</p>
        <button @click="go(graph.start)">回到起点</button>
      </div>
    </main>

    <footer class="shell-edges">
      <span class="edges-label">跳转 ({{ outgoing.length }})</span>
      <div class="edges-scroll">
        <button
          v-for="edge in outgoing"
          :key="`${edge.target}-${edge.trigger}-${edge.source}`"
          class="edge"
          :data-kind="edge.kind"
          @click="go(edge.target)"
        >
          <span class="edge-trigger">{{ edge.triggerTitle || edge.trigger || "直接调用" }}</span>
          <span class="edge-target">{{ shortName(edge.target) }}</span>
        </button>
        <span v-if="outgoing.length === 0" class="edges-empty">无出边（终点页）</span>
      </div>
    </footer>

    <div v-if="browserVisible" class="browser">
      <div class="browser-panel">
        <div class="browser-head">
          <input v-model="browserKeyword" placeholder="搜索 id / 类名" />
          <button @click="browserVisible = false">×</button>
        </div>
        <div class="browser-body">
          <section v-for="group in groupedNodes" :key="group.kind">
            <h3 v-if="group.nodes.length">{{ group.label }} · {{ group.nodes.length }}</h3>
            <button v-for="item in group.nodes" :key="item.id" :class="{ active: item.id === name }" @click="go(item.id)">
              <span>{{ item.id }}</span>
              <small>{{ item.class }}</small>
            </button>
          </section>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.shell {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
  color: var(--ac-primary-text);
}

.shell-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 48px;
  padding: 0 12px;
  background: var(--ac-primary);
  color: #fff;
}

.back {
  border: none;
  background: transparent;
  color: inherit;
  font-size: 14px;
  cursor: pointer;
  white-space: nowrap;
}

.title {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  overflow: hidden;
}

.name {
  font-size: 14px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.kind {
  flex: none;
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.2);
}

.class-name {
  font-size: 11px;
  opacity: 0.75;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.browser-toggle {
  flex: none;
  border: 1px solid rgba(255, 255, 255, 0.5);
  background: transparent;
  color: inherit;
  font-size: 12px;
  padding: 4px 10px;
  border-radius: 2px;
  cursor: pointer;
}

.shell-body {
  flex: 1;
  min-height: 0;
  overflow: auto;
  position: relative;
}

.missing {
  padding: 40px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.shell-edges {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--ac-background-menu);
  border-top: 1px solid var(--ac-divider);
}

.edges-label {
  flex: none;
  font-size: 11px;
  color: var(--ac-secondary-text);
}

.edges-scroll {
  flex: 1;
  min-width: 0;
  display: flex;
  gap: 6px;
  overflow-x: auto;
}

.edge {
  flex: none;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 1px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  border-radius: 3px;
  padding: 4px 10px;
  cursor: pointer;
}

.edge:hover {
  border-color: var(--ac-primary);
}

.edge[data-kind="dialog"] {
  border-style: dashed;
}

.edge-trigger {
  font-size: 12px;
  max-width: 130px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.edge-target {
  font-size: 10px;
  color: var(--ac-secondary-text);
  max-width: 130px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.edges-empty {
  font-size: 11px;
  color: var(--ac-secondary-text);
}

.browser {
  position: fixed;
  inset: 0;
  z-index: 60;
  background: rgba(0, 0, 0, 0.3);
  display: flex;
  justify-content: flex-end;
}

.browser-panel {
  width: min(380px, 90vw);
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
  border-left: 1px solid var(--ac-divider);
}

.browser-head {
  flex: none;
  display: flex;
  gap: 8px;
  padding: 10px 12px;
  border-bottom: 1px solid var(--ac-divider);
}

.browser-head input {
  flex: 1;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  padding: 6px 8px;
  background: var(--ac-background);
  color: inherit;
  font-size: 13px;
}

.browser-head button {
  border: none;
  background: transparent;
  font-size: 18px;
  cursor: pointer;
  color: inherit;
}

.browser-body {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}

.browser-body h3 {
  margin: 0;
  padding: 8px 16px 4px;
  font-size: 11px;
  color: var(--ac-primary);
}

.browser-body button {
  width: 100%;
  display: flex;
  align-items: baseline;
  gap: 8px;
  border: none;
  background: transparent;
  color: inherit;
  padding: 8px 16px;
  cursor: pointer;
  text-align: left;
}

.browser-body button span {
  font-size: 13px;
}

.browser-body button small {
  font-size: 10px;
  color: var(--ac-secondary-text);
  flex: 1;
  min-width: 0;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.browser-body button.active {
  background: var(--ac-transparent-10);
}
</style>
