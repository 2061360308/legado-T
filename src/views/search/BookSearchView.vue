<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();
const keyword = ref("");
const menuVisible = ref(false);
const message = ref("");
const history = ref<string[]>([]);

const menuItems: PopupMenuItem[] = [
  { key: "precision", label: "精确搜索" },
  { key: "source", label: "书源管理", icon: "ic_cfg_source" },
  { key: "scope", label: "分组或书源" },
  { key: "log", label: "日志", icon: "ic_cfg_about" },
];

function onMenu(key: string) {
  if (key === "source") router.push("/book-source");
  else if (key === "precision") message.value = "精确搜索暂未接入";
  else if (key === "scope") message.value = "搜索范围暂未接入";
  else if (key === "log") message.value = "日志暂未接入";
}

function search() {
  if (!keyword.value.trim()) return;
  if (!history.value.includes(keyword.value.trim())) {
    history.value.unshift(keyword.value.trim());
  }
  message.value = "在线书籍搜索暂未接入";
}

function clearHistory() {
  history.value = [];
}
</script>

<template>
  <div class="book-search">
    <TitleBar title="搜索" @back="router.back()">
      <template #content>
        <input
          v-model="keyword"
          class="search-input"
          placeholder="搜索书名或作者"
          @keydown.enter="search"
        />
      </template>
      <template #actions>
        <button aria-label="搜索" @click="search"><Icon name="ic_search" :size="24" /></button>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <div class="progress" />
    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />

    <main class="content">
      <p v-if="message" class="message">{{ message }}</p>
      <section class="shelf-block">
        <p class="section-title">书架</p>
        <EmptyState icon="ic_bottom_books_e" text="书架内搜索暂未接入" />
      </section>

      <section class="history-block">
        <div class="history-header">
          <p class="section-title">搜索历史</p>
          <button v-if="history.length" @click="clearHistory">清空</button>
        </div>
        <button v-for="item in history" :key="item" class="history-item" @click="keyword = item; search()">
          {{ item }}
        </button>
        <p v-if="history.length === 0" class="empty">暂无搜索历史</p>
      </section>

      <button class="stop-fab" @click="message = '搜索任务暂未接入'">
        <Icon name="ic_stop_black_24dp" :size="20" />
      </button>
    </main>
  </div>
</template>

<style scoped>
.book-search {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.search-input {
  flex: 1;
  min-width: 0;
  margin: 0 8px;
  padding: 6px 10px;
  border: none;
  border-radius: 2px;
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
  outline: none;
}
.search-input::placeholder {
  color: rgba(255, 255, 255, 0.6);
}
.progress {
  height: 2px;
  background: transparent;
}
.content {
  flex: 1;
  overflow-y: auto;
  position: relative;
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.section-title {
  margin: 0;
  padding: 6px;
  font-size: 14px;
  color: var(--ac-primary-text);
}
.shelf-block,
.history-block {
  padding: 8px;
}
.history-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.history-header button {
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  font-size: 13px;
  padding: 6px;
}
.history-item {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
  padding: 10px 8px;
  font-size: 14px;
  border-bottom: 1px solid var(--ac-divider);
  cursor: pointer;
}
.empty {
  margin: 8px;
  font-size: 13px;
  color: var(--ac-secondary-text);
}
.stop-fab {
  position: absolute;
  right: 16px;
  bottom: 16px;
  width: 42px;
  height: 42px;
  border: none;
  border-radius: 50%;
  background: var(--ac-background-menu);
  color: var(--ac-secondary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 2px 6px rgba(0, 0, 0, 0.28);
  cursor: pointer;
}
</style>
