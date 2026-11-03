<script setup lang="ts">
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

defineProps<{
  modelValue: boolean;
  source: string;
  message: string;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  "open-log": [];
}>();

function openLog() {
  emit("open-log");
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="出错了"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <p v-if="source" class="source">{{ source }}</p>
    <pre class="message">{{ message }}</pre>
    <template #footer>
      <button class="btn" @click="openLog">查看日志</button>
      <button class="btn primary" @click="emit('update:modelValue', false)">
        知道了
      </button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.source {
  margin: 0 0 8px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.message {
  margin: 0;
  max-height: 50vh;
  overflow: auto;
  white-space: pre-wrap;
  word-break: break-word;
  color: var(--ac-primary-text);
  font-family: inherit;
  font-size: 13px;
  line-height: 1.55;
}
.btn {
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 7px 14px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
</style>
