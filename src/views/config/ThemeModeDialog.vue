<script setup lang="ts">
import { ThemeMode, useThemeStore } from "@/stores/theme";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const theme = useThemeStore();

const modes = [
  { value: ThemeMode.FollowSystem, label: "跟随系统" },
  { value: ThemeMode.Day, label: "日间" },
  { value: ThemeMode.Night, label: "夜间" },
  { value: ThemeMode.Eink, label: "墨水屏" },
];

function choose(mode: ThemeMode) {
  theme.setMode(mode);
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="主题模式"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="modes">
      <button
        v-for="mode in modes"
        :key="mode.value"
        class="mode-row"
        :class="{ active: theme.mode === mode.value }"
        @click="choose(mode.value)"
      >
        <span>{{ mode.label }}</span>
        <span class="check">{{ theme.mode === mode.value ? "●" : "○" }}</span>
      </button>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.modes {
  display: flex;
  flex-direction: column;
}
.mode-row {
  display: flex;
  align-items: center;
  min-height: 50px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  font-size: 15px;
  padding: 0 4px;
  cursor: pointer;
  text-align: left;
}
.mode-row span:first-child {
  flex: 1;
}
.check {
  color: var(--ac-primary);
}
.mode-row.active {
  color: var(--ac-primary);
}
</style>
