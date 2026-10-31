<script setup lang="ts">
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AndroidDialog from "./AndroidDialog.vue";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  apply: [];
  toc: [];
  menu: [];
  stop: [];
  settings: [];
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
    <label>
      自动翻页速度
      <strong>{{ prefs.autoReadSeconds > 0 ? `${prefs.autoReadSeconds} 秒` : "关闭" }}</strong>
      <input
        type="range"
        min="5"
        max="120"
        step="5"
        :value="Math.max(prefs.autoReadSeconds, 5)"
        @input="setInterval(Number(($event.target as HTMLInputElement).value))"
      />
    </label>
    <div class="icon-row">
      <button @click="emit('toc')">目录</button>
      <button @click="emit('menu')">主菜单</button>
      <button @click="emit('stop'); setEnabled(false)">停止</button>
      <button @click="emit('settings')">设置</button>
    </div>
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
.icon-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 8px;
  margin-top: 4px;
}
.icon-row button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 8px 4px;
  font-size: 12px;
}
</style>
