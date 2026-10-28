<script setup lang="ts">
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AndroidDialog from "./AndroidDialog.vue";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  apply: [];
}>();

const prefs = useReaderPrefsStore();

const fonts = [
  { name: "系统默认", value: 'system-ui, "Microsoft YaHei", "PingFang SC", sans-serif' },
  { name: "黑体", value: '"SimHei", "Heiti SC", "Noto Sans SC", sans-serif' },
  { name: "宋体", value: '"SimSun", "Songti SC", "Noto Serif SC", serif' },
  { name: "楷体", value: '"KaiTi", "Kaiti SC", serif' },
  { name: "仿宋", value: '"FangSong", "Fangsong SC", serif' },
  { name: "衬线", value: "Georgia, serif" },
  { name: "无衬线", value: "Arial, sans-serif" },
];

function choose(value: string) {
  prefs.patch({ fontFamily: value });
  emit("apply");
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="选择字体"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="fonts">
      <button
        v-for="font in fonts"
        :key="font.name"
        class="font-row"
        :class="{ active: prefs.fontFamily === font.value }"
        :style="{ fontFamily: font.value }"
        @click="choose(font.value)"
      >
        <span>{{ font.name }}</span>
        <span class="sample">山海之间，风起云涌。Aa</span>
      </button>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.fonts {
  display: flex;
  flex-direction: column;
}
.font-row {
  display: flex;
  align-items: center;
  gap: 10px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  padding: 12px 4px;
  cursor: pointer;
  text-align: left;
}
.font-row span:first-child {
  width: 80px;
  flex: none;
  font-size: 14px;
}
.sample {
  flex: 1;
  font-size: 16px;
}
.font-row.active {
  color: var(--ac-primary);
}
</style>
