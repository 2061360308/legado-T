<script setup lang="ts">
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AndroidDialog from "./AndroidDialog.vue";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  apply: [];
}>();

const prefs = useReaderPrefsStore();

function setEnabled(enabled: boolean) {
  prefs.patch({ autoReadSeconds: enabled ? Math.max(prefs.autoReadSeconds, 30) : 0 });
  emit("apply");
}

function setInterval(value: number) {
  prefs.patch({ autoReadSeconds: value });
  emit("apply");
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="sheet"
    title="自动翻页"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <label class="check">
      <input
        type="checkbox"
        :checked="prefs.autoReadSeconds > 0"
        @change="setEnabled(($event.target as HTMLInputElement).checked)"
      />
      开启自动翻页
    </label>
    <label>
      翻页间隔 {{ prefs.autoReadSeconds > 0 ? `${prefs.autoReadSeconds} 秒` : "关闭" }}
      <input
        type="range"
        min="5"
        max="120"
        step="5"
        :value="Math.max(prefs.autoReadSeconds, 5)"
        @input="setInterval(Number(($event.target as HTMLInputElement).value))"
      />
    </label>
    <p class="hint">到达书籍末尾后自动翻页会停止并提示。</p>
  </AndroidDialog>
</template>

<style scoped>
label {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
  margin-bottom: 14px;
}
label.check {
  flex-direction: row;
  align-items: center;
  gap: 6px;
}
input[type="range"] {
  width: 100%;
  accent-color: var(--ac-primary);
}
.hint {
  margin: 0;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
</style>
