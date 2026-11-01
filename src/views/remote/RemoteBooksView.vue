<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import {
  addRemoteBooks,
  createRemoteFolder,
  deleteRemoteEntries,
  listRemoteEntries,
  moveRemoteEntry,
  refreshRemoteEntry,
  renameRemoteEntry,
  testRemoteServer,
  uploadBook,
} from "@/core/application/remoteBooks";
import { bookList } from "@/core/application/books";
import {
  deleteRemoteServer,
  getRemoteProxyUrl,
  getSelectedRemoteServerId,
  listRemoteServers,
  saveRemoteServer,
  selectRemoteServer,
  setRemoteProxyUrl,
} from "@/core/application/remoteServers";
import type {
  RemoteEntry,
  RemoteServer,
  RemoteServerType,
  S3ServerConfig,
  WebDavServerConfig,
} from "@/core/domain";

const router = useRouter();
const servers = ref<RemoteServer[]>([]);
const selectedServerId = ref(-1);
const entries = ref<RemoteEntry[]>([]);
const stack = ref<{ path: string; name: string }[]>([]);
const selected = ref(new Set<string>());
const query = ref("");
const loading = ref(false);
const message = ref("");
const proxyUrl = ref("");
const sortKey = ref<"name" | "time">("name");
const sortAsc = ref(true);
const operationController = ref<AbortController | null>(null);
const operationLabel = ref("");
const operationDone = ref(0);
const operationTotal = ref(0);
const BOOK_EXTENSIONS = new Set([
  "txt",
  "epub",
  "pdf",
  "umd",
  "mobi",
  "azw",
  "azw3",
  "zip",
]);

function isBookName(name: string): boolean {
  const index = name.lastIndexOf(".");
  return index >= 0 && BOOK_EXTENSIONS.has(name.slice(index + 1).toLowerCase());
}

const serverDialogVisible = ref(false);
const proxyDialogVisible = ref(false);
const uploadDialogVisible = ref(false);
const uploadBookUrl = ref("");
const localBooks = ref<{ bookUrl: string; name: string }[]>([]);
const uploading = ref(false);
const editingServer = ref<RemoteServer | null>(null);
const formType = ref<RemoteServerType>("webdav");
const formName = ref("");
const formUrl = ref("");
const formUsername = ref("");
const formPassword = ref("");
const formEndpoint = ref("");
const formRegion = ref("us-east-1");
const formBucket = ref("");
const formAccessKey = ref("");
const formSecretKey = ref("");
const formSessionToken = ref("");
const formPrefix = ref("");
const formPathStyle = ref(false);
const testing = ref(false);
const folderDialogVisible = ref(false);
const folderName = ref("");
const folderBusy = ref(false);
const entryActionVisible = ref(false);
const actionEntry = ref<RemoteEntry | null>(null);
const renameDialogVisible = ref(false);
const renameName = ref("");
const moveDialogVisible = ref(false);
const moveTarget = ref("");
const actionBusy = ref(false);

const currentServer = computed(
  () => servers.value.find((server) => server.id === selectedServerId.value) ?? null,
);
const currentPath = computed(() => stack.value[stack.value.length - 1]?.path ?? "");
const pathLabel = computed(() =>
  stack.value.length
    ? `${stack.value.map((item) => item.name).join("/")}/`
    : "/",
);
const filteredEntries = computed(() => {
  const keyword = query.value.trim().toLowerCase();
  return entries.value
    .filter((entry) => entry.isDir || isBookName(entry.name))
    .filter((entry) => !keyword || entry.name.toLowerCase().includes(keyword))
    .sort((left, right) => {
      if (left.isDir !== right.isDir) return left.isDir ? -1 : 1;
      const compared =
        sortKey.value === "time"
          ? left.modified - right.modified ||
            left.name.localeCompare(right.name, "zh-CN", { numeric: true })
          : left.name.localeCompare(right.name, "zh-CN", { numeric: true });
      return sortAsc.value ? compared : -compared;
    });
});
const selectable = computed(() =>
  filteredEntries.value.filter((entry) => !entry.isDir && !entry.isOnShelf),
);
const allChecked = computed(
  () => selectable.value.length > 0 && selectable.value.every((entry) => selected.value.has(entry.path)),
);

function showMessage(text: string) {
  message.value = text;
  window.setTimeout(() => {
    if (message.value === text) message.value = "";
  }, 3000);
}

function formatSize(size: number): string {
  if (!size) return "";
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${(size / 1024).toFixed(1)} KB`;
  if (size < 1024 * 1024 * 1024) return `${(size / 1024 / 1024).toFixed(1)} MB`;
  return `${(size / 1024 / 1024 / 1024).toFixed(1)} GB`;
}

function formatDate(seconds: number): string {
  if (!seconds) return "";
  return new Date(seconds * 1000).toLocaleDateString("zh-CN");
}

function isAbort(error: unknown): boolean {
  return (
    (error instanceof DOMException && error.name === "AbortError") ||
    String(error).includes("AbortError")
  );
}

function beginOperation(label: string, total: number): AbortController {
  operationController.value?.abort();
  const controller = new AbortController();
  operationController.value = controller;
  operationLabel.value = label;
  operationDone.value = 0;
  operationTotal.value = total;
  return controller;
}

function endOperation(controller: AbortController) {
  if (operationController.value !== controller) return;
  operationController.value = null;
  operationLabel.value = "";
  operationDone.value = 0;
  operationTotal.value = 0;
}

function cancelOperation() {
  operationController.value?.abort();
}

function openCreateFolder() {
  if (!currentServer.value) return;
  folderName.value = "";
  folderDialogVisible.value = true;
}

async function doCreateFolder() {
  const server = currentServer.value;
  const name = folderName.value.trim();
  if (!server || !name) return;
  folderBusy.value = true;
  try {
    await createRemoteFolder(server, currentPath.value, name);
    folderDialogVisible.value = false;
    showMessage("文件夹已创建");
    await load(currentPath.value);
  } catch (error) {
    showMessage(`创建失败：${error}`);
  } finally {
    folderBusy.value = false;
  }
}

function openEntryActions(entry: RemoteEntry) {
  actionEntry.value = entry;
  entryActionVisible.value = true;
}

function startRename() {
  if (!actionEntry.value) return;
  renameName.value = actionEntry.value.name;
  entryActionVisible.value = false;
  renameDialogVisible.value = true;
}

function startMove() {
  if (!actionEntry.value) return;
  moveTarget.value = currentPath.value;
  entryActionVisible.value = false;
  moveDialogVisible.value = true;
}

async function doRename() {
  const server = currentServer.value;
  const entry = actionEntry.value;
  const name = renameName.value.trim();
  if (!server || !entry || !name || name === entry.name) return;
  actionBusy.value = true;
  try {
    await renameRemoteEntry(server, entry, name);
    renameDialogVisible.value = false;
    showMessage(`已重命名为 ${name}`);
    await load(currentPath.value);
  } catch (error) {
    showMessage(`重命名失败：${error}`);
  } finally {
    actionBusy.value = false;
  }
}

async function doMove() {
  const server = currentServer.value;
  const entry = actionEntry.value;
  if (!server || !entry) return;
  actionBusy.value = true;
  try {
    await moveRemoteEntry(server, entry, moveTarget.value.trim());
    moveDialogVisible.value = false;
    showMessage(`已移动 ${entry.name}`);
    await load(currentPath.value);
  } catch (error) {
    showMessage(`移动失败：${error}`);
  } finally {
    actionBusy.value = false;
  }
}

async function loadServers() {
  servers.value = await listRemoteServers();
  const saved = await getSelectedRemoteServerId();
  selectedServerId.value = servers.value.some((server) => server.id === saved)
    ? saved
    : servers.value[0]?.id ?? -1;
  if (!servers.value.some((server) => server.id === selectedServerId.value)) {
    selectedServerId.value = servers.value[0]?.id ?? -1;
  }
}

async function load(path?: string) {
  const server = currentServer.value;
  if (!server) {
    entries.value = [];
    return;
  }
  loading.value = true;
  selected.value = new Set();
  try {
    entries.value = await listRemoteEntries(server, path || null);
  } catch (error) {
    entries.value = [];
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

async function onServerChange() {
  await selectRemoteServer(selectedServerId.value);
  stack.value = [];
  await load();
}

async function onRefresh() {
  await load(currentPath.value);
}

async function openEntry(entry: RemoteEntry) {
  if (entry.isDir) {
    stack.value = [...stack.value, { path: entry.path, name: entry.name }];
    await load(entry.path);
    return;
  }
  if (entry.isOnShelf) {
    if (entry.bookUrl) {
      router.push({ path: "/reader", query: { bookUrl: entry.bookUrl } });
    } else {
      showMessage("书籍已在书架");
    }
    return;
  }
  const next = new Set(selected.value);
  if (next.has(entry.path)) next.delete(entry.path);
  else next.add(entry.path);
  selected.value = next;
}

async function goUp() {
  if (stack.value.length <= 1) {
    stack.value = [];
    await load();
    return;
  }
  const next = stack.value.slice(0, -1);
  stack.value = next;
  await load(next[next.length - 1].path);
}

function toggleAll() {
  selected.value = allChecked.value
    ? new Set()
    : new Set(selectable.value.map((entry) => entry.path));
}

async function addSelected() {
  const server = currentServer.value;
  if (!server) return;
  const chosen = filteredEntries.value.filter((entry) => selected.value.has(entry.path));
  if (!chosen.length) return;
  loading.value = true;
  const controller = beginOperation("正在导入", chosen.length);
  try {
    const results = await addRemoteBooks(
      server,
      chosen,
      0,
      (name, index, total) => {
        operationDone.value = index;
        operationTotal.value = total;
        showMessage(`正在导入 ${index}/${total}：${name}`);
      },
      controller.signal,
    );
    const failed = results.filter((item) => item.error);
    showMessage(
      failed.length
        ? `完成，失败 ${failed.length} 个：${failed[0]?.error}`
        : `已加入书架 ${results.length} 本`,
    );
    await load(currentPath.value);
  } catch (error) {
    showMessage(isAbort(error) ? "已取消导入" : String(error));
  } finally {
    endOperation(controller);
    loading.value = false;
  }
}

async function deleteSelected() {
  const server = currentServer.value;
  if (!server) return;
  const chosen = filteredEntries.value.filter((entry) => selected.value.has(entry.path));
  if (!chosen.length || !confirm(`删除远端 ${chosen.length} 个文件？`)) return;
  loading.value = true;
  try {
    const results = await deleteRemoteEntries(server, chosen);
    const failed = results.filter((item) => item.error);
    showMessage(
      failed.length
        ? `已删除 ${results.length - failed.length} 个，失败 ${failed.length} 个：${failed[0]?.error}`
        : `已删除远端 ${results.length} 个文件`,
    );
    await load(currentPath.value);
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

async function updateEntry(entry: RemoteEntry) {
  const server = currentServer.value;
  if (!server || !entry.isOnShelf) return;
  loading.value = true;
  try {
    const result = await refreshRemoteEntry(server, entry);
    showMessage(result.error ? `更新失败：${result.error}` : `已更新《${entry.name}》`);
    await load(currentPath.value);
  } catch (error) {
    showMessage(String(error));
  } finally {
    loading.value = false;
  }
}

async function openUpload() {
  if (!currentServer.value) return;
  localBooks.value = (await bookList()).map((book) => ({ bookUrl: book.bookUrl, name: book.name }));
  uploadBookUrl.value = localBooks.value[0]?.bookUrl ?? "";
  uploadDialogVisible.value = true;
}

async function doUpload() {
  const server = currentServer.value;
  if (!server || !uploadBookUrl.value) return;
  uploading.value = true;
  const controller = beginOperation("正在上传", 1);
  try {
    await uploadBook(server, uploadBookUrl.value, currentPath.value, controller.signal);
    operationDone.value = 1;
    uploadDialogVisible.value = false;
    showMessage("上传完成");
    await load(currentPath.value);
  } catch (error) {
    showMessage(isAbort(error) ? "已取消上传" : `上传失败：${error}`);
  } finally {
    endOperation(controller);
    uploading.value = false;
  }
}

function openServerDialog(server?: RemoteServer) {
  editingServer.value = server ?? null;
  if (server) {
    formType.value = server.type;
    formName.value = server.name;
    if ("url" in server.config) {
      formUrl.value = server.config.url;
      formUsername.value = server.config.username;
      formPassword.value = server.config.password;
    } else {
      formEndpoint.value = server.config.endpoint;
      formRegion.value = server.config.region;
      formBucket.value = server.config.bucket;
      formAccessKey.value = server.config.accessKeyId;
      formSecretKey.value = server.config.secretAccessKey;
      formSessionToken.value = server.config.sessionToken ?? "";
      formPrefix.value = server.config.prefix ?? "";
      formPathStyle.value = server.config.pathStyle ?? false;
    }
  } else {
    formType.value = "webdav";
    formName.value = "";
    formUrl.value = "";
    formUsername.value = "";
    formPassword.value = "";
    formEndpoint.value = "";
    formRegion.value = "us-east-1";
    formBucket.value = "";
    formAccessKey.value = "";
    formSecretKey.value = "";
    formSessionToken.value = "";
    formPrefix.value = "";
    formPathStyle.value = false;
  }
  serverDialogVisible.value = true;
}

function buildServer(): RemoteServer {
  const id = editingServer.value?.id ?? Date.now();
  const config: WebDavServerConfig | S3ServerConfig =
    formType.value === "s3"
      ? {
          endpoint: formEndpoint.value,
          region: formRegion.value,
          bucket: formBucket.value,
          accessKeyId: formAccessKey.value,
          secretAccessKey: formSecretKey.value,
          sessionToken: formSessionToken.value.trim() || undefined,
          prefix: formPrefix.value,
          pathStyle: formPathStyle.value,
        }
      : {
          url: formUrl.value,
          username: formUsername.value,
          password: formPassword.value,
        };
  return {
    id,
    name: formName.value.trim() || (formType.value === "s3" ? "S3" : "WebDAV"),
    type: formType.value,
    config,
    sortNumber: editingServer.value?.sortNumber ?? Date.now(),
  };
}

async function saveServer() {
  try {
    const saved = await saveRemoteServer(buildServer());
    serverDialogVisible.value = false;
    await selectRemoteServer(saved.id);
    await loadServers();
    selectedServerId.value = saved.id;
    stack.value = [];
    await load();
  } catch (error) {
    showMessage(String(error));
  }
}

async function removeServer() {
  const server = editingServer.value;
  if (!server || server.id === -1) return;
  if (!confirm(`删除服务器「${server.name}」？`)) return;
  await deleteRemoteServer(server.id);
  serverDialogVisible.value = false;
  await loadServers();
  stack.value = [];
  await load();
}

async function testServer() {
  testing.value = true;
  try {
    await testRemoteServer(buildServer());
    showMessage("连接成功");
  } catch (error) {
    showMessage(`连接失败：${error}`);
  } finally {
    testing.value = false;
  }
}

async function saveProxy() {
  await setRemoteProxyUrl(proxyUrl.value);
  proxyDialogVisible.value = false;
  showMessage("代理配置已保存");
}

onMounted(async () => {
  proxyUrl.value = await getRemoteProxyUrl();
  await loadServers();
  await load();
});
</script>

<template>
  <div class="remote-page">
    <TitleBar title="远程书籍" @back="router.back()">
      <template #actions>
        <button aria-label="新增服务器" @click="openServerDialog()">
          <Icon name="ic_add" :size="24" />
        </button>
        <button aria-label="代理设置" @click="proxyDialogVisible = true">
          <Icon name="ic_cfg_other" :size="24" />
        </button>
        <button aria-label="新建文件夹" :disabled="!currentServer" @click="openCreateFolder">
          <Icon name="ic_create_folder_outline" :size="24" />
        </button>
        <button aria-label="上传本地书籍" @click="openUpload">
          <Icon name="ic_add_online" :size="24" />
        </button>
        <button aria-label="刷新" @click="onRefresh">
          <Icon name="ic_refresh_black_24dp" :size="24" />
        </button>
      </template>
    </TitleBar>

    <div class="toolbar">
      <select v-model.number="selectedServerId" @change="onServerChange">
        <option v-for="server in servers" :key="server.id" :value="server.id">
          {{ server.name }} · {{ server.type === "s3" ? "S3" : "WebDAV" }}
        </option>
      </select>
      <button :disabled="selectedServerId === -1" @click="openServerDialog(currentServer ?? undefined)">
        编辑
      </button>
    </div>
    <div class="toolbar sort-bar">
      <select v-model="sortKey">
        <option value="name">按名称</option>
        <option value="time">按时间</option>
      </select>
      <button @click="sortAsc = !sortAsc">{{ sortAsc ? "升序" : "降序" }}</button>
    </div>

    <div class="path-bar">
      <span>{{ pathLabel }}</span>
      <button :disabled="!stack.length" @click="goUp">上级</button>
    </div>

    <div class="search">
      <input v-model="query" placeholder="搜索远程文件" />
    </div>

    <div v-if="operationController" class="operation">
      <span>
        {{ operationLabel }}
        <template v-if="operationTotal">{{ operationDone }}/{{ operationTotal }}</template>
      </span>
      <button class="danger" @click="cancelOperation">取消</button>
    </div>

    <p v-if="message" class="message">{{ message }}</p>
    <p v-if="loading" class="loading">加载中…</p>
    <p v-else-if="!servers.length" class="empty">
      还没有 WebDAV 或 S3 服务器，请点击右上角添加。
    </p>
    <p v-else-if="!filteredEntries.length" class="empty">当前目录为空。</p>

    <div v-else class="entries">
      <div
        v-for="entry in filteredEntries"
        :key="entry.path"
        class="entry"
        role="button"
        tabindex="0"
        @click="openEntry(entry)"
        @keydown.enter="openEntry(entry)"
      >
        <span class="icon">
          <Icon
            :name="entry.isDir ? 'ic_folder' : entry.isOnShelf ? 'ic_book_has' : 'ic_bottom_books'"
            :size="24"
          />
        </span>
        <span class="info">
          <strong>{{ entry.name }}</strong>
          <small>
            {{ entry.isDir ? "目录" : formatSize(entry.size) }}
            {{ entry.modified ? formatDate(entry.modified) : "" }}
            {{ entry.isOnShelf ? " · 已在书架" : "" }}
          </small>
        </span>
        <button
          v-if="entry.isOnShelf"
          class="mini"
          @click.stop="updateEntry(entry)"
        >
          更新
        </button>
        <span v-else-if="!entry.isDir" class="check">
          {{ selected.has(entry.path) ? "✓" : "" }}
        </span>
        <button
          class="mini icon-mini"
          aria-label="更多操作"
          @click.stop="openEntryActions(entry)"
        >
          <Icon name="ic_more_vert" :size="18" />
        </button>
      </div>
    </div>

    <footer v-if="selectable.length" class="select-bar">
      <button @click="toggleAll">{{ allChecked ? "取消全选" : "全选" }}</button>
      <span>已选 {{ selected.size }}/{{ selectable.length }}</span>
      <button class="danger" :disabled="!selected.size || loading" @click="deleteSelected">
        删除远端
      </button>
      <button class="accent" :disabled="!selected.size || loading" @click="addSelected">
        加入书架
      </button>
    </footer>

    <AndroidDialog
      v-model="serverDialogVisible"
      :title="editingServer ? '编辑服务器' : '新增服务器'"
      mode="center"
    >
      <div class="form">
        <label>类型
          <select v-model="formType">
            <option value="webdav">WebDAV</option>
            <option value="s3">S3</option>
          </select>
        </label>
        <label>名称<input v-model="formName" /></label>
        <template v-if="formType === 'webdav'">
          <label>服务器地址<input v-model="formUrl" placeholder="https://dav.example.com/dav/" /></label>
          <label>账号<input v-model="formUsername" /></label>
          <label>密码<input v-model="formPassword" type="password" /></label>
        </template>
        <template v-else>
          <label>Endpoint<input v-model="formEndpoint" placeholder="https://s3.us-east-1.amazonaws.com" /></label>
          <label>Region<input v-model="formRegion" /></label>
          <label>Bucket<input v-model="formBucket" /></label>
          <label>Prefix<input v-model="formPrefix" placeholder="可选，例如 books/" /></label>
          <label>Access Key<input v-model="formAccessKey" /></label>
          <label>Secret Key<input v-model="formSecretKey" type="password" /></label>
          <label>Session Token（可选）<input v-model="formSessionToken" type="password" /></label>
          <label class="checkbox"><input v-model="formPathStyle" type="checkbox" /> path-style endpoint</label>
        </template>
        <div class="form-actions">
          <button :disabled="testing" @click="testServer">测试连接</button>
          <button v-if="editingServer && editingServer.id !== -1" class="danger" @click="removeServer">删除</button>
        </div>
      </div>
      <template #footer>
        <button @click="serverDialogVisible = false">取消</button>
        <button class="accent" @click="saveServer">保存</button>
      </template>
    </AndroidDialog>

    <AndroidDialog v-model="proxyDialogVisible" title="WebDAV/S3 代理" mode="center">
      <div class="form">
        <p class="hint">
          Web 端直连失败时使用。Cloudflare Pages 同域可填 /api/remote。
        </p>
        <label>代理地址<input v-model="proxyUrl" placeholder="https://your-worker.example.com/api/remote" /></label>
      </div>
      <template #footer>
        <button @click="proxyDialogVisible = false">取消</button>
        <button class="accent" @click="saveProxy">保存</button>
      </template>
    </AndroidDialog>

    <AndroidDialog v-model="uploadDialogVisible" title="上传本地书籍" mode="center">
      <div class="form">
        <p class="hint">上传到当前目录：{{ pathLabel }}</p>
        <label>选择书籍
          <select v-model="uploadBookUrl">
            <option value="" disabled>请选择书籍</option>
            <option v-for="book in localBooks" :key="book.bookUrl" :value="book.bookUrl">
              {{ book.name }}
            </option>
          </select>
        </label>
      </div>
      <template #footer>
        <button @click="uploadDialogVisible = false">取消</button>
        <button class="accent" :disabled="uploading || !uploadBookUrl" @click="doUpload">
          {{ uploading ? "上传中…" : "上传" }}
        </button>
      </template>
    </AndroidDialog>

    <AndroidDialog v-model="folderDialogVisible" title="新建文件夹" mode="center">
      <div class="form">
        <p class="hint">在当前目录创建：{{ pathLabel }}</p>
        <label>名称<input v-model="folderName" /></label>
      </div>
      <template #footer>
        <button @click="folderDialogVisible = false">取消</button>
        <button class="accent" :disabled="folderBusy || !folderName.trim()" @click="doCreateFolder">
          {{ folderBusy ? "创建中…" : "创建" }}
        </button>
      </template>
    </AndroidDialog>

    <AndroidDialog v-model="entryActionVisible" :title="actionEntry?.name ?? '文件操作'" mode="center">
      <div class="form">
        <button @click="startRename">重命名</button>
        <button @click="startMove">移动到目录</button>
      </div>
      <template #footer>
        <button @click="entryActionVisible = false">取消</button>
      </template>
    </AndroidDialog>

    <AndroidDialog v-model="renameDialogVisible" title="重命名" mode="center">
      <div class="form">
        <label>新名称<input v-model="renameName" /></label>
      </div>
      <template #footer>
        <button @click="renameDialogVisible = false">取消</button>
        <button class="accent" :disabled="actionBusy || !renameName.trim()" @click="doRename">
          {{ actionBusy ? "处理中…" : "确定" }}
        </button>
      </template>
    </AndroidDialog>

    <AndroidDialog v-model="moveDialogVisible" title="移动到目录" mode="center">
      <div class="form">
        <p class="hint">目标目录使用完整远程地址，例如 s3://bucket/books/ 或 https://dav.example.com/dav/books/</p>
        <label>目标目录<input v-model="moveTarget" :placeholder="currentPath || '/' " /></label>
      </div>
      <template #footer>
        <button @click="moveDialogVisible = false">取消</button>
        <button class="accent" :disabled="actionBusy" @click="doMove">
          {{ actionBusy ? "处理中…" : "移动" }}
        </button>
      </template>
    </AndroidDialog>
  </div>
</template>

<style scoped>
.remote-page {
  min-height: 100%;
  background: var(--ac-background);
  padding-bottom: 72px;
}
.toolbar,
.path-bar {
  display: flex;
  gap: 8px;
  align-items: center;
  padding: 8px 16px;
}
.toolbar select {
  flex: 1;
  min-width: 0;
}
.path-bar {
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.path-bar span {
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 6px 10px;
}
button.accent {
  border-color: var(--ac-primary);
  color: var(--ac-primary);
}
button.danger {
  color: var(--ac-error);
}
button:disabled {
  opacity: 0.5;
}
.sort-bar {
  padding-top: 0;
}
.sort-bar select {
  flex: 0 1 130px;
}
.operation {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 16px 8px;
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  border-radius: 4px;
  color: var(--ac-secondary-text);
  font-size: 13px;
}
.operation span {
  flex: 1;
  min-width: 0;
}
.icon-mini {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 4px;
}
.search {
  padding: 0 16px 8px;
}
.search input,
.form input,
.form select {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid var(--ac-divider);
  border-radius: 4px;
  background: var(--ac-background-menu);
  color: var(--ac-primary-text);
  padding: 8px;
}
.entries {
  display: flex;
  flex-direction: column;
}
.entry {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  border-radius: 0;
  padding: 10px 16px;
  text-align: left;
}
.entry .info {
  flex: 1;
  min-width: 0;
}
.entry strong,
.entry small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.entry small {
  color: var(--ac-secondary-text);
  margin-top: 2px;
}
.mini {
  border: 1px solid var(--ac-primary);
  color: var(--ac-primary);
  padding: 4px 8px;
  font-size: 12px;
}
.check {
  width: 22px;
  text-align: center;
  color: var(--ac-primary);
}
.select-bar {
  position: fixed;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 16px calc(8px + env(safe-area-inset-bottom));
  background: var(--ac-background-menu);
  border-top: 1px solid var(--ac-divider);
}
.select-bar span {
  flex: 1;
  text-align: center;
  color: var(--ac-secondary-text);
}
.message,
.loading,
.empty {
  margin: 16px;
  color: var(--ac-secondary-text);
  text-align: center;
}
.form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.form label.checkbox {
  flex-direction: row;
  align-items: center;
}
.form label.checkbox input {
  width: auto;
}
.form-actions {
  display: flex;
  gap: 8px;
}
.hint {
  margin: 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
  line-height: 1.5;
}
</style>
