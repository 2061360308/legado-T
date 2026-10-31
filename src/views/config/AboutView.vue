<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PreferenceRow from "@/components/android/PreferenceRow.vue";
import PreferenceCategory from "@/components/android/PreferenceCategory.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";

const router = useRouter();
const message = ref("");
const infoVisible = ref(false);
const infoTitle = ref("");
const infoText = ref("");
const version = "0.1.0";

const aboutText =
  "本地阅读器是基于 legado 本地阅读流程复刻的 TXT 阅读应用，支持导入、分章、排版阅读、替换净化、朗读与备份。";

const mainRows = [
  { key: "contributors", label: "贡献人员", summary: "gedoor、Invinciblelee、Xwite 等" },
  { key: "changelog", label: "更新日志", summary: `版本 ${version}` },
  { key: "update", label: "检查更新", summary: "" },
];

const otherRows = [
  { key: "crash", label: "崩溃日志", summary: "查看最近崩溃信息" },
  { key: "saveLog", label: "保存日志", summary: "导出当前运行日志" },
  { key: "heap", label: "创建堆内存转储", summary: "" },
  { key: "privacy", label: "用户隐私协议", summary: "" },
  { key: "license", label: "许可协议", summary: "" },
  { key: "disclaimer", label: "免责声明", summary: "" },
];

function openInfo(title: string, text: string) {
  infoTitle.value = title;
  infoText.value = text;
  infoVisible.value = true;
}

function onRow(key: string, label: string) {
  if (key === "contributors") {
    openInfo(
      "贡献人员",
      "legado 原作者：gedoor\n主要贡献者：Invinciblelee、Xwite、Celeter、Horizoncbe、AGPapa 等。\n\n感谢所有参与测试与反馈的用户。",
    );
  } else if (key === "changelog") {
    openInfo(
      "更新日志",
      `v${version}\n- 本地书架搜索与分组管理\n- 批量移动到分组\n- TXT 编码、目录开关与字数统计\n- 阅读器菜单、文件与缓存管理\n- 书签、阅读记录与音频界面`,
    );
  } else if (key === "update") {
    message.value = `当前已是最新版本 ${version}`;
  } else if (key === "crash") {
    openInfo("崩溃日志", "暂无崩溃日志。");
  } else if (key === "saveLog") {
    void shareText("本地阅读器运行日志\n版本：" + version);
  } else if (key === "heap") {
    message.value = "当前平台不支持堆内存转储";
  } else if (key === "privacy") {
    openInfo(
      "用户隐私协议",
      "本地阅读器仅在设备本地保存书籍、阅读进度、书签和设置。\n应用不会主动上传个人数据，也不会收集通讯录、位置或设备标识。",
    );
  } else if (key === "license") {
    openInfo(
      "许可协议",
      "本项目参考 legado 的实现与界面。legado 以 GPL-3.0 许可发布。\n使用、修改和分发时请遵守对应开源许可。",
    );
  } else if (key === "disclaimer") {
    openInfo(
      "免责声明",
      "本应用仅用于阅读用户自行导入的本地书籍。\n用户应确保所导入内容的来源合法，并自行承担相关责任。",
    );
  } else {
    message.value = `${label}暂不可用`;
  }
}

async function shareText(text: string) {
  try {
    if (navigator.share) {
      await navigator.share({ title: "本地阅读器", text });
      return;
    }
    await navigator.clipboard.writeText(text);
    message.value = "内容已复制到剪贴板";
  } catch {
    message.value = "分享已取消";
  }
}

function shareApp() {
  void shareText(`本地阅读器 v${version}\n${aboutText}`);
}
</script>

<template>
  <div class="about-page">
    <TitleBar title="关于" @back="router.back()">
      <template #actions>
        <button aria-label="分享" @click="shareApp">
          <Icon name="ic_share" :size="24" />
        </button>
        <button aria-label="评分" @click="message = '感谢支持，评分入口暂未接入'" >
          <Icon name="ic_scoring" :size="24" />
        </button>
      </template>
    </TitleBar>

    <main class="content">
      <section class="about-card">
        <p class="app-name">本地阅读器</p>
        <p class="app-version">v{{ version }}</p>
        <p class="app-summary">{{ aboutText }}</p>
      </section>

      <div class="rows">
        <PreferenceRow
          v-for="row in mainRows"
          :key="row.key"
          :label="row.label"
          :summary="row.summary"
          @click="onRow(row.key, row.label)"
        />
      </div>

      <PreferenceCategory title="其他">
        <PreferenceRow
          v-for="row in otherRows"
          :key="row.key"
          :label="row.label"
          :summary="row.summary"
          @click="onRow(row.key, row.label)"
        />
      </PreferenceCategory>

      <p v-if="message" class="message">{{ message }}</p>
    </main>

    <AndroidDialog
      v-model="infoVisible"
      mode="center"
      :title="infoTitle"
      width="min(92vw, 480px)"
    >
      <p class="info-text">{{ infoText }}</p>
    </AndroidDialog>
  </div>
</template>

<style scoped>
.about-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
}
.about-card {
  margin: 6px;
  padding: 16px 10px;
  background: var(--ac-background-card);
  border-radius: 2px;
  text-align: center;
}
.app-name {
  margin: 0;
  font-size: 20px;
  font-weight: 600;
}
.app-version {
  margin: 4px 0 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.app-summary {
  margin: 10px 0 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--ac-secondary-text);
  text-align: left;
}
.rows {
  margin-top: 8px;
}
.message {
  margin: 8px 20px 24px;
  color: var(--ac-primary);
  font-size: 12px;
}
.info-text {
  margin: 0;
  max-height: 60vh;
  overflow: auto;
  white-space: pre-wrap;
  line-height: 1.7;
  font-size: 14px;
}
</style>
