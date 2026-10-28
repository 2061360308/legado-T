<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { open } from "@tauri-apps/plugin-dialog";
import { importBook, type ImportResult } from "@/api/books";
import TitleBar from "@/components/android/TitleBar.vue";

const router = useRouter();
const filePath = ref("");
const bookName = ref("");
const charset = ref("");
const importing = ref(false);
const error = ref("");
const result = ref<ImportResult | null>(null);

const CHARSETS = ["", "UTF-8", "GBK", "UTF-16LE", "UTF-16BE"];

async function pickFile() {
  const picked = await open({
    multiple: false,
    filters: [{ name: "书籍文件", extensions: ["txt", "TXT", "epub", "EPUB", "pdf", "PDF"] }],
  });
  if (typeof picked === "string") {
    filePath.value = picked;
    if (!bookName.value) {
      const stem = picked.split(/[\\/]/).pop() ?? "";
      bookName.value = stem.replace(/\.[^.]+$/, "");
    }
  }
}

async function doImport() {
  if (!filePath.value) {
    error.value = "请先选择 TXT/EPUB/PDF 文件";
    return;
  }
  importing.value = true;
  error.value = "";
  try {
    result.value = await importBook(
      filePath.value,
      bookName.value || null,
      null,
      charset.value || null,
      null,
    );
  } catch (e) {
    error.value = String(e);
  } finally {
    importing.value = false;
  }
}

function goBookshelf() {
  router.push("/");
}

function goBookInfo() {
  if (result.value) {
    router.push({ path: "/book-info", query: { bookUrl: result.value.book.bookUrl } });
  }
}
</script>

<template>
  <div class="import">
    <TitleBar title="导入书籍" @back="goBookshelf" />

    <section class="form">
      <button @click="pickFile">选择文件</button>
      <p class="path">{{ filePath || "未选择文件" }}</p>

      <label>
        书名（默认取文件名，可修改）
        <input v-model="bookName" placeholder="书名" />
      </label>

      <label>
        字符集（留空自动探测）
        <select v-model="charset">
          <option v-for="c in CHARSETS" :key="c" :value="c">
            {{ c || "自动探测" }}
          </option>
        </select>
      </label>

      <button :disabled="importing" @click="doImport">
        {{ importing ? "分章中…" : "导入" }}
      </button>
      <p v-if="error" class="error">{{ error }}</p>
    </section>

    <section v-if="result" class="result">
      <h2>{{ result.book.name }}</h2>
      <p>共 {{ result.chapterCount }} 章 · 编码 {{ result.charset }}</p>
      <p class="intro">{{ result.book.intro || "（无简介）" }}</p>
      <div class="result-actions">
        <button @click="goBookInfo">查看书籍详情</button>
        <button @click="goBookshelf">返回书架</button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.import {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.form {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  width: 100%;
  max-width: 520px;
  margin: 0 auto;
}
.path {
  color: #888;
  font-size: 12px;
  word-break: break-all;
  margin: 0;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 14px;
}
input,
select {
  padding: 6px;
  border-radius: var(--lr-radius);
  border: 1px solid #ccc;
}
.error {
  color: #d32f2f;
  font-size: 13px;
}
.result {
  margin: 8px auto 24px;
  padding: 12px;
  width: calc(100% - 32px);
  max-width: 488px;
  background: var(--ac-background-card);
  border-radius: 2px;
}
.result-actions {
  display: flex;
  gap: 8px;
  margin-top: 10px;
}
.result-actions button {
  flex: 1;
}
.intro {
  font-size: 13px;
  color: #666;
  max-height: 120px;
  overflow-y: auto;
}
</style>