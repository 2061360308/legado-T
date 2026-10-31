<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import { bookList, bookUpsert } from "@/core/application/books";
import { backupExport, backupImport } from "@/core/application/backup";
import { applySettingsSnapshot, snapshotSettings } from "@/stores/settingsSnapshot";
import { kvGet, kvSet } from "@/core/application/config";
import type { Book } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";

const router = useRouter();
const baseUrl = ref("");
const user = ref("");
const password = ref("");
const backupPassword = ref("");
const log = ref("");
const busy = ref(false);

function append(text: string) {
  log.value = `${new Date().toLocaleTimeString()} ${text}\n${log.value}`;
}

function authHeaders(): Record<string, string> {
  if (!user.value && !password.value) return {};
  const token = btoa(unescape(encodeURIComponent(`${user.value}:${password.value}`)));
  return { Authorization: `Basic ${token}` };
}

function base(): string {
  return baseUrl.value.trim().replace(/\/+$/, "");
}

function fileUrl(path: string): string {
  return `${base()}/${path.replace(/^\/+/, "")}`;
}

async function loadConfig() {
  baseUrl.value = (await kvGet("webdavUrl")) ?? "";
  user.value = (await kvGet("webdavUser")) ?? "";
  password.value = (await kvGet("webdavPassword")) ?? "";
}

async function saveConfig() {
  await kvSet("webdavUrl", baseUrl.value);
  await kvSet("webdavUser", user.value);
  await kvSet("webdavPassword", password.value);
  append("配置已保存");
}

async function propfind(path = ""): Promise<string> {
  const response = await platform.http.fetch(fileUrl(path), {
    method: "PROPFIND",
    headers: {
      ...authHeaders(),
      Depth: "1",
      "Content-Type": "application/xml",
    },
    body: '<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:displayname/><d:getlastmodified/><d:resourcetype/></d:prop></d:propfind>',
  });
  if (!response.ok) throw new Error(`PROPFIND ${response.status}`);
  return response.text();
}

async function testConnection() {
  busy.value = true;
  try {
    await propfind("");
    append("连接成功");
  } catch (e) {
    append(`连接失败：${e}`);
  } finally {
    busy.value = false;
  }
}

async function ensureDirs() {
  busy.value = true;
  try {
    for (const dir of ["bookProgress/", "books/", "background/"]) {
      const response = await platform.http.fetch(fileUrl(dir), {
        method: "MKCOL",
        headers: authHeaders(),
      });
      if (response.ok || response.status === 405 || response.status === 409) {
        append(`目录就绪：${dir}`);
      } else {
        append(`创建目录失败：${dir} (${response.status})`);
      }
    }
  } catch (e) {
    append(`初始化失败：${e}`);
  } finally {
    busy.value = false;
  }
}

async function uploadBackup() {
  busy.value = true;
  try {
    const dir = await platform.files.appDataDir();
    const temp = `${dir.replace(/[\\/]+$/, "")}/backup-upload.zip`;
    await backupExport(temp, backupPassword.value || null, snapshotSettings());
    const bytes = await platform.files.readFile(temp);
    const response = await platform.http.fetch(fileUrl("backup.zip"), {
      method: "PUT",
      headers: { ...authHeaders(), "Content-Type": "application/zip" },
      body: bytes,
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    append("备份已上传为 backup.zip");
  } catch (e) {
    append(`上传失败：${e}`);
  } finally {
    busy.value = false;
  }
}

async function downloadBackup() {
  busy.value = true;
  try {
    const xml = await propfind("");
    const matches = [...xml.matchAll(/<d:href>([^<]+\.zip)<\/d:href>/gi)].map((m) => m[1]);
    const remote = matches.sort().at(-1) ?? "backup.zip";
    const response = await platform.http.fetch(new URL(remote, `${base()}/`).toString(), {
      headers: authHeaders(),
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    const path = await platform.dialog.save({
      defaultPath: decodeURIComponent(remote.split("/").pop() ?? "backup.zip"),
      filters: [{ name: "ZIP 备份", extensions: ["zip"] }],
    });
    if (!path) return;
    await platform.files.writeFile(path, bytes);
    if (confirm("已下载备份，是否立即恢复？")) {
      const report = await backupImport(path, backupPassword.value || null, []);
      const applied = applySettingsSnapshot(report.settingsJson);
      append(`恢复完成：${report.items} 条记录${applied ? "，设置将在重启后生效" : ""}`);
      if (applied) setTimeout(() => window.location.reload(), 800);
    }
  } catch (e) {
    append(`下载失败：${e}`);
  } finally {
    busy.value = false;
  }
}

function progressFileName(book: Book): string {
  const baseName = `${book.name}_${book.author || "未知作者"}`.replace(/[\\/:*?"<>|]/g, "_");
  return `bookProgress/${baseName}.json`;
}

async function syncProgress() {
  busy.value = true;
  let updated = 0;
  let uploaded = 0;
  try {
    const books = await bookList();
    for (const book of books) {
      if (!book.durChapterTitle) continue;
      const url = fileUrl(progressFileName(book));
      try {
        const response = await platform.http.fetch(url, { headers: authHeaders() });
        if (response.ok) {
          const remote = (await response.json()) as {
            chapterIndex?: number;
            chapterPos?: number;
            chapterTitle?: string;
            lastModify?: number;
          };
          const remoteIndex = remote.chapterIndex ?? 0;
          const remotePos = remote.chapterPos ?? 0;
          const localIndex = book.durChapterIndex ?? 0;
          const localPos = book.durChapterPos ?? 0;
          const localTime = book.durChapterTime ?? 0;
          const newer =
            (remote.lastModify ?? 0) > localTime &&
            (remoteIndex > localIndex || (remoteIndex === localIndex && remotePos > localPos));
          if (newer) {
            await bookUpsert({
              ...book,
              durChapterIndex: remoteIndex,
              durChapterPos: remotePos,
              durChapterTitle: remote.chapterTitle ?? book.durChapterTitle,
              durChapterTime: remote.lastModify,
            });
            updated += 1;
            continue;
          }
        }
        const payload = JSON.stringify({
          chapterIndex: book.durChapterIndex ?? 0,
          chapterPos: book.durChapterPos ?? 0,
          chapterTitle: book.durChapterTitle ?? "",
          lastModify: Math.floor(Date.now() / 1000),
        });
        const put = await platform.http.fetch(url, {
          method: "PUT",
          headers: { ...authHeaders(), "Content-Type": "application/json" },
          body: payload,
        });
        if (put.ok) uploaded += 1;
      } catch (e) {
        append(`同步《${book.name}》失败：${e}`);
      }
    }
    append(`进度同步完成：更新本地 ${updated} 本，上传 ${uploaded} 本`);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="webdav-view">
    <TitleBar title="WebDAV" @back="router.back()" />
    <header class="toolbar">
      <button :disabled="busy" @click="loadConfig">读取配置</button>
    </header>

    <section class="card">
      <label>服务器地址<input v-model="baseUrl" placeholder="https://dav.example.com/remote.php/dav/files/user/" /></label>
      <div class="grid">
        <label>账号<input v-model="user" /></label>
        <label>密码<input v-model="password" type="password" /></label>
      </div>
      <label>备份密码（可选）<input v-model="backupPassword" type="password" /></label>
      <div class="row">
        <button :disabled="busy" @click="saveConfig">保存配置</button>
        <button :disabled="busy" @click="testConnection">测试连接</button>
        <button :disabled="busy" @click="ensureDirs">初始化目录</button>
      </div>
    </section>

    <section class="card">
      <h2>备份</h2>
      <div class="row">
        <button :disabled="busy" @click="uploadBackup">上传 backup.zip</button>
        <button :disabled="busy" @click="downloadBackup">下载最新备份</button>
      </div>
    </section>

    <section class="card">
      <h2>阅读进度同步</h2>
      <p class="hint">进度文件位于 bookProgress/，以“书名_作者”命名；远端更新且进度更靠后才覆盖本地。</p>
      <button :disabled="busy" @click="syncProgress">同步全部书籍进度</button>
    </section>

    <pre v-if="log" class="log">{{ log }}</pre>
  </div>
</template>

<style scoped>
.webdav-view {
  height: 100%;
  overflow-y: auto;
  background: var(--ac-background);
  padding: 12px 16px 28px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
}
.toolbar h1 {
  margin: 0 auto 0 0;
  font-size: 18px;
}
.card {
  border: 1px solid #e2dccf;
  background: var(--lr-color-surface);
  border-radius: 10px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
h2 {
  margin: 0;
  font-size: 15px;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}
input {
  padding: 8px 10px;
  border: 1px solid #ccc;
  border-radius: 8px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
.hint {
  margin: 0;
  font-size: 12px;
  color: #8a8175;
}
.log {
  background: #1f2933;
  color: #dbe4ee;
  border-radius: 10px;
  padding: 12px;
  font-size: 12px;
  white-space: pre-wrap;
  max-height: 260px;
  overflow-y: auto;
}
button {
  border: 1px solid #d8d0c0;
  background: #fffdf8;
  border-radius: 8px;
  padding: 7px 12px;
  cursor: pointer;
  font-size: 13px;
}
button:disabled {
  opacity: 0.5;
}
</style>
