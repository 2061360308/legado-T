<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { bookDelete, bookList, groupList } from "@/api/books";
import type { Book, BookGroup } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import SelectActionBar from "@/components/android/SelectActionBar.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();
const books = ref<Book[]>([]);
const groups = ref<BookGroup[]>([]);
const selected = ref<Set<string>>(new Set());
const keyword = ref("");
const message = ref("");

const filtered = computed(() => {
  const key = keyword.value.trim().toLowerCase();
  if (!key) return books.value;
  return books.value.filter(
    (book) =>
      book.name.toLowerCase().includes(key) || (book.author || "").toLowerCase().includes(key),
  );
});

function groupNames(book: Book): string {
  const names = groups.value
    .filter((group) => group.groupId > 0 && ((book.group ?? 0) & group.groupId) !== 0)
    .map((group) => group.groupName);
  return names.length ? names.join("，") : "未分组";
}

function toggle(book: Book, value: boolean) {
  const next = new Set(selected.value);
  if (value) next.add(book.bookUrl);
  else next.delete(book.bookUrl);
  selected.value = next;
}

function cancel() {
  selected.value = new Set();
}

async function remove(book: Book) {
  if (!confirm(`删除《${book.name}》及其章节数据？`)) return;
  await bookDelete(book.bookUrl);
  selected.value.delete(book.bookUrl);
  await load();
}

async function removeSelected() {
  if (selected.value.size === 0) return;
  if (!confirm(`删除选中的 ${selected.value.size} 本书及其章节数据？`)) return;
  for (const url of selected.value) await bookDelete(url);
  cancel();
  await load();
}

async function load() {
  books.value = await bookList();
  try {
    groups.value = await groupList();
  } catch {
    groups.value = [];
  }
}

function openGroup(book: Book) {
  message.value = `《${book.name}》分组：${groupNames(book)}（分组选择后续接入）`;
}

onMounted(load);
</script>

<template>
  <div class="arrange-page">
    <TitleBar title="书架管理" @back="router.back()">
      <template #content>
        <input v-model="keyword" class="search" placeholder="搜索书名/作者" />
      </template>
    </TitleBar>
    <p v-if="message" class="message">{{ message }}</p>
    <main class="list">
      <div v-for="book in filtered" :key="book.bookUrl" class="book-row">
        <label class="check">
          <input
            type="checkbox"
            :checked="selected.has(book.bookUrl)"
            @change="toggle(book, ($event.target as HTMLInputElement).checked)"
          />
        </label>
        <div class="text">
          <p class="name-line">
            <span class="name">{{ book.name }}</span>
            <span class="author">{{ book.author }}</span>
          </p>
          <p class="meta-line">
            <span class="origin">{{ book.originName || book.origin }}</span>
            <span class="groups">{{ groupNames(book) }}</span>
            <button class="mini" @click="openGroup(book)">分组</button>
            <button class="mini" @click="remove(book)">删除</button>
          </p>
        </div>
      </div>
      <EmptyState v-if="filtered.length === 0" text="暂无书籍" />
    </main>
    <SelectActionBar
      :count="selected.size"
      @remove="removeSelected"
      @cancel="cancel"
      @enable="message = '批量启用暂未实现'"
      @disable="message = '批量禁用暂未实现'"
    />
  </div>
</template>

<style scoped>
.arrange-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.search {
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
.search::placeholder {
  color: rgba(255, 255, 255, 0.6);
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  overflow-y: auto;
}
.book-row {
  display: flex;
  gap: 10px;
  padding: 6px;
  border-bottom: 1px solid var(--ac-divider);
}
.check {
  padding: 10px;
  display: flex;
  align-items: center;
}
.check input {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.name-line {
  margin: 0;
  display: flex;
  align-items: baseline;
  gap: 8px;
}
.name {
  font-size: 15px;
  color: var(--ac-primary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.author {
  font-size: 12px;
  color: var(--ac-summary-text);
  flex: none;
}
.meta-line {
  margin: 0;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.origin,
.groups {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.groups {
  flex: 1;
}
.mini {
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  padding: 6px 8px;
  font-size: 12px;
  cursor: pointer;
}
.mini:hover {
  color: var(--ac-primary);
}
</style>
