<script setup lang="ts">
import { ref } from "vue";
import { useReaderPrefsStore, type ClickAction } from "@/stores/readerPrefs";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const prefs = useReaderPrefsStore();
const editing = ref<number | null>(null);

const labels: Record<ClickAction, string> = {
  none: "无动作",
  menu: "菜单",
  next: "下一页",
  prev: "上一页",
  nextChapter: "下一章",
  prevChapter: "上一章",
  readAloudPrev: "朗读上一段",
  readAloudNext: "朗读下一段",
  addBookmark: "添加书签",
  editContent: "编辑正文",
  replaceState: "生效替换",
  chapterList: "目录",
  search: "搜索",
  syncProgress: "同步进度",
  readAloudToggle: "朗读暂停/恢复",
};

const options = Object.entries(labels) as [ClickAction, string][];

function choose(index: number) {
  editing.value = index;
}

function apply(action: ClickAction) {
  if (editing.value === null) return;
  const list = [...prefs.clickActions];
  list[editing.value] = action;
  prefs.patch({ clickActions: list });
  editing.value = null;
}

function close() {
  emit("update:modelValue", false);
}
</script>

<template>
  <div v-if="modelValue" class="click-mask">
    <div class="click-header">
      <span>点击区域设置</span>
      <button aria-label="关闭" @click="close">×</button>
    </div>
    <div class="click-grid">
      <button
        v-for="(action, index) in prefs.clickActions"
        :key="index"
        class="click-cell"
        @click="choose(index)"
      >
        {{ labels[action] ?? "无动作" }}
      </button>
    </div>

    <div v-if="editing !== null" class="action-mask" @click.self="editing = null">
      <div class="action-list">
        <h3>选择动作 · 区域 {{ editing + 1 }}</h3>
        <button v-for="[value, label] in options" :key="value" @click="apply(value)">
          {{ label }}
        </button>
        <button class="cancel" @click="editing = null">取消</button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.click-mask {
  position: fixed;
  inset: 0;
  z-index: 70;
  background: rgba(52, 52, 52, 0.6);
  padding: 3px;
  display: flex;
  flex-direction: column;
}
.click-header {
  flex: none;
  margin: 3px;
  padding: 6px 16px;
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
  display: flex;
  align-items: center;
  border-radius: 2px;
}
.click-header span {
  flex: 1;
  font-size: 14px;
}
.click-header button {
  border: none;
  background: transparent;
  color: #fff;
  font-size: 24px;
  line-height: 1;
  cursor: pointer;
}
.click-grid {
  flex: 1;
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  grid-template-rows: repeat(3, 1fr);
  gap: 6px;
  padding: 3px;
}
.click-cell {
  border: none;
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
  font-size: 14px;
  cursor: pointer;
  border-radius: 2px;
}
.click-cell:hover {
  background: rgba(255, 255, 255, 0.26);
}
.action-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2;
}
.action-list {
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 4px;
  width: min(340px, 86%);
  max-height: 70vh;
  overflow-y: auto;
  padding: 12px 0;
  display: flex;
  flex-direction: column;
}
.action-list h3 {
  margin: 4px 16px 8px;
  font-size: 14px;
}
.action-list button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
  padding: 12px 20px;
  font-size: 14px;
  cursor: pointer;
}
.action-list button:hover {
  background: var(--ac-transparent-10);
}
.action-list .cancel {
  color: var(--ac-primary);
  border-top: 1px solid var(--ac-divider);
  margin-top: 6px;
}
</style>
