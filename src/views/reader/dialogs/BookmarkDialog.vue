<script setup lang="ts">
import { ref, watch } from "vue";
import type { Bookmark } from "@/core/domain";
import { bookmarkDelete, bookmarkUpdate } from "@/core/application/bookmarks";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean; bookmark: Bookmark | null }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  saved: [];
}>();

const text = ref("");
const note = ref("");
const busy = ref(false);
const error = ref("");

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    text.value = props.bookmark?.bookText ?? "";
    note.value = props.bookmark?.content ?? "";
    error.value = "";
  },
);

async function save() {
  if (!props.bookmark || busy.value) return;
  busy.value = true;
  try {
    await bookmarkUpdate(props.bookmark.time, text.value, note.value.trim() || null);
    emit("saved");
    emit("update:modelValue", false);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function remove() {
  if (!props.bookmark || busy.value) return;
  if (!confirm("删除这条书签？")) return;
  busy.value = true;
  try {
    await bookmarkDelete(props.bookmark.time);
    emit("saved");
    emit("update:modelValue", false);
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="书签"
    width="min(92vw, 480px)"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <p v-if="bookmark?.chapterName" class="chapter-name">{{ bookmark.chapterName }}</p>
    <div class="fields">
      <label>
        <span>摘录</span>
        <textarea v-model="text" rows="4" />
      </label>
      <label>
        <span>备注</span>
        <textarea v-model="note" rows="3" placeholder="可留空" />
      </label>
      <p v-if="error" class="error">{{ error }}</p>
    </div>
    <template #footer>
      <button class="danger" :disabled="busy" @click="remove">删除</button>
      <span class="spacer" />
      <button :disabled="busy" @click="emit('update:modelValue', false)">取消</button>
      <button class="accent" :disabled="busy" @click="save">保存</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.fields {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.chapter-name {
  margin: 0 0 10px;
  color: var(--ac-primary-text);
  font-size: 14px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
textarea {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px;
  font: inherit;
  resize: vertical;
}
.error {
  margin: 0;
  color: var(--ac-error);
  font-size: 12px;
}
button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 14px;
  padding: 8px 10px;
  cursor: pointer;
}
button.danger {
  color: var(--ac-error);
}
.spacer {
  flex: 1;
}
</style>
