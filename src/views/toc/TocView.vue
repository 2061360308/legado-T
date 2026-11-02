<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import {
  bookGet,
  chapterList,
  chapterWordCounts,
  reSplitBook,
  txtTocRuleList,
} from "@/core/application/books";
import type { Book, BookChapter, Bookmark, ReplaceRule, TxtTocRule } from "@/core/domain";
import { bookmarkList } from "@/core/application/bookmarks";
import { replaceRules } from "@/core/application/rules";
import TitleBar from "@/components/android/TitleBar.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import Icon from "@/components/android/Icon.vue";
import EmptyState from "@/components/android/EmptyState.vue";
import BookmarkDialog from "@/views/reader/dialogs/BookmarkDialog.vue";
import AppLogDialog from "@/views/misc/AppLogDialog.vue";
import { useReaderSettingsStore } from "@/stores/readerSettings";

const route = useRoute();
const router = useRouter();
const bookUrl = (route.query.bookUrl as string) || "";

const book = ref<Book | null>(null);
const chapters = ref<BookChapter[]>([]);
const bookmarks = ref<Bookmark[]>([]);
const editingBookmark = ref<Bookmark | null>(null);
const bookmarkDialogVisible = ref(false);
const logVisible = ref(false);
const rules = ref<TxtTocRule[]>([]);
const selectedRule = ref("");
const charset = ref("");
const reSplitting = ref(false);
const message = ref("");
const showResplit = ref(false);
const activeTab = ref<"toc" | "bookmark">("toc");
const moreMenuVisible = ref(false);
const searchVisible = ref(false);
const searchKeyword = ref("");
const searchInput = ref<HTMLInputElement | null>(null);
const settings = useReaderSettingsStore();
const isLocalTxt = computed(() =>
  /\.txt$/i.test(book.value?.originName || book.value?.bookUrl || bookUrl),
);
const splitLongChapter = ref(settings.get("tocSplitLongChapter") === true);
const useReplace = ref(settings.get("tocUseReplace") === true);
const loadWordCount = ref(settings.get("tocCountWords") === true);
const reverseToc = ref(settings.get("tocReverse") === true);
const titleRules = ref<ReplaceRule[]>([]);
const wordCounts = ref<Record<number, number>>({});

// book_toc.xml 溢出菜单（目录 tab / 书签 tab 动态显示）
const tocMenuItems = computed<PopupMenuItem[]>(() => {
  if (activeTab.value === "toc") {
    const items: PopupMenuItem[] = [];
    if (isLocalTxt.value) {
      items.push({ key: "txtTocRule", label: "TXT 目录规则" });
      items.push({
        key: "splitLongChapter",
        label: "拆分超长章节",
        checked: splitLongChapter.value,
      });
    }
    items.push({ key: "reverseToc", label: "反转目录", checked: reverseToc.value });
    items.push({ key: "useReplace", label: "使用替换", checked: useReplace.value });
    items.push({ key: "loadWordCount", label: "加载字数", checked: loadWordCount.value });
    items.push({ key: "log", label: "日志", muted: true });
    return items;
  }
  return [
    { key: "exportBookmark", label: "导出" },
    { key: "exportBookmarkMd", label: "导出(MD)" },
    { key: "log", label: "日志", muted: true },
  ];
});

const collapsed = ref(new Set<number>());

const parentIndex = computed(() => {
  const result: number[] = [];
  chapters.value.forEach((chapter, index) => {
    const level = chapter.tocLevel ?? 0;
    let parent = -1;
    if (level > 0) {
      for (let i = index - 1; i >= 0; i -= 1) {
        if ((chapters.value[i].tocLevel ?? 0) < level) {
          parent = i;
          break;
        }
      }
    }
    result.push(parent);
  });
  return result;
});

function isCollapsed(index: number): boolean {
  return collapsed.value.has(index);
}

function hasChildren(index: number): boolean {
  const level = chapters.value[index]?.tocLevel ?? 0;
  return (chapters.value[index + 1]?.tocLevel ?? 0) > level;
}

function isVisible(index: number): boolean {
  let parent = parentIndex.value[index] ?? -1;
  while (parent >= 0) {
    if (collapsed.value.has(parent)) return false;
    parent = parentIndex.value[parent] ?? -1;
  }
  return true;
}

function toggleVolume(index: number) {
  const next = new Set(collapsed.value);
  if (next.has(index)) next.delete(index);
  else next.add(index);
  collapsed.value = next;
}

function displayTitle(chapter: BookChapter): string {
  if (!useReplace.value) return chapter.title;
  let title = chapter.title;
  for (const rule of titleRules.value) {
    if (rule.isEnabled === false || rule.scopeTitle !== true || !rule.pattern) continue;
    try {
      title = rule.isRegex
        ? title.replace(new RegExp(rule.pattern, "gs"), rule.replacement ?? "")
        : title.split(rule.pattern).join(rule.replacement ?? "");
    } catch {
      // Ignore invalid legacy replacement rules.
    }
  }
  return title;
}

const displayChapters = computed(() => {
  const visible = chapters.value.filter((_, index) => isVisible(index));
  return reverseToc.value ? [...visible].reverse() : visible;
});

const filteredChapters = computed(() => {
  const key = searchKeyword.value.trim();
  if (!key) return displayChapters.value;
  const source = reverseToc.value ? [...chapters.value].reverse() : chapters.value;
  return source.filter((ch) => displayTitle(ch).includes(key));
});

const filteredBookmarks = computed(() => {
  const key = searchKeyword.value.trim();
  if (!key) return relatedBookmarks.value;
  return relatedBookmarks.value.filter(
    (item) =>
      (item.chapterName ?? "").includes(key) || (item.bookText ?? "").includes(key),
  );
});

async function loadWordCounts() {
  try {
    const list = await chapterWordCounts(bookUrl);
    wordCounts.value = Object.fromEntries(list.map((item) => [item.index, item.count]));
    message.value = `已加载 ${list.length} 章字数`;
  } catch (e) {
    message.value = String(e);
  }
}

async function onTocMenu(key: string) {
  if (key === "txtTocRule") {
    router.push({ path: "/toc-rules", query: { bookUrl } });
  } else if (key === "splitLongChapter") {
    splitLongChapter.value = !splitLongChapter.value;
    settings.set("tocSplitLongChapter", splitLongChapter.value);
    reSplitting.value = true;
    try {
      const count = await reSplitBook(
        bookUrl,
        selectedRule.value || book.value?.tocUrl || null,
        charset.value || null,
        splitLongChapter.value,
      );
      message.value = `已${splitLongChapter.value ? "拆分" : "合并"}超长章节，共 ${count} 章`;
      await load();
      if (loadWordCount.value) await loadWordCounts();
    } catch (e) {
      message.value = String(e);
    } finally {
      reSplitting.value = false;
    }
  } else if (key === "reverseToc") {
    reverseToc.value = !reverseToc.value;
    settings.set("tocReverse", reverseToc.value);
    message.value = reverseToc.value ? "目录已反转" : "目录已恢复";
  } else if (key === "useReplace") {
    useReplace.value = !useReplace.value;
    settings.set("tocUseReplace", useReplace.value);
    message.value = useReplace.value ? "目录标题已使用替换规则" : "目录标题已使用原文";
  } else if (key === "loadWordCount") {
    loadWordCount.value = !loadWordCount.value;
    settings.set("tocCountWords", loadWordCount.value);
    if (loadWordCount.value) await loadWordCounts();
    else {
      wordCounts.value = {};
      message.value = "已关闭字数显示";
    }
  } else if (key === "exportBookmark") {
    exportBookmarks("json");
  } else if (key === "exportBookmarkMd") {
    exportBookmarks("md");
  } else if (key === "log") {
    logVisible.value = true;
  }
}

function openSearch() {
  searchVisible.value = true;
  nextTick(() => searchInput.value?.focus());
}

function closeSearch() {
  searchVisible.value = false;
  searchKeyword.value = "";
}

function exportBookmarks(kind: "json" | "md") {
  const bookName = book.value?.name || "book";
  const items = relatedBookmarks.value;
  let content: string;
  let type: string;
  let ext: string;
  if (kind === "json") {
    content = JSON.stringify(items, null, 2);
    type = "application/json";
    ext = "json";
  } else {
    const body = items
      .map((item) => {
        const note = item.content ? `\n  > ${item.content}` : "";
        return `- ${item.chapterName ?? ""}\n  ${item.bookText ?? ""}${note}`;
      })
      .join("\n");
    content = `# ${bookName} 书签\n\n${body}\n`;
    type = "text/markdown";
    ext = "md";
  }
  const blob = new Blob([content], { type });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `${bookName}-bookmarks.${ext}`;
  anchor.click();
  URL.revokeObjectURL(url);
  message.value = `已导出 ${items.length} 条书签`;
}
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);

const CHARSETS = ["", "UTF-8", "GBK", "UTF-16LE", "UTF-16BE"];

const relatedBookmarks = computed(() =>
  bookmarks.value.filter((item) => !item.bookUrl || item.bookUrl === bookUrl),
);

const currentInfo = computed(() => {
  if (!book.value) return "";
  const index = (book.value.durChapterIndex ?? 0) + 1;
  return `第 ${index} 章 · ${book.value.durChapterTitle || "未开始"}`;
});

async function load() {
  book.value = await bookGet(bookUrl);
  chapters.value = await chapterList(bookUrl);
  collapsed.value = new Set();
  rules.value = await txtTocRuleList();
  try {
    titleRules.value = await replaceRules();
  } catch {
    titleRules.value = [];
  }
  try {
    bookmarks.value = await bookmarkList();
  } catch {
    bookmarks.value = [];
  }
  selectedRule.value = book.value?.tocUrl || "";
  charset.value = book.value?.charset || "";
}

async function onReSplit() {
  reSplitting.value = true;
  message.value = "";
  try {
    const n = await reSplitBook(
      bookUrl,
      selectedRule.value || null,
      charset.value || null,
      splitLongChapter.value,
    );
    message.value = `已按所选规则重建目录，共 ${n} 章`;
    await load();
  } catch (e) {
    message.value = String(e);
  } finally {
    reSplitting.value = false;
  }
}

function readerPath(): string {
  const type = book.value?.type ?? 0;
  if (type & 2) return "/audio";
  if (type & 4) return "/manga";
  if (type & 8) return "/file-source";
  return "/reader";
}

function openChapter(ch: BookChapter) {
  router.push({ path: readerPath(), query: { bookUrl, chapter: String(ch.index) } });
}

function openBookmark(bookmark: Bookmark) {
  router.push({
    path: readerPath(),
    query: {
      bookUrl: bookmark.bookUrl || bookUrl,
      chapter: String(bookmark.chapterIndex ?? 0),
      chapterPos: String(bookmark.chapterPos ?? 0),
    },
  });
}

function editBookmark(bookmark: Bookmark) {
  editingBookmark.value = bookmark;
  bookmarkDialogVisible.value = true;
}

function scrollTop() {
  listRef.value?.scrollTo({ top: 0, behavior: "smooth" });
}

function scrollBottom() {
  listRef.value?.scrollTo({ top: listRef.value.scrollHeight, behavior: "smooth" });
}

onMounted(load);
</script>

<template>
  <div class="toc">
    <TitleBar data-id="title_bar" title="" @back="router.back()">
      <template #content>
        <div v-if="!searchVisible" class="tabs">
          <button :class="{ active: activeTab === 'toc' }" @click="activeTab = 'toc'">
            目录
          </button>
          <button
            :class="{ active: activeTab === 'bookmark' }"
            @click="activeTab = 'bookmark'"
          >
            书签
          </button>
        </div>
        <label v-else class="search-field">
          <Icon name="ic_search" :size="20" />
          <input
            ref="searchInput"
            v-model="searchKeyword"
            type="search"
            placeholder="搜索"
          />
        </label>
      </template>
      <template #actions>
        <button v-if="searchVisible" aria-label="关闭搜索" @click="closeSearch">
          <Icon name="ic_baseline_close" :size="24" />
        </button>
        <template v-else>
          <button aria-label="搜索" @click="openSearch">
            <Icon name="ic_search" :size="24" />
          </button>
          <button aria-label="更多" @click="moreMenuVisible = true">
            <Icon name="ic_more_vert" :size="24" />
          </button>
        </template>
      </template>
    </TitleBar>

    <PopupMenu v-model="moreMenuVisible" :items="tocMenuItems" @select="onTocMenu" />

    <section v-if="showResplit" class="resplit">
      <label>
        目录规则
        <select v-model="selectedRule">
          <option value="">（兜底分章）</option>
          <option v-for="r in rules" :key="r.id" :value="r.rule">{{ r.name }}</option>
        </select>
      </label>
      <label>
        字符集
        <select v-model="charset">
          <option v-for="c in CHARSETS" :key="c" :value="c">
            {{ c || (book?.charset ? `当前：${book.charset}` : "自动探测") }}
          </option>
        </select>
      </label>
      <button :disabled="reSplitting" @click="onReSplit">
        {{ reSplitting ? "重新分章中…" : "重新分章" }}
      </button>
    </section>

    <p v-if="message" class="msg">{{ message }}</p>

    <main ref="listRef" class="list" data-id="view_pager">
      <template v-if="activeTab === 'toc'">
        <ol class="chapter-list" data-id="recycler_view">
          <li
            v-for="ch in filteredChapters"
            :key="ch.index"
            :class="{ volume: ch.isVolume, current: ch.index === book?.durChapterIndex }"
            :style="{ paddingLeft: `${12 + (ch.tocLevel ?? 0) * 16}px` }"
            @click="openChapter(ch)"
          >
            <button
              v-if="hasChildren(chapters.findIndex((item) => item.index === ch.index))"
              class="toc-toggle"
              :class="{ collapsed: isCollapsed(chapters.findIndex((item) => item.index === ch.index)) }"
              aria-label="展开/收起"
              @click.stop="toggleVolume(chapters.findIndex((item) => item.index === ch.index))"
            >
              <Icon name="ic_expand_more" :size="18" />
            </button>
            <span class="title">{{ displayTitle(ch) }}</span>
            <span v-if="loadWordCount && wordCounts[ch.index]" class="word-count">
              {{ wordCounts[ch.index] }}字
            </span>
            <Icon
              v-if="ch.index === book?.durChapterIndex"
              name="ic_check"
              :size="20"
              class="checked"
            />
          </li>
        </ol>
      </template>
      <template v-else>
        <button
          v-for="bookmark in filteredBookmarks"
          :key="bookmark.time"
          class="bookmark-row"
          @click="openBookmark(bookmark)"
          @contextmenu.prevent="editBookmark(bookmark)"
        >
          <span class="chapter">{{ bookmark.chapterName }}</span>
          <span class="text">{{ bookmark.bookText }}</span>
          <span v-if="bookmark.content" class="note">{{ bookmark.content }}</span>
        </button>
        <EmptyState
          v-if="filteredBookmarks.length === 0"
          icon="ic_bookmark"
          text="本书暂无书签"
        />
      </template>
    </main>

    <div class="base-info" data-id="ll_chapter_base_info">
      <span class="current" data-id="tv_current_chapter_info">{{ currentInfo }}</span>
      <button aria-label="顶部" data-id="iv_chapter_top" @click="scrollTop">
        <Icon name="ic_arrow_drop_up" :size="24" />
      </button>
      <button aria-label="底部" data-id="iv_chapter_bottom" @click="scrollBottom">
        <Icon name="ic_arrow_drop_down" :size="24" />
      </button>
    </div>

    <BookmarkDialog
      v-model="bookmarkDialogVisible"
      :bookmark="editingBookmark"
      @saved="load"
    />
    <AppLogDialog v-model="logVisible" />
  </div>
</template>

<style scoped>
.toc {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.tabs {
  flex: 1;
  display: flex;
  justify-content: center;
  gap: 6px;
}
.search-field {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 0 6px;
  padding: 0 10px;
  height: 34px;
  border-radius: 17px;
  background: rgba(255, 255, 255, 0.2);
  color: #fff;
}
.search-field input {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: inherit;
  font-size: 13px;
  outline: none;
}
.search-field input::placeholder {
  color: rgba(255, 255, 255, 0.65);
}
.tabs button {
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.75);
  font-size: 14px;
  padding: 8px 10px;
  border-bottom: 2px solid transparent;
  cursor: pointer;
}
.tabs button.active {
  color: #fff;
  border-bottom-color: #fff;
  font-weight: 500;
}
.resplit {
  display: flex;
  gap: 10px;
  padding: 10px 16px;
  flex-wrap: wrap;
  border-bottom: 1px solid var(--ac-divider);
  background: var(--ac-background-card);
}
.resplit label {
  flex: 1;
  min-width: 120px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.resplit select {
  padding: 6px;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: var(--ac-background);
  color: var(--ac-primary-text);
}
.resplit button {
  align-self: flex-end;
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 7px 14px;
  cursor: pointer;
}
.resplit button:disabled {
  opacity: 0.5;
}
.msg {
  margin: 6px 16px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.chapter-list {
  margin: 0;
  padding: 0;
  list-style: none;
}
.chapter-list li {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px;
  background: var(--ac-background);
  cursor: pointer;
  font-size: 15px;
  color: var(--ac-primary-text);
}
.chapter-list li:active {
  background: var(--ac-transparent-10);
}
.toc-toggle {
  width: 20px;
  height: 20px;
  flex: none;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  padding: 0;
  background: transparent;
  color: var(--ac-primary);
  cursor: pointer;
  transition: transform 0.15s ease;
}
.toc-toggle.collapsed {
  transform: rotate(-90deg);
}
.title {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chapter-list li.volume {
  background: var(--ac-background-card);
}
.chapter-list li.volume .title {
  font-weight: 600;
  color: var(--ac-primary);
}
.chapter-list li.current .title {
  color: var(--ac-primary);
  font-weight: 500;
}
.word-count {
  flex: none;
  color: var(--ac-secondary-text);
  font-size: 11px;
}
.checked {
  color: var(--ac-secondary-text);
}
.bookmark-row {
  width: 100%;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 0;
  text-align: left;
  cursor: pointer;
}
.bookmark-row:hover {
  background: var(--ac-transparent-10);
}
.chapter,
.text,
.note {
  padding: 4px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chapter {
  font-size: 14px;
}
.text,
.note {
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.base-info {
  flex: none;
  display: flex;
  align-items: center;
  height: 36px;
  padding: 0 10px;
  background: var(--ac-background);
  box-shadow: 0 -2px 6px rgba(0, 0, 0, 0.12);
}
.current {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 12px;
  color: var(--ac-primary-text);
}
.base-info button {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  cursor: pointer;
}
</style>
