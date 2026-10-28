<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const keyword = ref("");
const hasSources = false;

interface RssSource {
  sourceUrl: string;
  sourceName: string;
}

const sources: RssSource[] = [];
const activeMenu = ref<"favorites" | "groups" | "">("");
const menuVisible = computed({
  get: () => activeMenu.value !== "",
  set: (value: boolean) => {
    if (!value) activeMenu.value = "";
  },
});
const menuItems = computed<PopupMenuItem[]>(() =>
  activeMenu.value === "favorites"
    ? [{ key: "empty", label: "暂无收藏", muted: true }]
    : [{ key: "empty", label: "暂无分组", muted: true }],
);
const message = ref("");
function onMenuSelect() {
  activeMenu.value = "";
}
</script>

<template>
  <div class="rss-page">
    <TitleBar title="订阅" :back="false">
      <template #content>
        <label class="search">
          <input v-model="keyword" type="search" placeholder="搜索" />
        </label>
      </template>
      <template #actions>
        <button aria-label="收藏" @click="activeMenu = 'favorites'">
          <Icon name="ic_star" :size="24" />
        </button>
        <button aria-label="分组" @click="activeMenu = 'groups'">
          <Icon name="ic_groups" :size="24" />
        </button>
        <button aria-label="设置" @click="message = 'RSS 设置暂未实现'">
          <Icon name="ic_settings" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenuSelect" />
    <p v-if="message" class="message">{{ message }}</p>

    <main class="rss-grid">
      <template v-if="hasSources">
        <button v-for="source in sources" :key="source.sourceUrl" class="rss-item">
          <span class="rss-name">{{ source.sourceName }}</span>
        </button>
      </template>
      <EmptyState
        v-else
        icon="ic_bottom_rss_e"
        text="RSS 源目前为空！"
        action="去书源管理添加"
        @action="router.push('/book-source')"
      />
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
.rss-grid {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  align-content: start;
}
.rss-item {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 18px 8px;
  border: none;
  background: transparent;
  cursor: pointer;
}
.rss-name {
  font-size: 13px;
  color: var(--ac-primary-text);
  text-align: center;
}
</style>
