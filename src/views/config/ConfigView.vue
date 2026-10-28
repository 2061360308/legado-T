<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import { getCurrentWindow } from "@tauri-apps/api/window";
import TitleBar from "@/components/android/TitleBar.vue";
import PreferenceRow from "@/components/android/PreferenceRow.vue";
import PreferenceCategory from "@/components/android/PreferenceCategory.vue";
import Icon from "@/components/android/Icon.vue";
import { useThemeStore } from "@/stores/theme";
import ThemeModeDialog from "./ThemeModeDialog.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const router = useRouter();
const theme = useThemeStore();
const themeModeVisible = ref(false);
const helpVisible = ref(false);
const webServiceOn = ref(false);
const webServiceTip = ref("");

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
      { label: "文件管理", summary: "管理私有文件夹的文件", icon: "ic_folder_outline", path: "/files" },
      { label: "关于", summary: "", icon: "ic_cfg_about", path: "/about" },
      { label: "原版界面清单", summary: "286 个 XML 的壳子与实现状态", icon: "ic_menu", path: "/legacy" },
      { label: "生成组件清单", summary: "转换器生成的 283 个 Vue 组件", icon: "ic_cfg_other", path: "/generated" },
      { label: "原版流程演示", summary: "按原 app 跳转图浏览全部生成界面", icon: "ic_bookshelf_list", path: "/shell" },
      { label: "退出", summary: "", icon: "ic_exit", action: () => getCurrentWindow().close() },
    ],
  },
]);

const currentModeLabel = computed(() => {
  const labels: Record<number, string> = {
    0: "日间",
    1: "夜间",
    2: "跟随系统",
    3: "墨水屏",
  };
  return labels[theme.mode] ?? "跟随系统";
});

function open(item: Item) {
  if (item.action) item.action();
  else if (item.path) router.push(item.path);
}

function onWebServiceChange(value: boolean) {
  webServiceOn.value = value;
  webServiceTip.value = value
    ? "桌面版暂不支持 Web 服务，已记录开关状态"
    : "Web 服务未开启";
}

const webServiceSummary = computed(() =>
  webServiceOn.value ? "Web 服务与上传页面（暂不支持）" : "开启后可在局域网内传书",
);
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
        <PreferenceRow
          label="Web 服务"
          :summary="webServiceSummary"
          icon="ic_cfg_web"
          @click="onWebServiceChange(!webServiceOn)"
        >
          <template #right>
            <input
              class="web-switch"
              type="checkbox"
              :checked="webServiceOn"
              @change="onWebServiceChange(($event.target as HTMLInputElement).checked)"
            />
          </template>
        </PreferenceRow>
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

    <p v-if="webServiceTip" class="web-tip">{{ webServiceTip }}</p>

    <ThemeModeDialog v-model="themeModeVisible" />

    <AndroidDialog v-model="helpVisible" mode="center" title="帮助">
      <div class="help-body">
        <p>帮助说明暂未提供。</p>
        <p class="help-hint">如需帮助请查看 README 或 Issues。</p>
      </div>
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
.web-switch {
  flex: none;
  width: 40px;
  height: 22px;
  appearance: none;
  border-radius: 11px;
  background: var(--ac-transparent-10);
  position: relative;
  cursor: pointer;
  outline: none;
  transition: background 0.15s;
}
.web-switch::after {
  content: "";
  position: absolute;
  top: 2px;
  left: 2px;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.15s;
}
.web-switch:checked {
  background: var(--ac-primary);
}
.web-switch:checked::after {
  transform: translateX(18px);
}
.web-tip {
  margin: 0;
  padding: 8px 20px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.help-body {
  padding: 4px 0;
}
.help-body p {
  margin: 0 0 8px;
  font-size: 14px;
}
.help-hint {
  color: var(--ac-secondary-text);
  font-size: 12px;
}
</style>
