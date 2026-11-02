<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import { bookSourceExport } from "@/core/application/bookSources";
import {
  bookAddGroup,
  bookBatchAddGroup,
  bookBatchSetCanUpdate,
  bookDelete,
  bookList,
  bookSetGroup,
  groupList,
} from "@/core/application/books";
import type { Book, BookGroup } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import GroupSelectDialog from "./GroupSelectDialog.vue";

const router = useRouter();
const books = ref<Book[]>([]);
const groups = ref<BookGroup[]>([]);
const selected = ref<Set<string>>(new Set());
const keyword = ref("");
const message = ref("");
const groupSelectVisible = ref(false);
const groupMode = ref<"move" | "add">("move");
const singleBook = ref<Book | null>(null);
const menuVisible = ref(false);
const moreMenuVisible = ref(false);
const openDetailOnTitle = ref(localStorage.getItem("arrange:open-detail-on-title") === "1");

const menuItems = computed<PopupMenuItem[]>(() => [
  { key: "groups", label: "分组管理" },
  { key: "exportSources", label: "导出所有使用书源" },
  {
    key: "openDetail",
    label: "点击书名打开详情",
    checked: openDetailOnTitle.value,
  },
]);

const moreItems: PopupMenuItem[] = [
  { key: "move", label: "移入分组" },
  { key: "add", label: "加入分组" },
  { key: "enable", label: "允许更新" },
  { key: "disable", label: "禁止更新" },
  { key: "delete", label: "删除", danger: true },
];

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

async function changeGroup(groupId: number) {
  if (singleBook.value) {
    if (groupMode.value === "add") await bookAddGroup(singleBook.value.bookUrl, groupId);
    else await bookSetGroup(singleBook.value.bookUrl, groupId);
    message.value = `已更新《${singleBook.value.name}》的分组`;
    singleBook.value = null;
  } else {
    const selectedBooks = books.value.filter((book) => selected.value.has(book.bookUrl));
    if (groupMode.value === "add") {
      await bookBatchAddGroup(
        selectedBooks.map((book) => book.bookUrl),
        groupId,
      );
    } else {
      for (const book of selectedBooks) await bookSetGroup(book.bookUrl, groupId);
    }
    message.value = `已${groupMode.value === "add" ? "追加" : "移动"} ${selectedBooks.length} 本书到分组`;
    cancel();
  }
  groupSelectVisible.value = false;
  await load();
}

function selectAll() {
  selected.value = new Set(filtered.value.map((book) => book.bookUrl));
}

function invertSelection() {
  selected.value = new Set(
    filtered.value
      .filter((book) => !selected.value.has(book.bookUrl))
      .map((book) => book.bookUrl),
  );
}

async function setCanUpdate(value: boolean) {
  if (!selected.value.size) return;
  const count = await bookBatchSetCanUpdate([...selected.value], value);
  message.value = `已${value ? "启用" : "禁用"} ${count} 本书的更新`;
}

async function exportSources() {
  try {
    const json = await bookSourceExport();
    const path = await platform.dialog.save({
      defaultPath: "book-sources.json",
      filters: [{ name: "书源 JSON", extensions: ["json"] }],
    });
    if (!path) return;
    await platform.files.writeFile(path, new TextEncoder().encode(json));
    message.value = "书源已导出";
  } catch (error) {
    message.value = String(error);
  }
}

function onMenu(key: string) {
  if (key === "groups") router.push("/group-manage");
  else if (key === "exportSources") void exportSources();
  else if (key === "openDetail") {
    openDetailOnTitle.value = !openDetailOnTitle.value;
    localStorage.setItem("arrange:open-detail-on-title", openDetailOnTitle.value ? "1" : "0");
  }
}

function onMore(key: string) {
  if (key === "move") {
    groupMode.value = "move";
    groupSelectVisible.value = true;
  } else if (key === "add") {
    groupMode.value = "add";
    groupSelectVisible.value = true;
  } else if (key === "enable") {
    void setCanUpdate(true);
  } else if (key === "disable") {
    void setCanUpdate(false);
  } else if (key === "delete") {
    void removeSelected();
  }
}

async function openSingleGroup(book: Book) {
  singleBook.value = book;
  groupMode.value = "move";
  groupSelectVisible.value = true;
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
  void openSingleGroup(book);
}

function openInfo(book: Book) {
  router.push({ path: "/book-info", query: { bookUrl: book.bookUrl } });
}

onMounted(load);
</script>

<template>
  <div class="arrange-page">
    <TitleBar title="书架管理" @back="router.back()">
      <template #content>
        <input v-model="keyword" class="search" placeholder="搜索书名/作者" />
      </template>
      <template #actions>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>
    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />
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
            <span class="name" @click.stop="openDetailOnTitle && openInfo(book)">{{ book.name }}</span>
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
      <p v-if="filtered.length === 0" class="empty">暂无书籍</p>
    </main>
    <div class="select-bar" data-id="select_action_bar">
      <label class="select-all" @click.prevent="selected.size === filtered.length ? cancel() : selectAll()">
        <input type="checkbox" :checked="selected.size > 0 && selected.size === filtered.length" readonly />
        <span>
          {{ selected.size > 0 && selected.size === filtered.length ? "取消全选" : "全选" }}（{{
            selected.size
          }}/{{ filtered.length }}）
        </span>
      </label>
      <button class="bar-button" :disabled="!selected.size" @click="invertSelection">反选</button>
      <button
        class="bar-button accent"
        :disabled="!selected.size"
        @click="groupMode = 'move'; groupSelectVisible = true"
      >
        移入分组
      </button>
      <button
        v-if="selected.size"
        class="bar-icon"
        aria-label="更多"
        @click="moreMenuVisible = true"
      >
        <Icon name="ic_more_vert" :size="22" />
      </button>
    </div>
    <PopupMenu v-model="moreMenuVisible" :items="moreItems" :top="52" :right="10" @select="onMore" />

    <GroupSelectDialog
      v-model="groupSelectVisible"
      :group-id="0"
      @confirm="changeGroup"
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
.select-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 5px;
  padding: 6px 8px 6px 16px;
  background: var(--ac-background-menu);
  box-shadow: 0 -2px 2px rgba(0, 0, 0, 0.18);
}
.select-all {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
.select-all input {
  accent-color: var(--ac-primary);
}
.bar-button,
.bar-icon {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  min-width: 72px;
  padding: 6px 10px;
  font-size: 13px;
}
.bar-button.accent {
  background: var(--ac-primary);
  color: #fff;
}
.bar-icon {
  min-width: 36px;
  width: 36px;
  height: 36px;
  padding: 0;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-color: transparent;
  color: var(--ac-primary-text);
}
.bar-button:disabled,
.bar-icon:disabled {
  opacity: 0.35;
}
.empty {
  margin: 60px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
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
