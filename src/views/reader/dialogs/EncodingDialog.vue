<script setup lang="ts">
import { ref, watch } from "vue";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean; value?: string | null }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [value: string];
}>();

const charsets = ["UTF-8", "GBK", "GB18030", "UTF-16LE", "UTF-16BE", "BIG5"];
const current = ref(props.value || "UTF-8");

watch(
  () => props.modelValue,
  (open) => {
    if (open) current.value = props.value || "UTF-8";
  },
);

function confirm() {
  emit("confirm", current.value.trim() || "UTF-8");
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="设置编码"
    width="min(92vw, 420px)"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <label class="field">
      <span>字符集</span>
      <input v-model="current" placeholder="charset name" />
    </label>
    <div class="chips">
      <button
        v-for="charset in charsets"
        :key="charset"
        class="chip"
        :class="{ active: current === charset }"
        @click="current = charset"
      >
        {{ charset }}
      </button>
    </div>
    <p class="hint">修改编码后会重新解析目录和章节。</p>
    <template #footer>
      <button class="dialog-btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="dialog-btn primary" @click="confirm">确定</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.field input {
  height: 40px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 0 10px;
  font-size: 15px;
}
.chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 12px;
}
.chip {
  border: 1px solid var(--ac-divider);
  border-radius: 14px;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 5px 10px;
  font-size: 12px;
  cursor: pointer;
}
.chip.active {
  border-color: var(--ac-primary);
  color: var(--ac-primary);
}
.hint {
  margin: 12px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.dialog-btn {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 14px;
  padding: 8px 12px;
  cursor: pointer;
}
</style>
