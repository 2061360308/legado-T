<script setup lang="ts">
import type { Book } from "@/core/domain";

defineProps<{
  books: Book[];
  history: string[];
}>();

const emit = defineEmits<{
  openBook: [book: Book];
  selectHistory: [keyword: string];
  deleteHistory: [keyword: string];
  clearHistory: [];
}>();

let pressTimer: number | undefined;
let longPressed = false;

function startPress(keyword: string) {
  window.clearTimeout(pressTimer);
  longPressed = false;
  pressTimer = window.setTimeout(() => {
    longPressed = true;
    emit("deleteHistory", keyword);
  }, 600);
}

function cancelPress() {
  window.clearTimeout(pressTimer);
}

function chooseHistory(keyword: string) {
  if (longPressed) {
    longPressed = false;
    return;
  }
  emit("selectHistory", keyword);
}
</script>

<template>
  <section class="input-help">
    <template v-if="books.length">
      <p class="header">书架</p>
      <div class="chips">
        <button
          v-for="book in books"
          :key="book.bookUrl"
          class="chip"
          @click="emit('openBook', book)"
        >
          {{ book.name }}
        </button>
      </div>
    </template>

    <div class="header-row">
      <span class="header">搜索历史</span>
      <button v-if="history.length" class="clear" @click="emit('clearHistory')">清空</button>
    </div>
    <div v-if="history.length" class="chips">
      <button
        v-for="item in history"
        :key="item"
        class="chip"
        @click="chooseHistory(item)"
        @contextmenu.prevent="emit('deleteHistory', item)"
        @pointerdown="startPress(item)"
        @pointerup="cancelPress"
        @pointerleave="cancelPress"
        @pointercancel="cancelPress"
      >
        {{ item }}
      </button>
    </div>
    <p v-else class="empty">暂无搜索历史</p>
  </section>
</template>

<style scoped>
.input-help {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 6px 10px 24px;
  background: var(--ac-background);
}
.header-row {
  display: flex;
  align-items: center;
  gap: 8px;
}
.header {
  flex: 1;
  margin: 0;
  padding: 6px;
  color: var(--ac-primary-text);
  font-size: 13px;
}
.clear {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  padding: 6px;
  font-size: 13px;
  cursor: pointer;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  padding: 2px 0 8px;
}
.chip {
  max-width: 100%;
  margin: 3px;
  padding: 4px 12px;
  border: none;
  border-radius: 16px;
  background: rgba(172, 172, 172, 0.39);
  color: var(--ac-primary-text);
  font-size: 14px;
  line-height: 1.4;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  cursor: pointer;
}
.chip:active {
  background: rgba(133, 133, 133, 0.39);
}
.empty {
  margin: 12px 6px;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
</style>
