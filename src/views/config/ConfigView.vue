<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import TitleBar from "@/components/android/TitleBar.vue";
import PreferenceRow from "@/components/android/PreferenceRow.vue";
import PreferenceCategory from "@/components/android/PreferenceCategory.vue";
import Icon from "@/components/android/Icon.vue";
import { useThemeStore } from "@/stores/theme";
import { renderMarkdown } from "@/utils/markdown";
import ThemeModeDialog from "./ThemeModeDialog.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import helpMarkdown from "@/assets/help/appHelp.md?raw";

const router = useRouter();
const theme = useThemeStore();
const themeModeVisible = ref(false);
const helpVisible = ref(false);

interface Item {
  label: string;
  summary: string;
  icon: string;
  path?: string;
  action?: () => void;
}

const groups = computed<{ title: string; items: Item[] }[]>(() => [
  {
    title: "",
    items: [
      { label: "书源管理", summary: "新建/编辑/删除书源，或批量导入", icon: "ic_cfg_source", path: "/book-source" },
      { label: "TXT 目录规则", summary: "编辑 TXT 目录规则", icon: "ic_cfg_source", path: "/toc-rules" },
      { label: "替换净化", summary: "净化文字内容", icon: "ic_cfg_replace", path: "/replace" },
      { label: "字典规则", summary: "编辑字典规则", icon: "ic_translate", path: "/dict-rules" },
      {
        label: "主题模式",
        summary: currentModeLabel.value,
        icon: "ic_cfg_theme",
        action: () => (themeModeVisible.value = true),
      },
    ],
  },
  {
    title: "设置",
    items: [
      { label: "备份与恢复", summary: "WebDav 备份/导入旧版本数据", icon: "ic_cfg_backup", path: "/backup" },
      { label: "主题设置", summary: "主题/颜色相关的一些设置", icon: "ic_cfg_theme", path: "/theme" },
      { label: "其他设置", summary: "与功能相关的一些设置", icon: "ic_cfg_other", path: "/other-settings" },
    ],
  },
  {
    title: "其他",
    items: [
      { label: "书签", summary: "全部书签", icon: "ic_bookmark", path: "/bookmarks" },
      { label: "阅读记录", summary: "阅读时间记录", icon: "ic_history", path: "/read-records" },
      { label: "音频播放", summary: "播放本地音频文件", icon: "ic_play_24dp", path: "/audio" },
      { label: "文件管理", summary: "管理私有文件夹的文件", icon: "ic_folder_outline", path: "/files" },
      { label: "关于", summary: "", icon: "ic_cfg_about", path: "/about" },
      { label: "退出", summary: "", icon: "ic_exit", action: () => platform.closeWindow() },
    ],
  },
]);

const currentModeLabel = computed(() => {
  const labels: Record<number, string> = {
    0: "跟随系统",
    1: "日间",
    2: "夜间",
    3: "墨水屏",
  };
  return labels[theme.mode] ?? "跟随系统";
});

const helpHtml = computed(() => renderMarkdown(helpMarkdown));

function open(item: Item) {
  if (item.action) item.action();
  else if (item.path) router.push(item.path);
}
</script>

<template>
  <div class="config-page">
    <TitleBar data-id="title_bar" title="我的" :back="false">
      <template #actions>
        <button aria-label="帮助" @click="helpVisible = true">
          <Icon name="ic_help" :size="24" />
        </button>
      </template>
    </TitleBar>
    <main class="pref-scroll" data-id="pre_fragment">
      <PreferenceCategory title="">
        <PreferenceRow
          v-for="item in groups[0].items"
          :key="item.label"
          :label="item.label"
          :summary="item.summary"
          :icon="item.icon"
          chevron
          @click="open(item)"
        />
      </PreferenceCategory>

      <PreferenceCategory
        v-for="group in groups.slice(1)"
        :key="group.title"
        :title="group.title"
      >
        <PreferenceRow
          v-for="item in group.items"
          :key="item.label"
          :label="item.label"
          :summary="item.summary"
          :icon="item.icon"
          chevron
          @click="open(item)"
        />
      </PreferenceCategory>
    </main>

    <ThemeModeDialog v-model="themeModeVisible" />

    <AndroidDialog v-model="helpVisible" mode="center" title="帮助" width="min(92vw, 760px)">
      <article class="help-body markdown-body" v-html="helpHtml" />
    </AndroidDialog>
  </div>
</template>

<style scoped>
.config-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  min-height: 0;
  background: var(--ac-background);
}
.pref-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 0 0 24px;
}
.help-body {
  font-size: 14px;
  line-height: 1.7;
  overflow-wrap: anywhere;
}
.help-body :deep(h1) {
  margin: 0 0 12px;
  font-size: 22px;
  line-height: 1.3;
}
.help-body :deep(h2) {
  margin: 22px 0 8px;
  padding-bottom: 4px;
  border-bottom: 1px solid var(--ac-divider);
  font-size: 18px;
}
.help-body :deep(h3) {
  margin: 18px 0 6px;
  font-size: 16px;
}
.help-body :deep(h4),
.help-body :deep(h5),
.help-body :deep(h6) {
  margin: 14px 0 6px;
  font-size: 15px;
}
.help-body :deep(p) {
  margin: 0 0 10px;
}
.help-body :deep(ul),
.help-body :deep(ol) {
  margin: 0 0 10px;
  padding-left: 22px;
}
.help-body :deep(li) {
  margin: 2px 0;
}
.help-body :deep(blockquote) {
  margin: 8px 0;
  padding: 6px 12px;
  border-left: 3px solid var(--ac-primary);
  background: var(--ac-transparent-10);
  color: var(--ac-secondary-text);
}
.help-body :deep(a) {
  color: var(--ac-primary);
  text-decoration: none;
}
.help-body :deep(code) {
  padding: 1px 4px;
  border-radius: 2px;
  background: var(--ac-transparent-10);
  font-family: ui-monospace, SFMono-Regular, Consolas, monospace;
  font-size: 0.92em;
}
.help-body :deep(pre) {
  overflow-x: auto;
  padding: 10px;
  background: var(--ac-transparent-10);
}
.help-body :deep(pre code) {
  padding: 0;
  background: transparent;
}
.help-body :deep(img) {
  display: block;
  max-width: 100%;
  height: auto;
  margin: 8px 0;
}
.help-body :deep(hr) {
  margin: 14px 0;
  border: none;
  border-top: 1px solid var(--ac-divider);
}
</style>
