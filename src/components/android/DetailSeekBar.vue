<script setup lang="ts">
import { computed } from "vue";
import Icon from "./Icon.vue";

const props = withDefaults(
  defineProps<{
    title: string;
    modelValue: number;
    min?: number;
    max: number;
    step?: number;
    display?: (value: number) => string;
  }>(),
  {
    min: 0,
    step: 1,
    display: undefined,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: number];
  change: [value: number];
}>();

const shown = computed(() =>
  props.display ? props.display(props.modelValue) : String(props.modelValue),
);

function setValue(value: number) {
  const next = Math.min(props.max, Math.max(props.min, value));
  if (next === props.modelValue) return;
  emit("update:modelValue", next);
  emit("change", next);
}
</script>

<template>
  <div class="detail-seek">
    <span class="seek-title">{{ title }}</span>
    <button class="seek-icon" aria-label="减少" @click="setValue(modelValue - step)">
      <Icon name="ic_reduce" :size="20" />
    </button>
    <input
      class="seek-bar"
      type="range"
      :min="min"
      :max="max"
      :step="step"
      :value="modelValue"
      @input="setValue(Number(($event.target as HTMLInputElement).value))"
    />
    <button class="seek-icon" aria-label="增加" @click="setValue(modelValue + step)">
      <Icon name="ic_add" :size="20" />
    </button>
    <span class="seek-value">{{ shown }}</span>
  </div>
</template>

<style scoped>
.detail-seek {
  display: flex;
  align-items: center;
  gap: 4px;
  min-height: 40px;
  padding: 2px 16px;
}
.seek-title {
  width: 60px;
  flex: none;
  font-size: 14px;
  color: var(--ac-primary-text);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.seek-icon {
  width: 28px;
  height: 28px;
  flex: none;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--ac-primary-text);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
}
.seek-icon:hover {
  background: var(--ac-transparent-10);
}
.seek-bar {
  flex: 1;
  min-width: 0;
  height: 25px;
  accent-color: var(--ac-primary);
}
.seek-value {
  width: 60px;
  flex: none;
  text-align: right;
  font-size: 13px;
  color: var(--ac-primary-text);
  white-space: nowrap;
}
</style>
