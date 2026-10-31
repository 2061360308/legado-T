<script setup lang="ts">
import { ref, watch } from "vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const prefs = useReaderPrefsStore();
const capturing = ref<"prev" | "next" | null>(null);
const prevKeys = ref(prefs.pageKeyPrev);
const nextKeys = ref(prefs.pageKeyNext);

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    prevKeys.value = prefs.pageKeyPrev;
    nextKeys.value = prefs.pageKeyNext;
    capturing.value = null;
  },
);

function appendKey(target: "prev" | "next", event: KeyboardEvent) {
  event.preventDefault();
  if (event.key === "Backspace" || event.key === "Delete") {
    if (target === "prev") prevKeys.value = "";
    else nextKeys.value = "";
    return;
  }
  if (event.key === "Escape" || event.key === "Tab") return;
  const key = event.key === " " ? "Space" : event.key;
  const current = target === "prev" ? prevKeys.value : nextKeys.value;
  const next = !current || current.endsWith(",") ? `${current}${key}` : `${current},${key}`;
  if (target === "prev") prevKeys.value = next;
  else nextKeys.value = next;
}

function reset() {
  prevKeys.value = "";
  nextKeys.value = "";
}

function confirm() {
  prefs.patch({ pageKeyPrev: prevKeys.value, pageKeyNext: nextKeys.value });
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="自定义翻页键"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <label>
      <span>上一页按键</span>
      <input
        :value="capturing === 'prev' ? '请按键…' : prevKeys"
        readonly
        @focus="capturing = 'prev'"
        @blur="capturing = null"
        @keydown="appendKey('prev', $event)"
      />
    </label>
    <label>
      <span>下一页按键</span>
      <input
        :value="capturing === 'next' ? '请按键…' : nextKeys"
        readonly
        @focus="capturing = 'next'"
        @blur="capturing = null"
        @keydown="appendKey('next', $event)"
      />
    </label>
    <p class="hint">点击输入框后按键即可绑定，多个按键用逗号分隔。</p>
    <template #footer>
      <button class="btn" @click="reset">重置</button>
      <button class="btn primary" @click="confirm">确定</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 13px;
  margin-bottom: 12px;
}
label span {
  color: var(--ac-secondary-text);
  font-size: 12px;
}
input {
  padding: 9px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
}
.hint {
  margin: 0 0 12px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.btn {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 14px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
}
</style>
