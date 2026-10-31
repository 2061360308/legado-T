<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import { bookList, bookUpsert } from "@/core/application/books";
import { backupExport, backupImport } from "@/core/application/backup";
import { kvGet, kvSet } from "@/core/application/config";
import { applySettingsSnapshot, snapshotSettings } from "@/stores/settingsSnapshot";
import type { Book } from "@/core/domain";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";

const router = useRouter();
const webdavUrl = ref("");
const account = ref("");
const password = ref("");
const subDir = ref("legado");
const deviceName = ref("");
const backupPassword = ref("");
const syncProgress = ref(true);
const syncProgressPlus = ref(false);
const onlyLatestBackup = ref(true);
const autoCheckNewBackup = ref(true);
const message = ref("");
const log = ref("");
const busy = ref(false);
const menuVisible = ref(false);
const ignore = ref<string[]>([]);

const menuItems: PopupMenuItem[] = [
  { key: "help", label: "帮助" },
  { key: "log", label: "日志" },
];

const ignoreOptions = [
  { value: "localBooks", label: "本地书籍" },
  { value: "bookmarks", label: "书签" },
  { value: "readRecord", label: "阅读记录" },
  { value: "themes", label: "主题配置" },
  { value: "config", label: "阅读配置" },
];

const rootUrl = computed(() => {
  let url = webdavUrl.value.trim() || "https://dav.jianguoyun.com/dav/";
  if (!url.endsWith("/")) url += "/";
  const dir = subDir.value.trim().replace(/^\/+|\/+$/g, "");
  return dir ? `${url}${dir}/` : url;
});

function append(text: string) {
  log.value = `${new Date().toLocaleTimeString()} ${text}\n${log.value}`;
}

function authHeaders(): Record<string, string> {
  if (!account.value && !password.value) return {};
  const token = btoa(unescape(encodeURIComponent(`${account.value}:${password.value}`)));
  return { Authorization: `Basic ${token}` };
}

function fileUrl(path: string): string {
  return `${rootUrl.value}${path.replace(/^\/+/, "")}`;
}

async function loadConfig() {
  webdavUrl.value = (await kvGet("webdavUrl")) ?? "";
  account.value = (await kvGet("webdavAccount")) ?? (await kvGet("webdavUser")) ?? "";
  password.value = (await kvGet("webdavPassword")) ?? "";
  subDir.value = (await kvGet("webdavDir")) ?? "legado";
  deviceName.value = (await kvGet("webdavDeviceName")) ?? "";
  syncProgress.value = (await kvGet("syncBookProgress")) !== "false";
  syncProgressPlus.value = (await kvGet("syncBookProgressPlus")) === "true";
  onlyLatestBackup.value = (await kvGet("onlyLatestBackup")) !== "false";
  autoCheckNewBackup.value = (await kvGet("autoCheckNewBackup")) !== "false";
}

async function saveConfig() {
  await kvSet("webdavUrl", webdavUrl.value);
  await kvSet("webdavAccount", account.value);
  await kvSet("webdavPassword", password.value);
  await kvSet("webdavDir", subDir.value);
  await kvSet("webdavDeviceName", deviceName.value);
  await kvSet("syncBookProgress", String(syncProgress.value));
  await kvSet("syncBookProgressPlus", String(syncProgressPlus.value));
  await kvSet("onlyLatestBackup", String(onlyLatestBackup.value));
  await kvSet("autoCheckNewBackup", String(autoCheckNewBackup.value));
  append("WebDAV 配置已保存");
}

async function propfind(path = ""): Promise<string> {
  const response = await platform.http.fetch(fileUrl(path), {
    method: "PROPFIND",
    headers: { ...authHeaders(), Depth: "1", "Content-Type": "application/xml" },
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
      const response = await platform.http.fetch(fileUrl(dir), { method: "MKCOL", headers: authHeaders() });
      append(`${dir} ${response.ok || response.status === 405 || response.status === 409 ? "就绪" : `失败 ${response.status}`}`);
    }
  } catch (e) {
    append(`初始化失败：${e}`);
  } finally {
    busy.value = false;
  }
}

function backupFileName(): string {
  if (onlyLatestBackup.value) return "backup.zip";
  const date = new Date().toISOString().slice(0, 10);
  const name = deviceName.value.trim();
  return `backup${date}${name ? `-${name}` : ""}.zip`;
}

async function exportBackup() {
  const path = await platform.dialog.save({
    defaultPath: backupFileName(),
    filters: [{ name: "ZIP 备份", extensions: ["zip"] }],
  });
  if (!path) return;
  busy.value = true;
  try {
    const report = await backupExport(path, backupPassword.value || null, snapshotSettings());
    message.value = `备份完成：${report.files} 个文件，${report.items} 条记录`;
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function importBackup() {
  const path = await platform.dialog.open({ multiple: false, filters: [{ name: "ZIP 备份", extensions: ["zip"] }] });
  if (typeof path !== "string") return;
  if (!confirm("恢复将按主键合并覆盖本地数据，是否继续？")) return;
  busy.value = true;
  try {
    const report = await backupImport(path, backupPassword.value || null, ignore.value);
    const applied = applySettingsSnapshot(report.settingsJson);
    message.value = `恢复完成：${report.files} 个文件，${report.items} 条记录${applied ? "，设置将在重启后生效" : ""}`;
    if (applied) setTimeout(() => window.location.reload(), 800);
  } catch (e) {
    message.value = String(e);
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
    const name = backupFileName();
    const response = await platform.http.fetch(fileUrl(name), {
      method: "PUT",
      headers: { ...authHeaders(), "Content-Type": "application/zip" },
      body: bytes,
    });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    append(`已上传 ${name}`);
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
    if (!matches.length) throw new Error("没有找到备份文件");
    const remote = matches.sort().at(-1)!;
    const response = await platform.http.fetch(new URL(remote, rootUrl.value).toString(), { headers: authHeaders() });
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    const bytes = new Uint8Array(await response.arrayBuffer());
    const path = await platform.dialog.save({
      defaultPath: decodeURIComponent(remote.split("/").pop() ?? "backup.zip"),
      filters: [{ name: "ZIP 备份", extensions: ["zip"] }],
    });
    if (!path) return;
    await platform.files.writeFile(path, bytes);
    append(`已下载 ${remote}`);
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

async function syncAllProgress() {
  busy.value = true;
  let updated = 0;
  let uploaded = 0;
  try {
    for (const book of await bookList()) {
      if (!book.durChapterTitle) continue;
      const url = fileUrl(progressFileName(book));
      const response = await platform.http.fetch(url, { headers: authHeaders() });
      if (response.ok) {
        const remote = (await response.json()) as {
          durChapterIndex?: number;
          durChapterPos?: number;
          durChapterTitle?: string;
          durChapterTime?: number;
        };
        const newer =
          (remote.durChapterTime ?? 0) > (book.durChapterTime ?? 0) &&
          ((remote.durChapterIndex ?? 0) > (book.durChapterIndex ?? 0) ||
            ((remote.durChapterIndex ?? 0) === (book.durChapterIndex ?? 0) &&
              (remote.durChapterPos ?? 0) > (book.durChapterPos ?? 0)));
        if (newer) {
          await bookUpsert({
            ...book,
            durChapterIndex: remote.durChapterIndex,
            durChapterPos: remote.durChapterPos,
            durChapterTitle: remote.durChapterTitle ?? book.durChapterTitle,
            durChapterTime: remote.durChapterTime,
          });
          updated += 1;
          continue;
        }
      }
      const payload = JSON.stringify({
        durChapterIndex: book.durChapterIndex ?? 0,
        durChapterPos: book.durChapterPos ?? 0,
        durChapterTitle: book.durChapterTitle ?? "",
        durChapterTime: Math.floor(Date.now() / 1000),
      });
      const put = await platform.http.fetch(url, {
        method: "PUT",
        headers: { ...authHeaders(), "Content-Type": "application/json" },
        body: payload,
      });
      if (put.ok) uploaded += 1;
    }
    append(`进度同步完成：更新本地 ${updated} 本，上传 ${uploaded} 本`);
  } catch (e) {
    append(`进度同步失败：${e}`);
  } finally {
    busy.value = false;
  }
}

function toggleIgnore(value: string) {
  const set = new Set(ignore.value);
  if (set.has(value)) set.delete(value);
  else set.add(value);
  ignore.value = [...set];
}

function onMenu(key: string) {
  if (key === "help") message.value = "WebDAV 帮助暂未接入";
  else if (key === "log") message.value = log.value || "暂无日志";
}

onMounted(loadConfig);
</script>

<template>
  <div class="backup-page">
    <TitleBar title="备份与恢复" @back="router.back()">
      <template #actions>
        <button aria-label="更多" @click="menuVisible = true">
          <Icon name="ic_more_vert" :size="24" />
        </button>
      </template>
    </TitleBar>
    <PopupMenu v-model="menuVisible" :items="menuItems" :top="54" @select="onMenu" />

    <main class="content">
      <section class="pref-group">
        <h2>WebDAV 设置</h2>
        <label class="pref-field">
          <span>WebDAV URL</span>
          <input v-model="webdavUrl" placeholder="https://dav.jianguoyun.com/dav/" />
        </label>
        <label class="pref-field">
          <span>账号</span>
          <input v-model="account" />
        </label>
        <label class="pref-field">
          <span>密码</span>
          <input v-model="password" type="password" />
        </label>
        <label class="pref-field">
          <span>子目录</span>
          <input v-model="subDir" placeholder="legado" />
        </label>
        <label class="pref-field">
          <span>设备名</span>
          <input v-model="deviceName" />
        </label>
        <label class="pref-row">
          <span class="pref-text"><span class="pref-label">同步书籍进度</span></span>
          <input v-model="syncProgress" type="checkbox" />
        </label>
        <label class="pref-row">
          <span class="pref-text"><span class="pref-label">同步进度增强</span></span>
          <input v-model="syncProgressPlus" type="checkbox" />
        </label>
        <div class="button-row">
          <button :disabled="busy" @click="saveConfig">保存配置</button>
          <button :disabled="busy" @click="testConnection">测试连接</button>
          <button :disabled="busy" @click="ensureDirs">初始化目录</button>
        </div>
      </section>

      <section class="pref-group">
        <h2>备份与恢复</h2>
        <label class="pref-field">
          <span>备份密码</span>
          <input v-model="backupPassword" type="password" placeholder="留空则不加密" />
        </label>
        <div class="button-row">
          <button :disabled="busy" @click="exportBackup">本地备份</button>
          <button :disabled="busy" @click="importBackup">本地恢复</button>
        </div>
        <div class="button-row">
          <button :disabled="busy" @click="uploadBackup">上传到 WebDAV</button>
          <button :disabled="busy" @click="downloadBackup">从 WebDAV 下载</button>
        </div>
        <button class="wide" :disabled="busy" @click="syncAllProgress">同步全部书籍进度</button>
      </section>

      <section class="pref-group">
        <h2>恢复忽略项</h2>
        <label v-for="opt in ignoreOptions" :key="opt.value" class="pref-row">
          <span class="pref-text"><span class="pref-label">忽略{{ opt.label }}</span></span>
          <input
            type="checkbox"
            :checked="ignore.includes(opt.value)"
            @change="toggleIgnore(opt.value)"
          />
        </label>
        <label class="pref-row">
          <span class="pref-text"><span class="pref-label">仅保留最新备份</span></span>
          <input v-model="onlyLatestBackup" type="checkbox" />
        </label>
        <label class="pref-row">
          <span class="pref-text"><span class="pref-label">自动检查新备份</span></span>
          <input v-model="autoCheckNewBackup" type="checkbox" />
        </label>
        <button class="wide" disabled>导入旧版本数据（暂未接入）</button>
      </section>

      <p v-if="message" class="message">{{ message }}</p>
      <pre v-if="log" class="log">{{ log }}</pre>
    </main>
  </div>
</template>

<style scoped>
.backup-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 24px;
}
.pref-group {
  margin-top: 8px;
}
.pref-group h2 {
  margin: 0;
  padding: 10px 20px 6px;
  color: var(--ac-primary);
  font-size: 13px;
  font-weight: 500;
}
.pref-field,
.pref-row {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: var(--ac-background);
  border-bottom: 1px solid var(--ac-divider);
  font-size: 14px;
}
.pref-field {
  flex-direction: column;
  align-items: stretch;
  gap: 5px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.pref-field input,
.pref-field textarea {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px 10px;
  font: inherit;
}
.pref-text {
  flex: 1;
  min-width: 0;
}
.pref-label {
  color: var(--ac-primary-text);
  font-size: 14px;
}
.button-row {
  display: flex;
  gap: 8px;
  padding: 8px 16px;
  flex-wrap: wrap;
}
button {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 4px;
  padding: 7px 12px;
  font-size: 13px;
}
button:disabled {
  opacity: 0.45;
}
button.wide {
  margin: 8px 16px;
  width: calc(100% - 32px);
}
.message {
  margin: 8px 16px;
  color: var(--ac-primary);
  font-size: 12px;
}
.log {
  margin: 8px 16px;
  max-height: 180px;
  overflow: auto;
  white-space: pre-wrap;
  color: var(--ac-secondary-text);
  font-size: 11px;
}
</style>
