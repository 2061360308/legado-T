<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import {
  rssReadRecordClear,
  rssReadRecordList,
} from "@/core/application/rss";
import type { RssReadRecord } from "@/core/domain";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();
const records = ref<RssReadRecord[]>([]);
const message = ref("");

async function load() {
  try {
    records.value = await rssReadRecordList();
  } catch (error) {
    message.value = String(error);
  }
}

async function clearAll() {
  if (!records.value.length) return;
  if (!confirm(`清空 ${records.value.length} 条阅读记录？`)) return;
  await rssReadRecordClear();
  records.value = [];
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) void load();
  },
);

onMounted(() => {
  if (props.modelValue) void load();
});
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="阅读记录"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="toolbar">
      <span>{{ records.length }} 条</span>
      <button :disabled="!records.length" @click="clearAll">清空</button>
    </div>
    <p v-if="message" class="message">{{ message }}</p>
    <div class="records">
      <div v-for="record in records" :key="record.record" class="record">
        <p class="title">{{ record.title || record.record }}</p>
        <p class="link">{{ record.record }}</p>
        <p v-if="record.readTime" class="time">
          {{ new Date(record.readTime).toLocaleString("zh-CN") }}
        </p>
      </div>
      <p v-if="!records.length" class="empty">暂无阅读记录</p>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  border-bottom: 1px solid var(--ac-divider);
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.toolbar button {
  border: none;
  background: transparent;
  color: var(--ac-error);
  font-size: 13px;
}
.records {
  max-height: 60vh;
  overflow-y: auto;
}
.record {
  padding: 10px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.title,
.link,
.time {
  margin: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.title {
  color: var(--ac-primary-text);
  font-size: 14px;
}
.link,
.time {
  margin-top: 3px;
  color: var(--ac-secondary-text);
  font-size: 11px;
}
.message,
.empty {
  padding: 16px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
</style>
