<script setup lang="ts">
import { ref } from "vue";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean; totalChapters: number; currentChapter: number }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  finish: [];
  jump: [chapter: number, pos: number];
}>();

const enabled = ref(true);
const chapter = ref(props.currentChapter + 1);
const pos = ref(0);

function finishAll() {
  emit("finish");
  emit("update:modelValue", false);
}

function jump() {
  const index = Math.min(Math.max(chapter.value - 1, 0), Math.max(props.totalChapters - 1, 0));
  emit("jump", index, pos.value);
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="模拟读完"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <label class="switch-row">
      <span>开启模拟读完</span>
      <input v-model="enabled" type="checkbox" />
    </label>
    <p class="hint">
      模拟读完会把进度直接置为末章末页；也可以从指定章节位置开始。
    </p>
    <div class="grid">
      <label>
        起始章节（1-{{ totalChapters }}）
        <input v-model.number="chapter" type="number" min="1" :max="totalChapters" />
      </label>
      <label>
        章节内位置
        <input v-model.number="pos" type="number" min="0" />
      </label>
    </div>
    <template #footer>
      <button class="btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="btn" :disabled="!enabled" @click="jump">跳到指定位置</button>
      <button class="btn primary" :disabled="!enabled" @click="finishAll">模拟读完</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.switch-row {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 14px;
  min-height: 44px;
}
.switch-row span {
  flex: 1;
}
input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
.hint {
  font-size: 12px;
  color: var(--ac-secondary-text);
  margin: 4px 0 12px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}
input[type="number"] {
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
}
.btn {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 14px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
.btn:disabled {
  opacity: 0.45;
}
</style>
