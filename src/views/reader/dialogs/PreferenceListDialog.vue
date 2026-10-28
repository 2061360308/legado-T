<script setup lang="ts">
import { computed, ref, watch } from "vue";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{
  modelValue: boolean;
  title: string;
  entries: string[];
  entryValues: string[];
  value: string;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [value: string];
}>();

const selected = ref(props.value);

watch(
  () => props.modelValue,
  (open) => {
    if (open) selected.value = props.value;
  },
);

const selectedIndex = computed(() => {
  const index = props.entryValues.indexOf(selected.value);
  if (index >= 0) return index;
  const fallback = props.entryValues.indexOf(props.value);
  return fallback >= 0 ? fallback : 0;
});

function choose(index: number) {
  selected.value = props.entryValues[index] ?? "";
}

function confirm() {
  emit("confirm", selected.value);
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
    <div class="choice-list">
      <button
        v-for="(entry, index) in entries"
        :key="entry"
        class="choice-row"
        :class="{ active: selectedIndex === index }"
        @click="choose(index)"
      >
        <span class="choice-label">{{ entry }}</span>
        <Icon v-if="selectedIndex === index" name="ic_check" :size="20" class="choice-check" />
      </button>
    </div>
    <template #footer>
      <button class="dialog-btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="dialog-btn primary" @click="confirm">确定</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.choice-list {
  display: flex;
  flex-direction: column;
  max-height: 56vh;
  overflow-y: auto;
  margin: -16px -20px;
}
.choice-row {
  min-height: 48px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 20px;
  font-size: 14px;
  text-align: left;
  cursor: pointer;
}
.choice-row:hover {
  background: var(--ac-transparent-10);
}
.choice-label {
  flex: 1;
  min-width: 0;
}
.choice-check {
  color: var(--ac-primary);
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
