<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { useScrollMemory } from "@/composables/useScrollMemory";
import { kvGet, kvSet } from "@/core/application/config";
import { fetchRemoteText } from "@/utils/remoteFetch";
import { bookSourceImport } from "@/core/application/bookSources";
import { rssSourceImport } from "@/core/application/rss";
import { replaceRuleImport } from "@/core/application/rules";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

interface RuleSub {
  id: number;
  name: string;
  url: string;
  type: number;
  customOrder: number;
}

const STORAGE_KEY = "rss:ruleSubs";
const TYPE_LABELS = ["书源", "RSS 源", "替换规则"];
const router = useRouter();
const listRef = ref<HTMLElement | null>(null);
useScrollMemory(listRef);
const items = ref<RuleSub[]>([]);
const loading = ref(false);
const message = ref("");
const editing = ref(false);
const form = ref<RuleSub>(emptyItem());
const dragIndex = ref(-1);

const sorted = computed(() =>
  [...items.value].sort((a, b) => a.customOrder - b.customOrder),
);

function emptyItem(): RuleSub {
  return {
    id: Date.now(),
    name: "",
    url: "",
    type: 1,
    customOrder: Date.now(),
  };
}

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 2600);
}

async function load() {
  loading.value = true;
  try {
    const raw = await kvGet(STORAGE_KEY);
    const parsed = raw ? (JSON.parse(raw) as RuleSub[]) : [];
    items.value = Array.isArray(parsed) ? parsed : [];
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

async function persist() {
  await kvSet(STORAGE_KEY, JSON.stringify(items.value));
}

function addItem() {
  form.value = emptyItem();
  editing.value = true;
}

function editItem(item: RuleSub) {
  form.value = { ...item };
  editing.value = true;
}

async function saveItem() {
  if (!form.value.name.trim() || !form.value.url.trim()) {
    showMessage("名称和 URL 不能为空");
    return;
  }
  const next = { ...form.value, name: form.value.name.trim(), url: form.value.url.trim() };
  const index = items.value.findIndex((item) => item.id === next.id);
  if (index >= 0) items.value[index] = next;
  else items.value.push(next);
  await persist();
  editing.value = false;
}

async function removeItem(item: RuleSub) {
  if (!confirm(`删除规则订阅「${item.name}」？`)) return;
  items.value = items.value.filter((value) => value.id !== item.id);
  await persist();
}

async function openItem(item: RuleSub) {
  if (!item.url) return;
  try {
    const text = await fetchRemoteText(item.url);
    if (item.type === 0) {
      const count = await bookSourceImport(text);
      showMessage(`已导入 ${count} 个书源`);
    } else if (item.type === 1) {
      const count = await rssSourceImport(text);
      showMessage(`已导入 ${count} 个订阅源`);
    } else {
      const report = await replaceRuleImport(text);
      showMessage(`已导入 ${report.entries.length} 条替换规则`);
    }
  } catch (error) {
    showMessage(String(error));
  }
}

function onDragStart(index: number) {
  dragIndex.value = index;
}

async function onDrop(index: number) {
  const from = dragIndex.value;
  dragIndex.value = -1;
  if (from < 0 || from === index) return;
  const ordered = [...sorted.value];
  const [item] = ordered.splice(from, 1);
  ordered.splice(index, 0, item);
  items.value = ordered.map((value, order) => ({ ...value, customOrder: order + 1 }));
  await persist();
}

onMounted(load);
</script>

<template>
  <div class="rule-sub-page">
    <TitleBar title="规则订阅" @back="router.push('/rss-sources')">
      <template #actions>
        <button aria-label="新增" @click="addItem">
          <Icon name="ic_add" :size="24" />
        </button>
      </template>
    </TitleBar>

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <main v-else ref="listRef" class="list">
      <div
        v-for="(item, index) in sorted"
        :key="item.id"
        class="row"
        draggable="true"
        @dragstart="onDragStart(index)"
        @dragover.prevent
        @drop="onDrop(index)"
      >
        <button class="main" @click="openItem(item)">
          <span class="name">{{ item.name }}</span>
          <span class="meta">
            <span class="type">{{ TYPE_LABELS[item.type] || "未知" }}</span>
            <span class="url">{{ item.url }}</span>
          </span>
        </button>
        <button class="icon-button" aria-label="编辑" @click="editItem(item)">
          <Icon name="ic_edit" :size="22" />
        </button>
        <button class="icon-button" aria-label="删除" @click="removeItem(item)">
          <Icon name="ic_outline_delete" :size="22" />
        </button>
      </div>
      <p v-if="!sorted.length" class="empty">暂无规则订阅</p>
    </main>

    <AndroidDialog v-model="editing" mode="center" title="规则订阅">
      <div class="form">
        <label>
          类型
          <select v-model.number="form.type">
            <option :value="0">书源</option>
            <option :value="1">RSS 源</option>
            <option :value="2">替换规则</option>
          </select>
        </label>
        <label>名称<input v-model="form.name" /></label>
        <label>URL<input v-model="form.url" placeholder="https://example.com/sources.json" /></label>
      </div>
      <template #footer>
        <button @click="editing = false">取消</button>
        <button @click="saveItem">保存</button>
      </template>
    </AndroidDialog>
  </div>
</template>

<style scoped>
.rule-sub-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.message,
.loading {
  margin: 8px 16px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.list {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}
.row {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 8px 10px;
  border-bottom: 1px solid var(--ac-divider);
}
.main {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  text-align: left;
  color: var(--ac-primary-text);
  padding: 4px;
}
.name {
  display: block;
  font-size: 16px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.meta {
  display: flex;
  align-items: center;
  gap: 6px;
  margin-top: 4px;
}
.type {
  flex: none;
  border-radius: 2px;
  background: var(--ac-primary);
  color: #fff;
  font-size: 11px;
  padding: 1px 5px;
}
.url {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--ac-secondary-text);
  font-size: 11px;
}
.icon-button {
  width: 36px;
  height: 36px;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 50%;
}
.icon-button:active {
  background: var(--ac-transparent-10);
}
.empty {
  padding: 24px;
  text-align: center;
  color: var(--ac-secondary-text);
}
.form {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.form input,
.form select {
  padding: 9px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
}
</style>
