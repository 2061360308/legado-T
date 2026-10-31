<script setup lang="ts">
withDefaults(
  defineProps<{
    count: number;
    text?: string;
    mainText?: string;
    addText?: string;
  }>(),
  {
    text: "已选择",
    mainText: "",
    addText: "",
  },
);

defineEmits<{
  main: [];
  add: [];
  enable: [];
  disable: [];
  remove: [];
  cancel: [];
}>();
</script>

<template>
  <div v-if="count > 0" class="action-bar">
    <span class="count">{{ text }} {{ count }} 项</span>
    <button v-if="mainText" @click="$emit('main')">{{ mainText }}</button>
    <button v-if="addText" @click="$emit('add')">{{ addText }}</button>
    <button @click="$emit('enable')">启用</button>
    <button @click="$emit('disable')">禁用</button>
    <button class="danger" @click="$emit('remove')">删除</button>
    <button @click="$emit('cancel')">取消</button>
  </div>
</template>

<style scoped>
.action-bar {
  flex: none;
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: var(--ac-background-menu);
  border-top: 1px solid var(--ac-divider);
}
.count {
  flex: 1;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 13px;
  padding: 6px 8px;
  cursor: pointer;
}
button.danger {
  color: var(--ac-error);
}
</style>
