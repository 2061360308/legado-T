<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useBooksStore } from "@/stores/books";
import { useConfigStore } from "@/stores/config";
import {
  bookshelfExport,
  bookshelfList,
  groupList,
  VIRTUAL_GROUP,
} from "@/api/books";
import type { Book, BookGroup } from "@/api/bindings";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";
import BookItem from "./BookItem.vue";
import BookshelfConfigDialog from "./BookshelfConfigDialog.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import EmptyState from "@/components/android/EmptyState.vue";

const router = useRouter();

const manageItems: PopupMenuItem[] = [
  { key: "updateToc", label: "更新目录", icon: "ic_refresh_black_24dp" },
  { key: "import", label: "添加本地", icon: "ic_add" },
  { key: "remote", label: "远程书籍", icon: "ic_add" },
  { key: "addUrl", label: "添加网址", icon: "ic_add_online" },
  { key: "arrange", label: "书架管理", icon: "ic_arrange" },
  { key: "cache", label: "缓存/导出", icon: "ic_download_line" },
  { key: "group", label: "分组管理", icon: "ic_groups" },
  { key: "layout", label: "书架布局", icon: "ic_view_quilt" },
  { key: "export", label: "导出书单", icon: "ic_export" },
  { key: "importShelf", label: "导入书单", icon: "ic_import" },
  { key: "log", label: "日志", icon: "ic_cfg_about", muted: true },
];
const booksStore = useBooksStore();
const configStore = useConfigStore();

const bookshelfConfigVisible = ref(false);
const activeGroup = ref<number>(VIRTUAL_GROUP.all);
const manageMenuVisible = ref(false);
const userGroups = ref<BookGroup[]>([]);
const message = ref("");

const folderStyle = computed(() => configStore.global.bookGroupStyle === 1);
const layout = computed<"list" | "grid">(() =>
  configStore.global.bookshelfLayout === 0 ? "list" : "grid",
);

const groups = computed(() => [
  { id: VIRTUAL_GROUP.all, name: "全部" },
  ...userGroups.value.map((group) => ({ id: group.groupId, name: group.groupName })),
]);

const activeGroupName = computed(
  () => groups.value.find((group) => group.id === activeGroup.value)?.name ?? "全部",
);

type FolderItem =
  | { type: "group"; group: BookGroup }
  | { type: "book"; book: Book };

function hasGroupBit(value: number, bit: number): boolean {
  if (!bit) return false;
  return Math.floor(value / bit) % 2 === 1;
}

function inAnyGroup(book: Book): boolean {
  const value = book.group ?? 0;
  return userGroups.value.some((group) => hasGroupBit(value, group.groupId));
}

const folderItems = computed<FolderItem[]>(() => {
  if (!folderStyle.value || activeGroup.value !== VIRTUAL_GROUP.all) {
    return booksStore.list.map((book) => ({ type: "book" as const, book }));
  }
  return [
    ...userGroups.value.map((group) => ({ type: "group" as const, group })),
    ...booksStore.list
      .filter((book) => !inAnyGroup(book))
      .map((book) => ({ type: "book" as const, book })),
  ];
});

const gridColumns = computed(() =>
  configStore.global.bookshelfLayout === 0
    ? 0
    : configStore.global.bookshelfLayout + 2,
);

async function load() {
  booksStore.setList(
    await bookshelfList(activeGroup.value, configStore.global.bookshelfSort),
  );
}

async function loadGroups() {
  try {
    userGroups.value = (await groupList()).filter((group) => group.groupId > 0);
  } catch {
    userGroups.value = [];
  }
}

function rememberTab() {
  localStorage.setItem("bookshelf:lastGroup", String(activeGroup.value));
}

function selectGroup(id: number) {
  activeGroup.value = id;
  rememberTab();
  load();
}

function openBook(book: Book) {
  router.push({ path: "/reader", query: { bookUrl: book.bookUrl } });
}

function openInfo(book: Book) {
  router.push({ path: "/book-info", query: { bookUrl: book.bookUrl } });
}

function openGroup(group: BookGroup) {
  activeGroup.value = group.groupId;
  rememberTab();
  load();
}

function backToRoot() {
  activeGroup.value = VIRTUAL_GROUP.all;
  rememberTab();
  load();
}

async function onExport() {
  const json = await bookshelfExport();
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = "bookshelf.json";
  a.click();
  URL.revokeObjectURL(url);
  manageMenuVisible.value = false;
  message.value = "书架已导出";
}

function openBookshelfConfig() {
  manageMenuVisible.value = false;
  bookshelfConfigVisible.value = true;
}

function onManage(key: string) {
  if (key === "updateToc") router.push("/legacy/activity_chapter_list");
  else if (key === "import") router.push("/import");
  else if (key === "remote") router.push("/book-search");
  else if (key === "addUrl") router.push("/book-source-edit");
  else if (key === "arrange") router.push("/arrange-books");
  else if (key === "cache") router.push("/cache-books");
  else if (key === "group") router.push("/legacy/fragment_my_config");
  else if (key === "layout") openBookshelfConfig();
  else if (key === "export") onExport();
  else if (key === "importShelf") router.push("/legacy/activity_import_book");
  else if (key === "log") router.push("/legacy/activity_config");
}

async function onConfigSaved() {
  await load();
}

// 样式2 下拉刷新（fragment_bookshelf2 SwipeRefreshLayout 语义）
const refreshing = ref(false);
let pullStartY = 0;
let pulling = false;

function onPullStart(e: PointerEvent) {
  if (!folderStyle.value) return;
  const el = e.currentTarget as HTMLElement;
  if (el.scrollTop <= 0) {
    pulling = true;
    pullStartY = e.clientY;
  }
}

async function onPullMove(e: PointerEvent) {
  if (!pulling || refreshing.value) return;
  const el = e.currentTarget as HTMLElement;
  if (el.scrollTop > 0) {
    pulling = false;
    return;
  }
  if (e.clientY - pullStartY > 72) {
    pulling = false;
    refreshing.value = true;
    await load();
    window.setTimeout(() => (refreshing.value = false), 600);
  }
}

function onPullEnd() {
  pulling = false;
}

onMounted(async () => {
  const saved = localStorage.getItem("bookshelf:lastGroup");
  if (saved) activeGroup.value = Number(saved);
  await configStore.load();
  await loadGroups();
  await load();
});
</script>

<template>
  <div class="bookshelf-page">
    <TitleBar
      data-id="title_bar"
      :title="folderStyle && activeGroup !== VIRTUAL_GROUP.all ? `书架（${activeGroupName}）` : '书架'"
      :back="folderStyle && activeGroup !== VIRTUAL_GROUP.all"
      @back="backToRoot"
    >
      <template #content>
        <div v-if="!folderStyle" class="tabs" data-id="view_pager_bookshelf">
          <button
            v-for="group in groups"
            :key="group.id"
            :class="{ active: activeGroup === group.id }"
            @click="selectGroup(group.id)"
          >
            {{ group.name }}
          </button>
        </div>
      </template>
      <template #actions>
        <button aria-label="搜索" @click="router.push('/book-search')">
          <Icon name="ic_search" :size="24" />
        </button>
        <button aria-label="管理" @click="manageMenuVisible = !manageMenuVisible">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <PopupMenu
      v-model="manageMenuVisible"
      :items="manageItems"
      @select="onManage"
    />

    <p v-if="message" class="message">{{ message }}</p>
    <div v-if="refreshing" class="refresh-bar">正在刷新…</div>

    <main
      class="book-area"
      data-id="refresh_layout"
      :class="{ 'pull-refresh': folderStyle }"
      @pointerdown="onPullStart"
      @pointermove="onPullMove"
      @pointerup="onPullEnd"
    >
      <div
        class="books"
        data-id="rv_bookshelf"
        :class="layout"
        :style="layout === 'grid' ? { gridTemplateColumns: `repeat(${gridColumns}, minmax(0, 1fr))` } : {}"
      >
        <template v-for="item in folderItems" :key="item.type === 'group' ? `group-${item.group.groupId}` : item.book.bookUrl">
          <div
            v-if="item.type === 'group'"
            class="group-item"
            :class="layout"
            @click="openGroup(item.group)"
          >
            <CoverImage
              :name="item.group.groupName"
              :src="item.group.coverUrl"
              :width="layout === 'list' ? 66 : 0"
              :height="layout === 'list' ? 90 : 0"
              :fill="layout === 'grid'"
            />
            <span class="group-name">{{ item.group.groupName }}</span>
          </div>
          <BookItem
            v-else
            :book="item.book"
            :layout="layout"
            :show-last-update-time="configStore.global.showLastUpdateTime"
            :show-unread="configStore.global.showUnread"
            :show-wait-up-count="configStore.global.showWaitUpCount"
            @open="openBook"
            @menu="openInfo"
          />
        </template>
      </div>

      <EmptyState
        v-if="folderItems.length === 0"
        data-id="tv_empty_msg"
        icon="ic_bottom_books_e"
        text="书架还很空哦～快去添加书籍添加进来吧"
        action="添加本地书籍"
        @action="router.push('/import')"
      />
    </main>

    <BookshelfConfigDialog v-model="bookshelfConfigVisible" @saved="onConfigSaved" />
  </div>
</template>

<style scoped>
.bookshelf-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
  background: var(--ac-background);
}
.group-item.list {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 16px 16px 12px;
  border-bottom: 1px solid var(--ac-divider);
  background: var(--ac-background);
  cursor: pointer;
}
.group-item.grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  background: var(--ac-background);
  cursor: pointer;
}
.group-item .group-name {
  font-size: 15px;
  color: var(--ac-primary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.group-item.grid .group-name {
  font-size: 12px;
  text-align: center;
}
.tabs {
  display: flex;
  gap: 4px;
  overflow-x: auto;
  scrollbar-width: none;
}
.tabs::-webkit-scrollbar {
  display: none;
}
.tabs button {
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.78);
  font-size: 14px;
  padding: 8px 10px;
  white-space: nowrap;
  cursor: pointer;
  border-bottom: 2px solid transparent;
}
.tabs button.active {
  color: #ffffff;
  border-bottom-color: #ffffff;
  font-weight: 500;
}
.title-bar.flat .tabs button {
  color: var(--ac-secondary-text);
}
.title-bar.flat .tabs button.active {
  color: var(--ac-primary);
  border-bottom-color: var(--ac-primary);
}
.sort-select {
  max-width: 76px;
  border: none;
  background: transparent;
  color: inherit;
  font-size: 13px;
  outline: none;
  cursor: pointer;
}
.sort-select option {
  color: var(--ac-primary-text);
  background: var(--ac-background);
}
.message {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.book-area {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-bottom: 12px;
}
.refresh-bar {
  padding: 6px 16px;
  font-size: 12px;
  color: var(--ac-primary);
  background: var(--ac-background-menu);
}
.books {
  display: flex;
  flex-direction: column;
}
.books.grid {
  display: grid;
  gap: 6px;
  padding: 8px 12px;
}
.empty {
  min-height: 320px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: var(--ac-secondary-text);
}
.empty p {
  margin: 0;
  font-size: 14px;
}
.empty button {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 8px 18px;
  cursor: pointer;
}
.popup-mask {
  position: fixed;
  inset: 0;
  z-index: 45;
  background: transparent;
}
.popup-menu {
  position: absolute;
  top: 52px;
  right: 8px;
  min-width: 172px;
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 4px 0;
  box-shadow: 0 4px 14px rgba(0, 0, 0, 0.24);
  display: flex;
  flex-direction: column;
}
.popup-menu button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 11px 16px;
  font-size: 14px;
  text-align: left;
  cursor: pointer;
  white-space: nowrap;
}
.popup-menu button:hover {
  background: var(--ac-transparent-10);
}
.popup-menu button.muted {
  color: var(--ac-secondary-text);
}
.mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: flex-end;
  justify-content: center;
  z-index: 30;
}
.menu-sheet,
.book-sheet {
  width: min(100%, 720px);
  background: var(--ac-background);
  border-radius: 8px 8px 0 0;
  padding: 8px 0 calc(8px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  box-shadow: 0 -4px 16px rgba(0, 0, 0, 0.2);
}
.sheet-title {
  margin: 4px 16px 8px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.menu-sheet button,
.book-sheet button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
  padding: 14px 20px;
  font-size: 15px;
  cursor: pointer;
}
.menu-sheet button:hover,
.book-sheet button:hover {
  background: var(--ac-transparent-10);
}
.menu-sheet button.danger,
.book-sheet button.danger {
  color: var(--ac-error);
}
.book-preview {
  display: flex;
  gap: 12px;
  padding: 12px 20px;
  align-items: center;
}
.preview-name {
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}
.preview-meta {
  margin: 4px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
</style>
