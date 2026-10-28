<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AutoReadDialog from "@/views/reader/dialogs/AutoReadDialog.vue";
import ClickActionDialog from "@/views/reader/dialogs/ClickActionDialog.vue";
import PageKeyDialog from "@/views/reader/dialogs/PageKeyDialog.vue";
import MoreConfigDialog from "@/views/reader/dialogs/MoreConfigDialog.vue";

const router = useRouter();
const prefs = useReaderPrefsStore();
const autoRead = ref(false);
const clickAction = ref(false);
const pageKey = ref(false);
const more = ref(false);
</script>

<template>
  <div class="other-settings">
    <TitleBar title="其他设置" @back="router.back()" />
    <main class="pref-scroll">
      <section class="pref-group">
        <h2>阅读</h2>
        <label class="pref-row">
          <Icon name="ic_interface_setting" :size="24" class="pref-icon" />
          <span class="pref-text">
            <span class="pref-label">两端对齐</span>
            <span class="pref-summary">中文行两端对齐</span>
          </span>
          <input
            type="checkbox"
            :checked="prefs.fullJustify"
            @change="prefs.patch({ fullJustify: ($event.target as HTMLInputElement).checked })"
          />
        </label>
        <label class="pref-row">
          <Icon name="ic_toc" :size="24" class="pref-icon" />
          <span class="pref-text">
            <span class="pref-label">中文禁则断行</span>
            <span class="pref-summary">行首行尾标点禁则</span>
          </span>
          <input
            type="checkbox"
            :checked="prefs.useZhLayout"
            @change="prefs.patch({ useZhLayout: ($event.target as HTMLInputElement).checked })"
          />
        </label>
        <button class="pref-row" @click="autoRead = true">
          <Icon name="ic_auto_page" :size="24" class="pref-icon" />
          <span class="pref-text">
            <span class="pref-label">自动翻页</span>
            <span class="pref-summary">
              {{ prefs.autoReadSeconds > 0 ? `${prefs.autoReadSeconds} 秒` : "关闭" }}
            </span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="clickAction = true">
          <Icon name="ic_interface_setting" :size="24" class="pref-icon" />
          <span class="pref-text">
            <span class="pref-label">点击区域</span>
            <span class="pref-summary">3×3 九宫格动作</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="pageKey = true">
          <Icon name="ic_swap_horiz" :size="24" class="pref-icon" />
          <span class="pref-text">
            <span class="pref-label">翻页键</span>
            <span class="pref-summary">{{ prefs.pageKeyPrev }} / {{ prefs.pageKeyNext }}</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="more = true">
          <Icon name="ic_settings" :size="24" class="pref-icon" />
          <span class="pref-text">
            <span class="pref-label">更多阅读设置</span>
            <span class="pref-summary">亮度条、手势阈值</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
      </section>
    </main>

    <AutoReadDialog v-model="autoRead" />
    <ClickActionDialog v-model="clickAction" />
    <PageKeyDialog v-model="pageKey" />
    <MoreConfigDialog v-model="more" />
  </div>
</template>

<style scoped>
.other-settings {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.pref-scroll {
  flex: 1;
  overflow-y: auto;
}
.pref-group h2 {
  margin: 0;
  padding: 10px 20px 6px;
  font-size: 13px;
  color: var(--ac-primary);
}
.pref-row {
  width: 100%;
  min-height: 56px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 10px 20px;
  text-align: left;
  cursor: pointer;
}
.pref-icon {
  color: var(--ac-secondary-text);
}
.pref-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.pref-label {
  font-size: 15px;
}
.pref-summary {
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.chevron {
  opacity: 0.5;
}
input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
</style>
