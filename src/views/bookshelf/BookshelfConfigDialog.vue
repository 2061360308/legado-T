<script setup lang="ts">
import { useConfigStore, type GlobalConfig } from "@/stores/config";
import { BOOK_SORT_LABELS } from "@/core/application/books";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  saved: [];
}>();

const config = useConfigStore();

const layoutOptions = ["列表", "三列网格", "四列网格", "五列网格", "六列网格"];
const sortOptions = Object.entries(BOOK_SORT_LABELS).map(([value, label]) => ({
  value: Number(value),
  label,
}));

function update<K extends keyof GlobalConfig>(key: K, value: GlobalConfig[K]) {
  config.global[key] = value;
  config.persist(key);
}

function close() {
  emit("saved");
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="书架布局"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="prefs">
      <label class="pref-row">
        <span>分组样式</span>
        <select
          :value="config.global.bookGroupStyle"
          @change="update('bookGroupStyle', Number(($event.target as HTMLSelectElement).value))"
        >
          <option :value="0">Tab</option>
          <option :value="1">Folder</option>
        </select>
      </label>
      <label class="pref-row">
        <span>显示未读</span>
        <input
          type="checkbox"
          :checked="config.global.showUnread"
          @change="update('showUnread', ($event.target as HTMLInputElement).checked)"
        />
      </label>
      <label class="pref-row">
        <span>显示最近更新时间</span>
        <input
          type="checkbox"
          :checked="config.global.showLastUpdateTime"
          @change="update('showLastUpdateTime', ($event.target as HTMLInputElement).checked)"
        />
      </label>
      <label class="pref-row">
        <span>显示待更新数量</span>
        <input
          type="checkbox"
          :checked="config.global.showWaitUpCount"
          @change="update('showWaitUpCount', ($event.target as HTMLInputElement).checked)"
        />
      </label>
      <label class="pref-row">
        <span>显示快速滚动条</span>
        <input
          type="checkbox"
          :checked="config.global.showBookshelfFastScroller"
          @change="update('showBookshelfFastScroller', ($event.target as HTMLInputElement).checked)"
        />
      </label>

      <div class="columns">
        <div class="column">
          <h3>显示</h3>
          <label v-for="(label, index) in layoutOptions" :key="label">
            <input
              type="radio"
              :checked="config.global.bookshelfLayout === index"
              @change="update('bookshelfLayout', index)"
            />
            {{ label }}
          </label>
        </div>
        <div class="column">
          <h3>排序</h3>
          <label v-for="option in sortOptions" :key="option.value">
            <input
              type="radio"
              :checked="config.global.bookshelfSort === option.value"
              @change="update('bookshelfSort', option.value)"
            />
            {{ option.label }}
          </label>
        </div>
      </div>
    </div>
    <template #footer>
      <button class="btn primary" @click="close">确定</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.prefs {
  display: flex;
  flex-direction: column;
}
.pref-row {
  display: flex;
  align-items: center;
  min-height: 44px;
  border-bottom: 1px solid var(--ac-divider);
  font-size: 14px;
  gap: 8px;
}
.pref-row span:first-child {
  flex: 1;
}
select {
  padding: 4px 8px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
}
input[type="checkbox"],
input[type="radio"] {
  accent-color: var(--ac-primary);
}
.columns {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 16px;
  padding-top: 12px;
}
.column {
  display: flex;
  flex-direction: column;
  gap: 8px;
}
.column h3 {
  margin: 0;
  font-size: 15px;
  color: var(--ac-primary);
}
.column label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
}
.btn {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 16px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
</style>
