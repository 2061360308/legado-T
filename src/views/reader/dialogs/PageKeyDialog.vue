<script setup lang="ts">
import { ref } from "vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AndroidDialog from "./AndroidDialog.vue";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const prefs = useReaderPrefsStore();
const capturing = ref<"prev" | "next" | null>(null);

function capture(which: "prev" | "next", event: KeyboardEvent) {
  event.preventDefault();
  const key = event.key;
  if (which === "prev") prefs.patch({ pageKeyPrev: key });
  else prefs.patch({ pageKeyNext: key });
  capturing.value = null;
}

function reset() {
  prefs.patch({ pageKeyPrev: "ArrowLeft", pageKeyNext: "ArrowRight" });
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
      上一页按键
      <input
        :value="capturing === 'prev' ? '请按键…' : prefs.pageKeyPrev"
        readonly
        @focus="capturing = 'prev'"
        @keydown="capture('prev', $event)"
      />
    </label>
    <label>
      下一页按键
      <input
        :value="capturing === 'next' ? '请按键…' : prefs.pageKeyNext"
        readonly
        @focus="capturing = 'next'"
        @keydown="capture('next', $event)"
      />
    </label>
    <p class="hint">点击输入框后按下键盘按键即可绑定；默认左右方向键。</p>
    <button class="reset" @click="reset">恢复默认</button>
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
.reset {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 8px 14px;
  cursor: pointer;
}
</style>
