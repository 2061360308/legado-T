<script setup lang="ts">
import { computed, onBeforeUnmount } from "vue";
import type { Book } from "@/core/domain";
import Icon from "@/components/android/Icon.vue";
import CoverImage from "@/components/android/CoverImage.vue";

const props = withDefaults(
  defineProps<{
    book: Book;
    layout: "list" | "grid";
    showLastUpdateTime?: boolean;
    showUnread?: boolean;
    showWaitUpCount?: boolean;
  }>(),
  {
    showLastUpdateTime: true,
    showUnread: true,
    showWaitUpCount: false,
  },
);
const emit = defineEmits<{ open: [book: Book]; menu: [book: Book] }>();

let pressTimer: number | undefined;
let longPressed = false;

function fmtTimeAgo(ts?: number): string {
  if (!ts) return "";
  const diff = Math.max(0, Date.now() / 1000 - ts);
  if (diff < 60) return "刚刚";
  if (diff < 3600) return `${Math.floor(diff / 60)} 分钟前`;
  if (diff < 86400) return `${Math.floor(diff / 3600)} 小时前`;
  if (diff < 86400 * 30) return `${Math.floor(diff / 86400)} 天前`;
  return new Date(ts * 1000).toLocaleDateString("zh-CN");
}

const author = computed(() => props.book.author || "未知作者");
const isLocalBook = computed(() => ((props.book.type ?? 0) & 16) !== 0);
const progress = computed(() => props.book.durChapterTitle || "暂未开始阅读");
const latest = computed(() => props.book.latestChapterTitle || "暂无最新章节");
const updateTime = computed(() => fmtTimeAgo(props.book.syncTime || props.book.durChapterTime));
const unreadCount = computed(() => {
  const total = props.book.totalChapterNum ?? 0;
  const read = (props.book.durChapterIndex ?? -1) + 1;
  return Math.max(total - read, 0);
});

function startPress() {
  window.clearTimeout(pressTimer);
  longPressed = false;
  pressTimer = window.setTimeout(() => {
    longPressed = true;
    emit("menu", props.book);
  }, 480);
}

function cancelPress() {
  window.clearTimeout(pressTimer);
}

function open() {
  if (pressTimer) {
    window.clearTimeout(pressTimer);
    pressTimer = undefined;
  }
  if (longPressed) {
    longPressed = false;
    return;
  }
  emit("open", props.book);
}

onBeforeUnmount(() => window.clearTimeout(pressTimer));
</script>

<template>
  <div
    v-if="layout === 'list'"
    class="book-row"
    data-id="cv_content"
    @click="open"
    @contextmenu.prevent="emit('menu', book)"
    @pointerdown="startPress"
    @pointerup="cancelPress"
    @pointerleave="cancelPress"
  >
    <CoverImage :name="book.name" :author="book.author" :src="book.customCoverUrl || book.coverUrl" :width="66" :height="90" data-id="iv_cover" />
    <div class="row-text">
      <div class="name-line">
        <span class="name" data-id="tv_name">{{ book.name }}</span>
        <span v-if="showUnread && unreadCount > 0" class="badge" data-id="bv_unread">{{ unreadCount > 99 ? "99+" : unreadCount }}</span>
      </div>
      <div class="meta-line">
        <Icon name="ic_author" :size="18" class="meta-icon" data-id="iv_author" />
        <span class="author" data-id="tv_author">{{ author }}</span>
        <span v-if="showLastUpdateTime && !isLocalBook" class="time" data-id="tv_last_update_time">{{ updateTime }}</span>
      </div>
      <div class="meta-line">
        <Icon name="ic_history" :size="18" class="meta-icon" data-id="iv_read" />
        <span class="summary" data-id="tv_read">{{ progress }}</span>
      </div>
      <div class="meta-line last">
        <Icon name="ic_book_last" :size="18" class="meta-icon" data-id="iv_last" />
        <span class="summary" data-id="tv_last">{{ latest }}</span>
      </div>
    </div>
  </div>

  <div
    v-else
    class="book-grid"
    @click="open"
    @contextmenu.prevent="emit('menu', book)"
    @pointerdown="startPress"
    @pointerup="cancelPress"
    @pointerleave="cancelPress"
  >
    <div class="grid-cover-wrap">
      <CoverImage :name="book.name" :author="book.author" :src="book.customCoverUrl || book.coverUrl" fill class="grid-cover" />
      <span v-if="showUnread && unreadCount > 0" class="badge grid-badge" data-id="bv_unread">{{ unreadCount > 99 ? "99+" : unreadCount }}</span>
    </div>
    <span class="grid-name">{{ book.name }}</span>
  </div>
</template>

<style scoped>
.book-row {
  display: flex;
  gap: 10px;
  padding: 16px 16px 12px;
  background: var(--ac-background);
  cursor: pointer;
  border-bottom: 1px solid var(--ac-divider);
  -webkit-tap-highlight-color: transparent;
}
.book-row:active {
  background: var(--ac-transparent-10);
}
.row-text {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.name-line {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-top: -2px;
  padding-left: 2px;
  padding-bottom: 4px;
}
.name {
  font-size: 16px;
  color: var(--ac-primary-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.badge {
  flex: none;
  min-width: 16px;
  height: 16px;
  border-radius: 8px;
  background: var(--ac-error);
  color: #fff;
  font-size: 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 4px;
}
.grid-badge {
  position: absolute;
  top: 2px;
  right: 2px;
  margin: 5px;
}
.meta-line {
  display: flex;
  align-items: center;
  gap: 3px;
  min-height: 20px;
  font-size: 13px;
  color: var(--ac-summary-text);
  padding-left: 2px;
  padding-right: 6px;
}
.meta-line.last {
  padding-bottom: 0;
}
.meta-icon {
  opacity: 0.7;
}
.author {
  flex: none;
  max-width: 45%;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.time {
  margin-left: auto;
  flex: none;
  font-size: 12px;
}
.summary {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.book-grid {
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 8px;
  background: var(--ac-background);
  cursor: pointer;
  -webkit-tap-highlight-color: transparent;
}
.grid-cover-wrap {
  position: relative;
  width: 100%;
}
.grid-cover {
  width: 100%;
  height: auto;
  aspect-ratio: 3 / 4;
}
.grid-name {
  font-size: 12px;
  line-height: 1.35;
  color: var(--ac-primary-text);
  text-align: center;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
