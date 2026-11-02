<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import {
  rssStarClear,
  rssStarDelete,
  rssStarDeleteGroup,
  rssStarGroups,
  rssStarList,
} from "@/core/application/rss";
import type { RssStar } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const groups = ref<string[]>([]);
const activeGroup = ref("");
const stars = ref<RssStar[]>([]);
const loading = ref(false);
const message = ref("");
const menuVisible = ref(false);
let pressTimer: number | undefined;
let longPressed = false;

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "deleteGroup", label: "删除当前分组" },
  { key: "deleteAll", label: "删除全部收藏", danger: true },
]);

async function loadGroups() {
  groups.value = await rssStarGroups();
  if (!groups.value.includes(activeGroup.value)) {
    activeGroup.value = groups.value[0] ?? "";
  }
}

async function loadStars() {
  loading.value = true;
  try {
    stars.value = activeGroup.value
      ? await rssStarList(activeGroup.value)
      : await rssStarList();
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

async function load() {
  try {
    await loadGroups();
    await loadStars();
  } catch (error) {
    message.value = String(error);
  }
}

function selectGroup(group: string) {
  if (activeGroup.value === group) return;
  activeGroup.value = group;
  void loadStars();
}

function openStar(star: RssStar) {
  if (longPressed) {
    longPressed = false;
    return;
  }
  router.push({
    path: "/rss-read",
    query: {
      sourceUrl: star.origin,
      title: star.title,
      link: star.link,
      origin: star.origin,
      sort: star.sort,
    },
  });
}

function startPress(star: RssStar) {
  window.clearTimeout(pressTimer);
  longPressed = false;
  pressTimer = window.setTimeout(() => {
    longPressed = true;
    void remove(star);
  }, 480);
}

function cancelPress() {
  window.clearTimeout(pressTimer);
}

async function remove(star: RssStar) {
  if (!confirm(`删除收藏「${star.title}」？`)) return;
  await rssStarDelete(star.origin, star.link);
  await load();
}

async function onMenu(key: string) {
  if (key === "deleteGroup") {
    if (!activeGroup.value) return;
    if (!confirm(`删除分组「${activeGroup.value}」的全部收藏？`)) return;
    await rssStarDeleteGroup(activeGroup.value);
    await load();
  } else if (key === "deleteAll") {
    if (!confirm("删除全部收藏？")) return;
    await rssStarClear();
    await load();
  }
}

onMounted(load);
</script>

<template>
  <div class="favorites-page">
    <TitleBar title="收藏" @back="router.back()">
      <template #actions>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />
    <div v-if="groups.length > 1" class="tabs">
      <button
        v-for="group in groups"
        :key="group"
        :class="{ active: activeGroup === group }"
        @click="selectGroup(group)"
      >
        {{ group }}
      </button>
    </div>

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else ref="listRef" class="list">
      <EmptyState
        v-if="!stars.length"
        icon="ic_star_border"
        text="暂无收藏"
      />
      <article
        v-for="star in stars"
        :key="`${star.origin}-${star.link}`"
        class="article"
        @click="openStar(star)"
        @contextmenu.prevent="remove(star)"
        @pointerdown="startPress(star)"
        @pointerup="cancelPress"
        @pointerleave="cancelPress"
      >
        <div v-if="star.image" class="image-wrap">
          <img :src="star.image" alt="" loading="lazy" />
        </div>
        <div class="text">
          <p class="title">{{ star.title || star.link }}</p>
          <p v-if="star.pubDate" class="date">{{ star.pubDate }}</p>
        </div>
      </article>
    </main>
  </div>
</template>

<style scoped>
.favorites-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.tabs {
  display: flex;
  overflow-x: auto;
  border-bottom: 1px solid var(--ac-divider);
}
.tabs button {
  flex: none;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 11px 14px;
  font-size: 13px;
  border-bottom: 2px solid transparent;
  white-space: nowrap;
}
.tabs button.active {
  color: var(--ac-primary);
  border-bottom-color: var(--ac-primary);
}
.message,
.loading {
  margin: 8px 16px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.article {
  display: flex;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ac-divider);
  cursor: pointer;
}
.article:active {
  background: var(--ac-transparent-10);
}
.image-wrap {
  flex: none;
  order: 2;
}
.image-wrap img {
  width: 110px;
  height: 68px;
  object-fit: cover;
  border-radius: 2px;
}
.text {
  flex: 1;
  min-width: 0;
}
.title {
  margin: 0;
  color: var(--ac-primary-text);
  font-size: 16px;
  font-weight: 600;
  line-height: 1.35;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.date {
  margin: 8px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
  font-style: italic;
}
</style>
