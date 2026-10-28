<script setup lang="ts">
import { ref, watch } from "vue";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{
  modelValue: boolean;
  title: string;
  min: number;
  max: number;
  value: number;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [value: number];
}>();

const current = ref(props.value);

watch(
  () => props.modelValue,
  (open) => {
    if (open) current.value = props.value;
  },
);

function set(value: number) {
  const next = Number.isFinite(value) ? Math.round(value) : props.min;
  current.value = Math.min(props.max, Math.max(props.min, next));
}

function confirm() {
  emit("confirm", current.value);
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    :title="title"
    width="min(92vw, 420px)"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="picker">
      <button class="picker-btn" aria-label="减少" @click="set(current - 1)">
        <Icon name="ic_reduce" :size="22" />
      </button>
      <input
        class="picker-input"
        type="number"
        :min="min"
        :max="max"
        :value="current"
        @input="set(Number(($event.target as HTMLInputElement).value))"
      />
      <button class="picker-btn" aria-label="增加" @click="set(current + 1)">
        <Icon name="ic_add" :size="22" />
      </button>
    </div>
    <template #footer>
      <button class="dialog-btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="dialog-btn primary" @click="confirm">确定</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.picker {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 0;
}
.picker-btn {
  width: 40px;
  height: 40px;
  flex: none;
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.picker-input {
  flex: 1;
  min-width: 0;
  height: 40px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  font-size: 16px;
  text-align: center;
  padding: 0 8px;
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
