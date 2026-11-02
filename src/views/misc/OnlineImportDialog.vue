<script setup lang="ts">
import { ref, watch } from "vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    placeholder?: string;
  }>(),
  {
    title: "在线导入",
    placeholder: "https://example.com/rules.json",
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [url: string];
}>();

const url = ref("");

watch(
  () => props.modelValue,
  (visible) => {
    if (visible) url.value = "";
  },
);

function submit() {
  const value = url.value.trim();
  if (!value) return;
  emit("confirm", value);
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    :title="title"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <label class="field">
      URL
      <input
        v-model="url"
        :placeholder="placeholder"
        @keydown.enter="submit"
      />
    </label>
    <template #footer>
      <button class="btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="btn primary" @click="submit">导入</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.field {
  display: flex;
  flex-direction: column;
  gap: 6px;
  font-size: 13px;
}
input {
  width: 100%;
  box-sizing: border-box;
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: var(--ac-background);
  color: var(--ac-primary-text);
  outline: none;
}
.btn {
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 8px 16px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
</style>
