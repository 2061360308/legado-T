<script setup lang="ts">
import { computed, onActivated, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import {
  rssSourceDelete,
  rssSourceList,
  rssSourceSave,
} from "@/core/application/rss";
import type { RssSource } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import SearchBar from "@/components/android/SearchBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import legadoIcon from "@/assets/android/drawable/image_legado.png";

const router = useRouter();
const rssGridRef = ref<HTMLElement | null>(null);
useScrollMemory(rssGridRef);
const sources = ref<RssSource[]>([]);
const keyword = ref("");
const loading = ref(false);
const message = ref("");
const menuVisible = ref(false);
const groupVisible = ref(false);
const menuSource = ref<RssSource | null>(null);
const menuTop = ref(52);
const menuRight = ref(8);
let pressTimer: number | undefined;
let longPressed = false;

const enabledSources = computed(() =>
  sources.value.filter((source) => source.enabled !== false),
);

const groups = computed(() => {
  const values = new Set<string>();
  for (const source of enabledSources.value) {
    for (const group of String(source.sourceGroup || "").split(/[,，;；]/)) {
      const name = group.trim();
      if (name) values.add(name);
    }
  }
  return [...values].sort((a, b) => a.localeCompare(b, "zh-CN"));
});

const filtered = computed(() => {
  const key = keyword.value.trim();
  if (!key) return enabledSources.value;
  if (key.startsWith("group:")) {
    const group = key.slice(6).trim();
    return enabledSources.value.filter((source) =>
      String(source.sourceGroup || "")
        .split(/[,，;；]/)
        .map((value) => value.trim())
        .includes(group),
    );
  }
  const lower = key.toLowerCase();
  return enabledSources.value.filter((source) => {
    const name = String(source.sourceName || "").toLowerCase();
    const group = String(source.sourceGroup || "").toLowerCase();
    return name.includes(lower) || group.includes(lower);
  });
});

const groupItems = computed<PopupMenuItem[]>(() =>
  groups.value.map((group) => ({
    key: `group:${group}`,
    label: group,
    checked: keyword.value.trim() === `group:${group}`,
  })),
);

const menuItems: PopupMenuItem[] = [
  { key: "top", label: "置顶" },
  { key: "edit", label: "编辑" },
  { key: "disable", label: "禁用" },
  { key: "delete", label: "删除", danger: true },
];

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2500);
}

async function load() {
  loading.value = true;
  try {
    sources.value = await rssSourceList();
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

function openSource(source: RssSource) {
  if (longPressed) {
    longPressed = false;
    return;
  }
  if (source.singleUrl) {
    router.push({
      path: "/rss-read",
      query: {
        sourceUrl: source.sourceUrl,
        title: source.sourceName,
        link: source.sourceUrl,
      },
    });
    return;
  }
  router.push({
    path: "/rss-articles",
    query: { sourceUrl: source.sourceUrl, sourceName: source.sourceName },
  });
}

function startPress(source: RssSource, event: PointerEvent | MouseEvent) {
  window.clearTimeout(pressTimer);
  longPressed = false;
  const x = event.clientX || window.innerWidth - 24;
  const y = event.clientY || 56;
  pressTimer = window.setTimeout(() => {
    longPressed = true;
    openMenu(source, x, y);
  }, 480);
}

function cancelPress() {
  window.clearTimeout(pressTimer);
}

function openMenu(source: RssSource, x: number, y: number) {
  menuSource.value = source;
  menuTop.value = Math.min(y, window.innerHeight - 240);
  menuRight.value = Math.max(8, window.innerWidth - x);
  menuVisible.value = true;
}

async function onMenu(key: string) {
  const source = menuSource.value;
  if (!source) return;
  if (key === "top") {
    const min = Math.min(...sources.value.map((item) => item.customOrder ?? 0), 0);
    await rssSourceSave({ ...source, customOrder: min - 1 });
    await load();
  } else if (key === "edit") {
    router.push({ path: "/rss-source-edit", query: { sourceUrl: source.sourceUrl } });
  } else if (key === "disable") {
    await rssSourceSave({ ...source, enabled: false });
    await load();
  } else if (key === "delete") {
    if (!confirm(`删除订阅源「${source.sourceName || source.sourceUrl}」？`)) return;
    await rssSourceDelete(source.sourceUrl);
    await load();
  }
}

function chooseGroup(key: string) {
  keyword.value = key;
}

let activatedOnce = false;
onMounted(async () => {
  await load();
  activatedOnce = true;
});
onActivated(() => {
  if (activatedOnce) void load();
});
</script>

<template>
  <div class="rss-page">
    <TitleBar :back="false" search-only>
      <template #content>
        <SearchBar v-model="keyword" placeholder="搜索" />
      </template>
      <template #actions>
        <button aria-label="收藏" @click="router.push('/rss-favorites')">
          <Icon name="ic_star" :size="24" />
        </button>
        <button aria-label="分组" @click="groupVisible = groupItems.length > 0">
          <Icon name="ic_groups" :size="24" />
        </button>
        <button aria-label="设置" @click="router.push('/rss-source-manage')">
          <Icon name="ic_settings" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu
      v-model="menuVisible"
      :items="menuItems"
      :top="menuTop"
      :right="menuRight"
      @select="onMenu"
    />
    <PopupMenu v-model="groupVisible" :items="groupItems" @select="chooseGroup" />

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else ref="rssGridRef" class="rss-grid">
      <button class="rss-item rule-sub" @click="router.push('/rule-sub')">
        <span class="rss-icon rule-icon">
          <img :src="legadoIcon" alt="" />
        </span>
        <span class="rss-name">规则订阅</span>
      </button>
      <EmptyState
        v-if="!filtered.length"
        class="empty-state"
        icon="ic_bottom_rss_e"
        text="RSS 源目前为空！"
        action="打开订阅源管理"
        @action="router.push('/rss-source-manage')"
      />
      <button
        v-for="source in filtered"
        :key="source.sourceUrl"
        class="rss-item"
        @click="openSource(source)"
        @contextmenu.prevent="openMenu(source, $event.clientX, $event.clientY)"
        @pointerdown="startPress(source, $event)"
        @pointerup="cancelPress"
        @pointerleave="cancelPress"
      >
        <span class="rss-icon">
          <img v-if="source.sourceIcon" :src="source.sourceIcon" alt="" />
          <Icon v-else name="ic_bottom_rss_feed" :size="28" />
        </span>
        <span class="rss-name">{{ source.sourceName || source.sourceUrl }}</span>
      </button>
    </main>
  </div>
</template>

<style scoped>
.rss-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.message,
.loading {
  margin: 8px 16px;
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.rss-grid {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  align-content: start;
  padding: 0 0 20px;
}
.empty-state {
  grid-column: 1 / -1;
}
.rss-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border: none;
  background: transparent;
  cursor: pointer;
  border-radius: 4px;
}
.rss-item:active {
  background: var(--ac-transparent-10);
}
.rss-icon {
  width: 50px;
  height: 50px;
  border-radius: 12px;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--ac-transparent-10);
  color: var(--ac-primary);
}
.rss-icon img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.rule-icon {
  background: transparent;
}
.rss-name {
  width: 100%;
  font-size: 13px;
  line-height: 1.25;
  color: var(--ac-secondary-text);
  text-align: center;
  overflow: hidden;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  word-break: break-all;
}
</style>
