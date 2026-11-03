<script setup lang="ts">
import { computed, ref, watch } from "vue";
import type { RuleImportItem, RuleImportState } from "@/utils/ruleImport";

const props = defineProps<{
  modelValue: boolean;
  title: string;
  items: RuleImportItem[];
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [keys: string[]];
}>();

const selected = ref(new Set<string>());
const labels: Record<RuleImportState, string> = {
  new: "新增",
  update: "更新",
  existing: "已有",
};

const selectedCount = computed(() => selected.value.size);
const allSelected = computed(
  () => props.items.length > 0 && selected.value.size === props.items.length,
);

function reset() {
  selected.value = new Set(
    props.items.filter((item) => item.selected).map((item) => item.key),
  );
}

watch(
  () => [props.modelValue, props.items] as const,
  ([open]) => {
    if (open) reset();
  },
);

function toggle(key: string) {
  const next = new Set(selected.value);
  if (next.has(key)) next.delete(key);
  else next.add(key);
  selected.value = next;
}

function toggleAll() {
  selected.value = allSelected.value
    ? new Set()
    : new Set(props.items.map((item) => item.key));
}

function close() {
  emit("update:modelValue", false);
}

function confirm() {
  if (!selected.value.size) return;
  const keys = props.items
    .filter((item) => selected.value.has(item.key))
    .map((item) => item.key);
  emit("confirm", keys);
  close();
}
</script>

<template>
  <div
    v-if="modelValue"
    class="import-dialog"
    @click.self="close"
  >
    <section class="panel" role="dialog" aria-modal="true">
      <header class="head">
        <h2>{{ title }}</h2>
        <span>{{ items.length }} 条</span>
        <button aria-label="关闭" @click="close">×</button>
      </header>
      <div class="list">
        <label v-for="item in items" :key="item.key" class="row">
          <input
            type="checkbox"
            :checked="selected.has(item.key)"
            @change="toggle(item.key)"
          />
          <span class="main">
            <span class="name">{{ item.title }}</span>
            <span v-if="item.subtitle" class="subtitle">{{ item.subtitle }}</span>
          </span>
          <span class="state" :class="item.state">{{ labels[item.state] }}</span>
        </label>
        <p v-if="!items.length" class="empty">没有可导入的规则</p>
      </div>
      <footer class="foot">
        <label class="select-all">
          <input type="checkbox" :checked="allSelected" @change="toggleAll" />
          全选
        </label>
        <button class="btn" @click="close">取消</button>
        <button class="btn primary" :disabled="!selectedCount" @click="confirm">
          确定
        </button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.import-dialog {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: rgba(0, 0, 0, 0.4);
}
.panel {
  display: flex;
  flex-direction: column;
  width: min(100%, 460px);
  max-height: min(86vh, 720px);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
  overflow: hidden;
}
.head {
  flex: none;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  border-bottom: 1px solid var(--ac-divider);
}
.head h2 {
  flex: 1;
  min-width: 0;
  margin: 0;
  font-size: 16px;
  font-weight: 500;
}
.head span {
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.head button {
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  font-size: 24px;
  line-height: 1;
  padding: 0 2px;
  cursor: pointer;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  overscroll-behavior: contain;
}
.row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  border-bottom: 1px solid var(--ac-divider);
  cursor: pointer;
}
.row input {
  flex: none;
  width: 16px;
  height: 16px;
  accent-color: var(--ac-primary);
}
.main {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.name,
.subtitle {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.name {
  font-size: 14px;
}
.subtitle {
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.state {
  flex: none;
  border: 1px solid currentColor;
  border-radius: 2px;
  padding: 1px 5px;
  font-size: 11px;
}
.state.new {
  color: #4caf50;
}
.state.update {
  color: var(--ac-primary);
}
.state.existing {
  color: var(--ac-secondary-text);
}
.empty {
  margin: 24px 16px;
  text-align: center;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.foot {
  flex: none;
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px calc(10px + env(safe-area-inset-bottom));
  border-top: 1px solid var(--ac-divider);
}
.select-all {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--ac-secondary-text);
  font-size: 13px;
  cursor: pointer;
}
.select-all input {
  width: 16px;
  height: 16px;
  accent-color: var(--ac-primary);
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
.btn:disabled {
  opacity: 0.4;
  cursor: default;
}
</style>
