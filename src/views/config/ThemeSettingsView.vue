<script setup lang="ts">
import { ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import { useThemeStore } from "@/stores/theme";

const router = useRouter();
const theme = useThemeStore();
const message = ref("");
const statusBar = ref(true);
const navigationBar = ref(true);

const dayColors = [
  ["primary", "主色"],
  ["accent", "强调色"],
  ["background", "背景色"],
  ["bottomBackground", "导航栏色"],
] as const;
const nightColors = dayColors;

function update(
  part: "day" | "night",
  key: "primary" | "accent" | "background" | "bottomBackground",
  value: string,
) {
  theme.updateGlobal(part, key, value);
}

function notify(text: string) {
  message.value = text;
}
</script>

<template>
  <div class="theme-page">
    <TitleBar title="主题设置" @back="router.back()" />
    <main class="pref-scroll">
      <section class="pref-group">
        <button class="pref-row" @click="notify('更换图标暂未实现')">
          <span class="pref-text">
            <span class="pref-label">更换图标</span>
            <span class="pref-summary">启动图标样式</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <button class="pref-row" @click="notify('欢迎页样式暂未实现')">
          <span class="pref-text">
            <span class="pref-label">欢迎页样式</span>
            <span class="pref-summary">启动欢迎页</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">沉浸状态栏</span>
            <span class="pref-summary">状态栏透明</span>
          </span>
          <input v-model="statusBar" type="checkbox" />
        </label>
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">沉浸导航栏</span>
            <span class="pref-summary">导航栏透明</span>
          </span>
          <input v-model="navigationBar" type="checkbox" />
        </label>
        <button class="pref-row" @click="notify('标题栏阴影高度：10dp')">
          <span class="pref-text">
            <span class="pref-label">标题栏阴影高度</span>
            <span class="pref-summary">10dp</span>
          </span>
        </button>
        <button class="pref-row" @click="notify('字体缩放：100%')">
          <span class="pref-text">
            <span class="pref-label">字体缩放</span>
            <span class="pref-summary">100%</span>
          </span>
        </button>
        <button class="pref-row" @click="notify('封面配置暂未实现')">
          <span class="pref-text">
            <span class="pref-label">封面配置</span>
            <span class="pref-summary">书籍封面显示方式</span>
          </span>
        </button>
        <button class="pref-row" @click="router.push('/read-schemes')">
          <span class="pref-text">
            <span class="pref-label">阅读排版方案</span>
            <span class="pref-summary">阅读页背景、文字色与排版方案</span>
          </span>
          <Icon name="ic_expand_more" :size="20" class="chevron" />
        </button>
      </section>

      <section class="pref-group">
        <h2>日间</h2>
        <label v-for="[key, label] in dayColors" :key="key" class="pref-row">
          <span class="pref-text">
            <span class="pref-label">{{ label }}</span>
            <span class="pref-summary">{{ theme.globalTokens.day[key] }}</span>
          </span>
          <input
            type="color"
            :value="theme.globalTokens.day[key]"
            @input="update('day', key, ($event.target as HTMLInputElement).value)"
          />
        </label>
        <button class="pref-row" @click="notify('日间背景图片暂未实现')">
          <span class="pref-text">
            <span class="pref-label">背景图片</span>
            <span class="pref-summary">未设置</span>
          </span>
        </button>
        <button class="pref-row" @click="notify('日间主题已保存')">
          <span class="pref-text">
            <span class="pref-label">保存主题配置</span>
            <span class="pref-summary">保存当前日间配色</span>
          </span>
        </button>
      </section>

      <section class="pref-group">
        <h2>夜间</h2>
        <label v-for="[key, label] in nightColors" :key="key" class="pref-row">
          <span class="pref-text">
            <span class="pref-label">{{ label }}</span>
            <span class="pref-summary">{{ theme.globalTokens.night[key] }}</span>
          </span>
          <input
            type="color"
            :value="theme.globalTokens.night[key]"
            @input="update('night', key, ($event.target as HTMLInputElement).value)"
          />
        </label>
        <button class="pref-row" @click="notify('夜间背景图片暂未实现')">
          <span class="pref-text">
            <span class="pref-label">背景图片</span>
            <span class="pref-summary">未设置</span>
          </span>
        </button>
        <button class="pref-row" @click="notify('夜间主题已保存')">
          <span class="pref-text">
            <span class="pref-label">保存主题配置</span>
            <span class="pref-summary">保存当前夜间配色</span>
          </span>
        </button>
      </section>

      <p v-if="message" class="message">{{ message }}</p>
    </main>
  </div>
</template>

<style scoped>
.theme-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.pref-scroll {
  flex: 1;
  overflow-y: auto;
  padding-bottom: 24px;
}
.pref-group h2 {
  margin: 0;
  padding: 10px 20px 6px;
  font-size: 13px;
  font-weight: 500;
  color: var(--ac-primary);
}
.pref-row {
  width: 100%;
  min-height: 52px;
  border: none;
  border-bottom: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  text-align: left;
  cursor: pointer;
}
.pref-row:hover {
  background: var(--ac-transparent-10);
}
.pref-text {
  flex: 1;
  min-width: 0;
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.chevron {
  opacity: 0.5;
}
input[type="checkbox"] {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
input[type="color"] {
  width: 42px;
  height: 30px;
  border: 1px solid var(--ac-divider);
  border-radius: 2px;
  background: transparent;
  padding: 1px;
}
.message {
  margin: 8px 20px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
</style>
