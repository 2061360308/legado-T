<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import {
  bookSourceDelete,
  bookSourceExploreCategories,
  bookSourceList,
  bookSourceSave,
} from "@/core/application/bookSources";
import type { BookSource, ExploreCategory } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const sources = ref<BookSource[]>([]);
const keyword = ref("");
const loading = ref(false);
const message = ref("");
const expanded = ref("");
const categories = ref<Record<string, ExploreCategory[]>>({});
const categoryLoading = ref("");
const menuVisible = ref(false);
const groupVisible = ref(false);
const menuSource = ref<BookSource | null>(null);
const menuTop = ref(52);
const menuRight = ref(8);
let pressTimer: number | undefined;
let longPressed = false;

const exploreSources = computed(() =>
  sources.value.filter(
    (source) =>
      source.enabledExplore !== false &&
      Boolean(String(source.exploreUrl || "").trim()),
  ),
);

const groups = computed(() => {
  const values = new Set<string>();
  for (const source of exploreSources.value) {
    for (const group of String(source.bookSourceGroup || "").split(/[,，;；]/)) {
      const name = group.trim();
      if (name) values.add(name);
    }
  }
  return [...values].sort((a, b) => a.localeCompare(b, "zh-CN"));
});

const filtered = computed(() => {
  const key = keyword.value.trim();
  if (!key) return exploreSources.value;
  if (key.startsWith("group:")) {
    const group = key.slice(6).trim();
    return exploreSources.value.filter((source) =>
      String(source.bookSourceGroup || "")
        .split(/[,，;；]/)
        .map((value) => value.trim())
        .includes(group),
    );
  }
  const lower = key.toLowerCase();
  return exploreSources.value.filter((source) => {
    const name = String(source.bookSourceName || "").toLowerCase();
    const group = String(source.bookSourceGroup || "").toLowerCase();
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

const menuItems = computed<PopupMenuItem[]>(() => {
  const source = menuSource.value;
  return [
    { key: "edit", label: "编辑" },
    { key: "top", label: "置顶" },
    { key: "search", label: "搜索" },
    ...(String(source?.loginUrl || "").trim()
      ? [{ key: "login", label: "登录" }]
      : []),
    { key: "refresh", label: "刷新" },
    { key: "delete", label: "删除", danger: true },
  ];
});

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2500);
}

async function load() {
  loading.value = true;
  try {
    sources.value = await bookSourceList();
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

async function loadCategories(source: BookSource, force = false) {
  const key = source.bookSourceUrl;
  if (!force && categories.value[key]) return;
  categoryLoading.value = key;
  try {
    categories.value = {
      ...categories.value,
      [key]: await bookSourceExploreCategories(key),
    };
  } catch (error) {
    categories.value = { ...categories.value, [key]: [] };
    showMessage(String(error));
  } finally {
    if (categoryLoading.value === key) categoryLoading.value = "";
  }
}

async function toggleSource(source: BookSource) {
  if (longPressed) {
    longPressed = false;
    return;
  }
  const key = source.bookSourceUrl;
  if (expanded.value === key) {
    expanded.value = "";
    return;
  }
  expanded.value = key;
  await loadCategories(source);
}

function startPress(source: BookSource, event: PointerEvent | MouseEvent) {
  window.clearTimeout(pressTimer);
  longPressed = false;
  const x = event.clientX || window.innerWidth - 24;
  const y = event.clientY || 56;
  pressTimer = window.setTimeout(() => {
    longPressed = true;
    openSourceMenu(source, x, y);
  }, 480);
}

function cancelPress() {
  window.clearTimeout(pressTimer);
}

function openSourceMenu(source: BookSource, x: number, y: number) {
  menuSource.value = source;
  menuTop.value = Math.min(y, window.innerHeight - 260);
  menuRight.value = Math.max(8, window.innerWidth - x);
  menuVisible.value = true;
}

function openCategory(source: BookSource, category: ExploreCategory) {
  if (category.title.startsWith("ERROR:")) {
    showMessage(String(category.url || "分类加载失败"));
    return;
  }
  if (!String(category.url || "").trim()) return;
  router.push({
    path: "/explore-show",
    query: {
      sourceUrl: source.bookSourceUrl,
      sourceName: source.bookSourceName,
      exploreUrl: category.url,
      categoryTitle: category.title,
    },
  });
}

async function onMenu(key: string) {
  const source = menuSource.value;
  if (!source) return;
  if (key === "edit") {
    router.push({ path: "/book-source-edit", query: { url: source.bookSourceUrl } });
  } else if (key === "top") {
    const min = Math.min(...sources.value.map((item) => item.customOrder ?? 0), 0);
    await bookSourceSave({ ...source, customOrder: min - 1 });
    await load();
  } else if (key === "search") {
    router.push({ path: "/book-search", query: { sourceUrl: source.bookSourceUrl } });
  } else if (key === "login") {
    router.push({ path: "/source-login", query: { url: source.bookSourceUrl } });
  } else if (key === "refresh") {
    categories.value = { ...categories.value, [source.bookSourceUrl]: [] };
    await loadCategories(source, true);
  } else if (key === "delete") {
    if (!confirm(`删除书源「${source.bookSourceName || source.bookSourceUrl}」？`)) return;
    await bookSourceDelete(source.bookSourceUrl);
    if (expanded.value === source.bookSourceUrl) expanded.value = "";
    await load();
  }
}

function chooseGroup(key: string) {
  keyword.value = key;
}

onMounted(load);
</script>

<template>
  <div class="explore-page">
    <TitleBar title="发现" :back="false">
      <template #content>
        <label class="search">
          <input v-model="keyword" type="search" placeholder="搜索书源" />
        </label>
      </template>
      <template #actions>
        <button aria-label="分组" @click="groupVisible = groupItems.length > 0">
          <Icon name="ic_groups" :size="24" />
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
    <main v-else class="explore-list">
      <EmptyState
        v-if="!filtered.length"
        icon="ic_bottom_explore_e"
        text="当前没有发现源"
        action="去书源管理添加"
        @action="router.push('/book-source')"
      />
      <section
        v-for="source in filtered"
        :key="source.bookSourceUrl"
        class="source-block"
      >
        <div
          class="source-title"
          @click="toggleSource(source)"
          @contextmenu.prevent="openSourceMenu(source, $event.clientX, $event.clientY)"
          @pointerdown="startPress(source, $event)"
          @pointerup="cancelPress"
          @pointerleave="cancelPress"
        >
          <span class="source-name">{{ source.bookSourceName || source.bookSourceUrl }}</span>
          <span v-if="categoryLoading === source.bookSourceUrl" class="spinner" />
          <Icon
            v-else
            :name="expanded === source.bookSourceUrl ? 'ic_arrow_down' : 'ic_arrow_right'"
            :size="20"
            class="arrow"
          />
        </div>
        <div v-if="expanded === source.bookSourceUrl" class="categories">
          <button
            v-for="category in categories[source.bookSourceUrl] || []"
            :key="`${category.title}-${category.url}`"
            class="category"
            :class="{ disabled: !String(category.url || '').trim() }"
            @click.stop="openCategory(source, category)"
          >
            {{ category.title }}
          </button>
          <span
            v-if="!(categories[source.bookSourceUrl] || []).length && categoryLoading !== source.bookSourceUrl"
            class="empty-category"
          >
            暂无发现分类
          </span>
        </div>
      </section>
    </main>
  </div>
</template>

<style scoped>
.explore-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.search {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
}
.search input {
  width: 100%;
  height: 30px;
  border: none;
  border-radius: 17px;
  background: rgba(255, 255, 255, 0.22);
  color: #fff;
  font-size: 13px;
  padding: 0 16px;
  outline: none;
}
.search input::placeholder {
  color: rgba(255, 255, 255, 0.65);
}
.message,
.loading {
  margin: 8px 16px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.explore-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.source-block {
  padding: 0 10px;
}
.source-title {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 46px;
  padding: 6px 10px;
  background: var(--ac-background-menu);
  border-bottom: 1px solid var(--ac-divider);
  cursor: pointer;
}
.source-title:active {
  background: var(--ac-transparent-10);
}
.source-name {
  flex: 1;
  min-width: 0;
  color: var(--ac-primary-text);
  font-size: 15px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.arrow {
  color: var(--ac-secondary-text);
}
.spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--ac-divider);
  border-top-color: var(--ac-primary);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}
@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
.categories {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 10px 8px 14px;
}
.category {
  border: none;
  background: var(--ac-transparent-10);
  color: var(--ac-primary-text);
  border-radius: 16px;
  padding: 6px 14px;
  font-size: 13px;
  cursor: pointer;
}
.category:active {
  background: var(--ac-primary);
  color: #fff;
}
.category.disabled {
  opacity: 0.4;
  pointer-events: none;
}
.empty-category {
  color: var(--ac-secondary-text);
  font-size: 12px;
  padding: 4px;
}
</style>
