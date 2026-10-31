<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import type { HttpTts } from "@/core/domain";
import { httpTtsDelete, httpTtsEngines } from "@/core/application/rules";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AndroidDialog from "./AndroidDialog.vue";
import HttpTtsEditDialog from "./HttpTtsEditDialog.vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const prefs = useReaderPrefsStore();
const engines = ref<HttpTts[]>([]);
const editVisible = ref(false);
const editing = ref<HttpTts | null>(null);
const selected = ref(prefs.ttsEngine);

async function load() {
  try {
    engines.value = await httpTtsEngines();
  } catch {
    engines.value = [];
  }
}

function select(id: string) {
  selected.value = id;
}

function confirm() {
  prefs.patch({ ttsEngine: selected.value });
  emit("update:modelValue", false);
}

function add() {
  editing.value = null;
  editVisible.value = true;
}

function edit(engine: HttpTts) {
  editing.value = engine;
  editVisible.value = true;
}

async function remove(engine: HttpTts) {
  if (!window.confirm(`删除语音引擎「${engine.name}」？`)) return;
  await httpTtsDelete(engine.id);
  if (selected.value === String(engine.id)) selected.value = "";
  await load();
}

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      selected.value = prefs.ttsEngine;
      void load();
    }
  },
);

onMounted(load);
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="sheet"
    title="语音引擎"
    height="86vh"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <template #actions>
      <button @click="add">新增</button>
    </template>
    <div class="list">
      <button class="engine-row" :class="{ active: !selected }" @click="select('')">
        <span class="radio">{{ !selected ? "●" : "○" }}</span>
        <span class="info">
          <span class="name">系统 TTS</span>
          <span class="meta">使用宿主系统语音合成</span>
        </span>
      </button>
      <div v-for="engine in engines" :key="engine.id" class="engine-row" :class="{ active: selected === String(engine.id) }">
        <button class="select" @click="select(String(engine.id))">
          <span class="radio">{{ selected === String(engine.id) ? "●" : "○" }}</span>
          <span class="info">
            <span class="name">{{ engine.name }}</span>
            <span class="meta">{{ engine.url }}</span>
          </span>
        </button>
        <button class="small" @click="edit(engine)">编辑</button>
        <button class="small danger" @click="remove(engine)">删除</button>
      </div>
    </div>
    <template #footer>
      <button class="btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="btn primary" @click="confirm">确定</button>
    </template>

    <HttpTtsEditDialog v-model="editVisible" :engine="editing" @saved="load" />
  </AndroidDialog>
</template>

<style scoped>
.list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.engine-row {
  display: flex;
  align-items: center;
  gap: 6px;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  padding: 8px;
  background: transparent;
  color: var(--ac-primary-text);
  text-align: left;
}
.engine-row.active {
  border-color: var(--ac-primary);
}
.engine-row .select {
  flex: 1;
}
button.select,
button.engine-row {
  display: flex;
  align-items: center;
  gap: 8px;
  border: none;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
  padding: 0;
}
.radio {
  color: var(--ac-primary);
  font-size: 14px;
}
.info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}
.name {
  font-size: 14px;
}
.meta {
  font-size: 11px;
  color: var(--ac-secondary-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.small {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 4px 8px;
  font-size: 12px;
  cursor: pointer;
}
.small.danger {
  color: var(--ac-error);
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
