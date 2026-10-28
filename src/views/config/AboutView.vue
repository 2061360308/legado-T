<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PreferenceRow from "@/components/android/PreferenceRow.vue";
import PreferenceCategory from "@/components/android/PreferenceCategory.vue";

const router = useRouter();
const message = ref("");

const mainRows = [
  { label: "贡献人员", summary: "gedoor、Invinciblelee、Xwite 等，详见 GitHub" },
  { label: "更新日志", summary: "" },
  { label: "检查更新", summary: "" },
];

const otherRows = [
  { label: "崩溃日志", summary: "" },
  { label: "保存日志", summary: "" },
  { label: "创建堆内存转储", summary: "" },
  { label: "用户隐私协议", summary: "" },
  { label: "许可协议", summary: "" },
  { label: "免责声明", summary: "" },
];

function notify(label: string) {
  message.value = `${label}暂未接入`;
}
</script>

<template>
  <div class="about-page">
    <TitleBar title="关于" @back="router.back()">
      <template #actions>
        <button aria-label="分享" @click="notify('分享')">
          <Icon name="ic_share" :size="24" />
        </button>
        <button aria-label="评分" @click="notify('评分')">
          <Icon name="ic_scoring" :size="24" />
        </button>
      </template>
    </TitleBar>

    <main class="content">
      <section class="about-card">
        <p class="app-name">本地阅读器</p>
        <p class="app-summary">
          本地 TXT 阅读器，复刻 legado 的本地书导入、分章、排版阅读、替换净化、朗读与备份能力。
        </p>
      </section>

      <div class="rows">
        <PreferenceRow
          v-for="row in mainRows"
          :key="row.label"
          :label="row.label"
          :summary="row.summary"
          @click="notify(row.label)"
        />
      </div>

      <PreferenceCategory title="其他">
        <PreferenceRow
          v-for="row in otherRows"
          :key="row.label"
          :label="row.label"
          @click="notify(row.label)"
        />
      </PreferenceCategory>

      <p v-if="message" class="message">{{ message }}</p>
    </main>
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
  padding: 10px;
  background: var(--ac-background-card);
  border-radius: 2px;
}
.app-name {
  margin: 0;
  text-align: center;
  font-size: 20px;
  font-weight: 600;
}
.app-summary {
  margin: 8px 0 0;
  font-size: 13px;
  line-height: 1.6;
  color: var(--ac-secondary-text);
}
.rows {
  margin-top: 8px;
}
.message {
  margin: 8px 20px 24px;
  color: var(--ac-primary);
  font-size: 12px;
}
</style>
