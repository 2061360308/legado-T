<script setup lang="ts">
import { computed } from "vue";
import type { OnlineSearchResult } from "@/core/domain";
import CoverImage from "@/components/android/CoverImage.vue";
import Badge from "@/components/android/Badge.vue";
import LabelsBar from "@/components/android/LabelsBar.vue";

const props = defineProps<{
  result: OnlineSearchResult;
  inShelf: boolean;
}>();

const emit = defineEmits<{ open: [] }>();

const name = computed(() => String(props.result.name || "未命名"));
const author = computed(() => String(props.result.author || "未知作者"));
const kinds = computed(() =>
  String(props.result.kind || "")
    .split(/[,，、/]/)
    .map((item) => item.trim())
    .filter(Boolean),
);
const latest = computed(() => String(props.result.latestChapterTitle || "").trim());
const intro = computed(() => String(props.result.intro || "").trim());
</script>

<template>
  <article class="result-item" @click="emit('open')">
    <CoverImage
      :name="name"
      :author="author"
      :src="result.coverUrl ? String(result.coverUrl) : null"
      :width="80"
      :height="110"
    />
    <div class="main">
      <div class="name-row">
        <span v-if="inShelf" class="shelf-dot" aria-label="已在书架" />
        <span class="name">{{ name }}</span>
        <Badge :count="result.originCount ?? 1" />
      </div>
      <p class="author">作者：{{ author }}</p>
      <LabelsBar :labels="kinds" />
      <p v-if="latest" class="latest">最新：{{ latest }}</p>
      <p v-if="intro" class="intro">{{ intro }}</p>
    </div>
  </article>
</template>

<style scoped>
.result-item {
  display: flex;
  align-items: flex-start;
  min-height: 126px;
  cursor: pointer;
}
.result-item:active {
  background: var(--ac-transparent-10);
}
.result-item :deep(.cover) {
  flex: none;
  margin: 8px;
}
.main {
  flex: 1;
  min-width: 0;
  padding: 8px 8px 8px 0;
}
.name-row {
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
}
.shelf-dot {
  flex: none;
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #4caf50;
}
.name {
  flex: 1;
  min-width: 0;
  color: var(--ac-primary-text);
  font-size: 16px;
  line-height: 1.3;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.author,
.latest,
.intro {
  margin: 3px 0 0;
  color: var(--ac-primary-text);
  font-size: 12px;
  line-height: 1.35;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.intro {
  margin-top: 4px;
  color: var(--ac-secondary-text);
  white-space: normal;
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
}
</style>
