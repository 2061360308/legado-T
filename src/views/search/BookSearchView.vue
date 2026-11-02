<script setup lang="ts">
import { computed, onActivated, onMounted, ref, watch } from "vue";
import { onBeforeRouteLeave, useRoute, useRouter } from "vue-router";
import type { Book, OnlineSearchResult } from "@/core/domain";
import type { PopupMenuItem } from "@/components/android/PopupMenu.vue";
import { useBookSearchStore, type SearchScopeValue } from "@/stores/bookSearch";
import { useScrollMemory } from "@/composables/useScrollMemory";
import TitleBar from "@/components/android/TitleBar.vue";
import SearchBar from "@/components/android/SearchBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu from "@/components/android/PopupMenu.vue";
import RefreshProgressBar from "@/components/android/RefreshProgressBar.vue";
import AppLogDialog from "@/views/misc/AppLogDialog.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import SearchInputHelp from "./SearchInputHelp.vue";
import SearchResultItem from "./SearchResultItem.vue";
import SearchFab from "./SearchFab.vue";
import SearchScopeDialog from "./SearchScopeDialog.vue";

const route = useRoute();
const router = useRouter();
const search = useBookSearchStore();
const searchBarRef = ref<InstanceType<typeof SearchBar> | null>(null);
const resultListRef = ref<HTMLElement | null>(null);
const menuVisible = ref(false);
const scopeVisible = ref(false);
const logVisible = ref(false);
const searchFocused = ref(false);
let mountedOnce = false;
useScrollMemory(resultListRef);

const menuItems = computed<PopupMenuItem[]>(() => {
  const items: PopupMenuItem[] = [
    { key: "precision", label: "精准搜索", checked: search.precision },
    { key: "source", label: "书源管理", icon: "ic_cfg_source" },
    { key: "scope", label: "分组或书源", icon: "ic_groups" },
    { key: "log", label: "日志", icon: "ic_cfg_about" },
  ];
  if (search.groups.length) {
    items.push({
      key: "scope:all",
      label: "全部书源",
      checked: search.scope.type === "all",
    });
    for (const group of search.groups) {
      items.push({
        key: `scope:group:${group}`,
        label: group,
        checked:
          search.scope.type === "groups" && search.scope.groups.includes(group),
      });
    }
  }
  return items;
});

const showInputHelp = computed(
  () => search.inputHelpVisible || !search.keyword.trim(),
);

const fabMode = computed<"stop" | "continue" | null>(() => {
  if (search.running) return "stop";
  if (!search.manualStopped && search.hasMore && search.results.length) return "continue";
  return null;
});

const emptyMessage = computed(() =>
  search.precision
    ? "当前为精准搜索，是否关闭精准搜索？"
    : `范围「${search.scopeLabel}」搜索结果为空，是否切换到全部书源？`,
);

function onBack() {
  if (searchFocused.value) {
    searchBarRef.value?.blur();
    searchFocused.value = false;
    search.inputHelpVisible = true;
    return;
  }
  router.back();
}

function onSearchFocus() {
  searchFocused.value = true;
  search.inputHelpVisible = true;
  if (!search.books.length) void search.loadBooks();
  if (!search.history.length) void search.loadHistory();
}

function onSearchBlur() {
  window.setTimeout(() => {
    searchFocused.value = false;
    if (search.running || (search.results.length && search.keyword.trim())) {
      search.inputHelpVisible = false;
    } else {
      search.inputHelpVisible = true;
    }
  }, 120);
}

function onSubmit() {
  search.inputHelpVisible = false;
  searchFocused.value = false;
  searchBarRef.value?.blur();
  void search.startSearch(search.keyword);
}

function selectHistory(keyword: string) {
  search.keyword = keyword;
  search.inputHelpVisible = true;
  const inShelf = search.books.some((book) => book.name === keyword);
  if (inShelf) return;
  search.inputHelpVisible = false;
  void search.startSearch(keyword);
}

function onClearHistory() {
  if (!search.history.length) return;
  if (!confirm("清空全部搜索历史？")) return;
  void search.clearHistory();
}

function openShelfBook(book: Book) {
  search.pause();
  router.push({ path: "/book-info", query: { bookUrl: book.bookUrl } });
}

function openResult(result: OnlineSearchResult) {
  if (!result.bookUrl) return;
  search.pause();
  router.push({
    path: "/book-info",
    query: {
      bookUrl: result.bookUrl,
      sourceUrl: String(result.bookSourceUrl || result.origin || ""),
      name: String(result.name || ""),
      author: String(result.author || ""),
      preview: "1",
    },
  });
}

function onScroll(event: Event) {
  const element = event.currentTarget as HTMLElement;
  if (element.scrollHeight - element.scrollTop - element.clientHeight < 120) {
    void search.nextPage();
  }
}

function onMenu(key: string) {
  if (key === "precision") {
    search.setPrecision(!search.precision);
  } else if (key === "source") {
    router.push("/book-source");
  } else if (key === "scope") {
    scopeVisible.value = true;
  } else if (key === "log") {
    logVisible.value = true;
  } else if (key === "scope:all") {
    chooseScope({ type: "all", groups: [], sourceUrl: "" });
  } else if (key.startsWith("scope:group:")) {
    chooseScope({
      type: "groups",
      groups: [key.slice("scope:group:".length)],
      sourceUrl: "",
    });
  }
}

function chooseScope(scope: SearchScopeValue) {
  search.setScope(scope);
  if (search.keyword.trim() && !search.inputHelpVisible) {
    void search.startSearch(search.keyword);
  }
}

function onEmptyConfirm() {
  search.emptyDialogVisible = false;
  if (search.precision) {
    search.setPrecision(false);
  } else {
    search.setScope({ type: "all", groups: [], sourceUrl: "" });
    if (search.keyword.trim()) void search.startSearch(search.keyword);
  }
}

watch(
  () => search.keyword,
  (value, oldValue) => {
    if (value === oldValue) return;
    if (search.running) search.stop();
    search.inputHelpVisible = true;
  },
);

onMounted(async () => {
  await Promise.all([search.loadSources(), search.loadBooks(), search.loadHistory()]);
  const sourceUrl = String(route.query.sourceUrl || "");
  if (sourceUrl) {
    search.setScope({ type: "source", groups: [], sourceUrl });
  }
  const key = String(route.query.key || "").trim();
  if (key) {
    search.keyword = key;
    await search.startSearch(key);
  } else if (search.keyword.trim()) {
    search.inputHelpVisible = false;
  }
  mountedOnce = true;
});

onActivated(() => {
  search.resume();
  if (mountedOnce) {
    void Promise.all([
      search.loadSources(),
      search.loadBooks(),
      search.loadHistory(),
    ]);
  }
});

onBeforeRouteLeave((to) => {
  if (searchFocused.value) {
    searchBarRef.value?.blur();
    searchFocused.value = false;
    search.inputHelpVisible = true;
    return false;
  }
  if (["/", "/explore", "/rss-sources", "/config"].includes(to.path)) {
    search.reset();
  } else {
    search.pause();
  }
});
</script>

<template>
  <div class="book-search">
    <TitleBar search-only @back="onBack">
      <template #content>
        <SearchBar
          ref="searchBarRef"
          v-model="search.keyword"
          placeholder="搜索书名、作者"
          submit-button
          @submit="onSubmit"
          @focus="onSearchFocus"
          @blur="onSearchBlur"
        />
      </template>
      <template #actions>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>

    <RefreshProgressBar :active="search.running" />

    <PopupMenu v-model="menuVisible" :items="menuItems" @select="onMenu" />
    <SearchScopeDialog
      v-model="scopeVisible"
      :sources="search.sources"
      :groups="search.groups"
      :scope="search.scope"
      @select="chooseScope"
    />

    <SearchInputHelp
      v-if="showInputHelp"
      :books="search.bookshelfSuggestions"
      :history="search.filteredHistory"
      @open-book="openShelfBook"
      @select-history="selectHistory"
      @delete-history="search.deleteHistory"
      @clear-history="onClearHistory"
    />

    <main
      v-else
      ref="resultListRef"
      class="result-list"
      @scroll.passive="onScroll"
    >
      <SearchResultItem
        v-for="result in search.results"
        :key="`${result.name}\u0000${result.author}\u0000${result.bookUrl}`"
        :result="result"
        :in-shelf="search.isInBookshelf(result)"
        @open="openResult(result)"
      />
      <p v-if="!search.running && !search.results.length" class="empty-result">
        没有找到结果
      </p>
    </main>

    <SearchFab
      v-if="fabMode"
      :mode="fabMode"
      @stop="search.stop"
      @continue="search.nextPage"
    />

    <AndroidDialog
      :model-value="search.emptyDialogVisible"
      mode="center"
      title="搜索结果为空"
      @update:model-value="search.emptyDialogVisible = $event"
    >
      <p class="empty-message">{{ emptyMessage }}</p>
      <template #footer>
        <button class="btn" @click="search.emptyDialogVisible = false">取消</button>
        <button class="btn primary" @click="onEmptyConfirm">确定</button>
      </template>
    </AndroidDialog>

    <AppLogDialog v-model="logVisible" />
  </div>
</template>

<style scoped>
.book-search {
  position: relative;
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.result-list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding-bottom: calc(72px + var(--safe-bottom, 0px));
}
.empty-result {
  margin: 24px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.empty-message {
  margin: 0;
  color: var(--ac-primary-text);
  font-size: 14px;
  line-height: 1.5;
}
.btn {
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px 16px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
</style>
