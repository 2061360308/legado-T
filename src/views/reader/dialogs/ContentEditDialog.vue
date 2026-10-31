<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { ReaderSnapshot } from "@/core/domain";
import { readerSaveChapterContent } from "@/core/application/reader";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean; snapshot: ReaderSnapshot | null }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  saved: [];
}>();

const content = ref("");
const original = ref("");
const saving = ref(false);
const error = ref("");
const notice = ref("");
const title = computed(() => props.snapshot?.chapterTitle || "编辑正文");

function loadContent() {
  return (props.snapshot?.chapterParagraphs ?? [])
    .map((paragraph) => paragraph.replace(/^[\s\u3000]+/, ""))
    .join("\n");
}

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    error.value = "";
    notice.value = "";
    original.value = loadContent();
    content.value = original.value;
  },
);

function reset() {
  content.value = original.value;
  notice.value = "已还原为当前缓存正文";
}

async function copyAll() {
  try {
    await navigator.clipboard.writeText(`${title.value}\n${content.value}`);
    notice.value = "已复制全部正文";
  } catch (e) {
    error.value = String(e);
  }
}

async function save() {
  const snapshot = props.snapshot;
  if (!snapshot) return;
  saving.value = true;
  error.value = "";
  try {
    await readerSaveChapterContent(snapshot.book.bookUrl, snapshot.chapterIndex, content.value);
    emit("saved");
    emit("update:modelValue", false);
  } catch (e) {
    error.value = String(e);
  } finally {
    saving.value = false;
  }
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="fullscreen"
    :title="title"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #actions>
      <button @click="reset">还原</button>
      <button @click="copyAll">复制</button>
    </template>
    <div class="editor-wrap">
      <p class="hint">编辑只影响当前章节的本地缓存，不会修改原始 TXT 文件。</p>
      <textarea v-model="content" spellcheck="false" />
      <p v-if="notice" class="notice">{{ notice }}</p>
      <p v-if="error" class="error">{{ error }}</p>
    </div>
    <template #footer>
      <button class="btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="btn primary" :disabled="saving" @click="save">
        {{ saving ? "保存中…" : "保存并重排" }}
      </button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.editor-wrap {
  height: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.hint {
  margin: 0;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
textarea {
  flex: 1;
  min-height: 320px;
  resize: none;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 12px;
  font-size: 15px;
  line-height: 1.7;
  font-family: inherit;
}
.notice {
  margin: 0;
  color: var(--ac-primary);
  font-size: 12px;
}
.error {
  margin: 0;
  color: var(--ac-error);
  font-size: 12px;
}
.btn {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 16px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
.btn:disabled {
  opacity: 0.5;
}
</style>
