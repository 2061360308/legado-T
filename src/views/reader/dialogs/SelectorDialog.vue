<script setup lang="ts">
import AndroidDialog from "./AndroidDialog.vue";

defineProps<{
  modelValue: boolean;
  title: string;
  items: string[];
  selected?: number;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  select: [index: number];
}>();

function choose(index: number) {
  emit("select", index);
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
    <div class="selector-list">
      <button
        v-for="(item, index) in items"
        :key="item"
        class="selector-row"
        @click="choose(index)"
      >
        {{ item }}
      </button>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.selector-list {
  display: flex;
  flex-direction: column;
  margin: -16px -20px;
  max-height: 60vh;
  overflow-y: auto;
}
.selector-row {
  min-height: 48px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 14px;
  text-align: left;
  padding: 10px 20px;
  cursor: pointer;
}
.selector-row:hover {
  background: var(--ac-transparent-10);
}
</style>
