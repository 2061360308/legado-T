<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import {
  bookSourceDeleteVariable,
  bookSourceGet,
  bookSourceGetVariables,
  bookSourceSetVariable,
} from "@/core/application/bookSources";
import { rssSourceList } from "@/core/application/rss";
import { platform } from "@/runtime";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

interface VariableRow {
  key: string;
  value: string;
  original: boolean;
}

const route = useRoute();
const router = useRouter();
const sourceUrl = computed(() => String(route.query.url || ""));
const sourceType = computed(() => String(route.query.type || "bookSource"));
const sourceName = ref("");
const rows = ref<VariableRow[]>([]);
const originalKeys = ref<string[]>([]);
const loading = ref(false);
const saving = ref(false);
const message = ref("");

async function load() {
  if (!sourceUrl.value) return;
  loading.value = true;
  try {
    const [variables, sourceNameValue] = await Promise.all([
      bookSourceGetVariables(sourceUrl.value),
      sourceType.value === "rss"
        ? rssSourceList().then(
            (list) =>
              list.find((item) => item.sourceUrl === sourceUrl.value)?.sourceName ||
              sourceUrl.value,
          )
        : bookSourceGet(sourceUrl.value).then(
            (source) => source?.bookSourceName || sourceUrl.value,
          ),
    ]);
    sourceName.value = sourceNameValue;
    const entries = Object.entries(variables || {});
    originalKeys.value = entries.map(([key]) => key);
    rows.value = entries.map(([key, value]) => ({
      key,
      value: String(value),
      original: true,
    }));
  } catch (error) {
    message.value = String(error);
  } finally {
    loading.value = false;
  }
}

function addRow() {
  rows.value.push({ key: "", value: "", original: false });
}

function removeRow(index: number) {
  rows.value.splice(index, 1);
}

async function importVariables() {
  const path = await platform.dialog.open({
    multiple: false,
    filters: [{ name: "变量 JSON", extensions: ["json", "txt"] }],
  });
  if (typeof path !== "string" || !path) return;
  try {
    const bytes = await platform.files.readFile(path);
    const parsed = JSON.parse(new TextDecoder("utf-8").decode(bytes)) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      throw new Error("变量文件必须是 JSON 对象");
    }
    const entries = Object.entries(parsed as Record<string, unknown>).map(
      ([key, value]) => ({ key, value: String(value), original: true }),
    );
    rows.value = entries;
    await save();
    message.value = `已导入 ${entries.length} 个变量`;
  } catch (error) {
    message.value = String(error);
  }
}

async function exportVariables() {
  try {
    const map = Object.fromEntries(
      rows.value
        .map((row) => [row.key.trim(), row.value])
        .filter(([key]) => key),
    );
    const path = await platform.dialog.save({
      defaultPath: "source-variables.json",
      filters: [{ name: "变量 JSON", extensions: ["json"] }],
    });
    if (!path) return;
    await platform.files.writeFile(
      path,
      new TextEncoder().encode(JSON.stringify(map, null, 2)),
    );
    message.value = "变量已导出";
  } catch (error) {
    message.value = String(error);
  }
}

async function save() {
  if (saving.value) return;
  const currentKeys = new Set(
    rows.value.map((row) => row.key.trim()).filter(Boolean),
  );
  saving.value = true;
  message.value = "";
  try {
    for (const key of originalKeys.value) {
      if (!currentKeys.has(key)) {
        await bookSourceDeleteVariable(sourceUrl.value, key);
      }
    }
    for (const row of rows.value) {
      const key = row.key.trim();
      if (!key) continue;
      await bookSourceSetVariable(sourceUrl.value, key, row.value);
    }
    await load();
    message.value = "变量已保存";
  } catch (error) {
    message.value = String(error);
  } finally {
    saving.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="variables-page">
    <TitleBar :title="sourceType === 'rss' ? '订阅源变量' : '书源变量'" @back="router.back()">
      <template #actions>
        <button aria-label="保存" :disabled="saving || loading" @click="save">
          <Icon name="ic_save" :size="24" />
        </button>
      </template>
    </TitleBar>

    <main class="content">
      <p class="source-name">{{ sourceName }}</p>
      <div class="row">
        <button :disabled="loading" @click="importVariables">导入变量</button>
        <button :disabled="loading" @click="exportVariables">导出变量</button>
      </div>
      <p v-if="message" class="message">{{ message }}</p>
      <p v-if="loading" class="message">加载中…</p>

      <div v-if="!loading" class="table">
        <div class="head">
          <span>键</span>
          <span>值</span>
          <span />
        </div>
        <div v-for="(row, index) in rows" :key="index" class="row">
          <input v-model="row.key" placeholder="变量名" />
          <input v-model="row.value" placeholder="变量值" />
          <button class="delete" @click="removeRow(index)">删除</button>
        </div>
        <p v-if="!rows.length" class="empty">暂无变量</p>
      </div>

      <button class="add" :disabled="loading" @click="addRow">新增变量</button>
      <p class="hint">变量会写入 `source_variables`，供书源规则和登录流程复用。</p>
    </main>
  </div>
</template>

<style scoped>
.variables-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 14px 24px;
}
.source-name {
  margin: 0 0 8px;
  color: var(--ac-primary-text);
  font-size: 15px;
}
.message {
  margin: 0 0 8px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.row {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}
.row button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 6px 10px;
  font-size: 12px;
}
.table {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.head,
.row {
  display: grid;
  grid-template-columns: 1fr 1.4fr auto;
  gap: 6px;
  align-items: center;
}
.head {
  color: var(--ac-secondary-text);
  font-size: 11px;
}
input {
  min-width: 0;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 7px 8px;
  border-radius: 3px;
  font-size: 13px;
}
.delete {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 3px;
  padding: 6px 8px;
  font-size: 12px;
}
.empty {
  margin: 8px 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.add {
  margin-top: 12px;
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 8px 12px;
}
.hint {
  margin: 10px 0 0;
  color: var(--ac-secondary-text);
  font-size: 11px;
  line-height: 1.5;
}
</style>
