<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import { open, save } from "@tauri-apps/plugin-dialog";
import { backupExport, backupImport } from "@/api/backup";
import TitleBar from "@/components/android/TitleBar.vue";

const router = useRouter();

const password = ref("");
const message = ref("");
const busy = ref(false);
const ignore = ref<string[]>([]);

const ignoreOptions = [
  { value: "localBooks", label: "本地书籍" },
  { value: "bookmarks", label: "书签" },
  { value: "readRecord", label: "阅读记录" },
  { value: "themes", label: "主题配置" },
  { value: "config", label: "阅读配置" },
];

function toggleIgnore(value: string) {
  const set = new Set(ignore.value);
  if (set.has(value)) set.delete(value);
  else set.add(value);
  ignore.value = [...set];
}

async function exportBackup() {
  const path = await save({
    defaultPath: "backup.zip",
    filters: [{ name: "ZIP 备份", extensions: ["zip"] }],
  });
  if (!path) return;
  busy.value = true;
  message.value = "";
  try {
    const report = await backupExport(path, password.value || null);
    message.value = `备份完成：${report.files} 个文件，${report.items} 条记录`;
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}

async function importBackup() {
  const path = await open({
    multiple: false,
    filters: [{ name: "ZIP 备份", extensions: ["zip"] }],
  });
  if (typeof path !== "string") return;
  if (!confirm("恢复将按主键合并覆盖本地数据，是否继续？")) return;
  busy.value = true;
  message.value = "";
  try {
    const report = await backupImport(path, password.value || null, ignore.value);
    message.value = `恢复完成：${report.files} 个文件，${report.items} 条记录`;
  } catch (e) {
    message.value = String(e);
  } finally {
    busy.value = false;
  }
}
</script>

<template>
  <div class="backup-view">
    <TitleBar title="备份与恢复" @back="router.back()" />

    <section class="card">
      <h2>备份密码</h2>
      <p class="hint">
        设置密码后，敏感配置项按 AES-128/ECB/PKCS5 加密写入备份；与 legado 使用相同密码时可互通。
      </p>
      <input v-model="password" type="password" placeholder="留空则不加密" />
    </section>

    <section class="card">
      <h2>备份</h2>
      <p class="hint">导出扁平 zip，包含书籍、分组、书签、规则、语音引擎、阅读记录、主题与配置；不包含章节正文与缓存。</p>
      <button :disabled="busy" @click="exportBackup">导出备份</button>
    </section>

    <section class="card">
      <h2>恢复</h2>
      <p class="hint">合并覆盖恢复，不清空本地已有数据；缺失文件按空表处理。</p>
      <div class="ignore">
        <label v-for="opt in ignoreOptions" :key="opt.value">
          <input
            type="checkbox"
            :checked="ignore.includes(opt.value)"
            @change="toggleIgnore(opt.value)"
          />
          忽略{{ opt.label }}
        </label>
      </div>
      <button :disabled="busy" @click="importBackup">选择备份并恢复</button>
    </section>

    <p v-if="message" class="message">{{ message }}</p>
  </div>
</template>

<style scoped>
.backup-view {
  height: 100%;
  overflow-y: auto;
  background: var(--ac-background);
  padding: 12px 16px 28px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.toolbar h1 {
  margin: 0;
  font-size: 18px;
}
.card {
  border: 1px solid #e2dccf;
  background: var(--lr-color-surface);
  border-radius: 10px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}
h2 {
  margin: 0;
  font-size: 15px;
}
.hint {
  margin: 0;
  font-size: 12px;
  color: #8a8175;
}
input[type="password"] {
  padding: 8px 10px;
  border: 1px solid #ccc;
  border-radius: 8px;
}
.ignore {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
}
.ignore label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
}
button {
  align-self: flex-start;
  border: 1px solid #d8d0c0;
  background: #fffdf8;
  border-radius: 8px;
  padding: 8px 14px;
  cursor: pointer;
  font-size: 13px;
}
button:disabled {
  opacity: 0.5;
}
.message {
  color: var(--lr-color-primary);
  font-size: 13px;
}
</style>
