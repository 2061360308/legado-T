<script setup lang="ts">
import { computed, ref } from "vue";
import { useRouter } from "vue-router";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import {
  BUILTIN_SCHEME_COUNT,
  useThemeStore,
  type ReadStyleScheme,
} from "@/stores/theme";

const router = useRouter();
const theme = useThemeStore();
const newName = ref("");
const message = ref("");

const scheme = computed(() => theme.currentSchemeData);
const parts = [
  { key: "day", label: "日间" },
  { key: "night", label: "夜间" },
  { key: "eink", label: "墨水屏" },
] as const;

function update(
  part: "day" | "night" | "eink",
  field: "bgValue" | "textColor",
  value: string,
) {
  const next = JSON.parse(JSON.stringify(scheme.value)) as ReadStyleScheme;
  next[part][field] = value;
  theme.updateScheme(theme.currentScheme, next);
}

function rename(value: string) {
  const next = JSON.parse(JSON.stringify(scheme.value)) as ReadStyleScheme;
  next.name = value;
  theme.updateScheme(theme.currentScheme, next);
}

function addScheme() {
  theme.addScheme(newName.value.trim());
  newName.value = "";
}

function removeScheme(index: number) {
  if (index < BUILTIN_SCHEME_COUNT) {
    message.value = "前 5 套预置方案不可删除";
    return;
  }
  if (confirm(`删除方案「${theme.schemes[index].name}」？`)) {
    theme.removeScheme(index);
    message.value = "方案已删除";
  }
}
</script>

<template>
  <div class="schemes-page">
    <TitleBar title="阅读排版方案" @back="router.back()" />
    <main class="pref-scroll">
      <section class="pref-group">
        <h2>方案列表</h2>
        <div v-for="(item, index) in theme.schemes" :key="index" class="scheme-row">
          <button class="scheme-main" @click="theme.setScheme(index)">
            <span
              class="swatch"
              :style="{ background: item.day.bgValue, color: item.day.textColor }"
            >文</span>
            <span class="pref-text">
              <span class="pref-label">
                {{ item.name }}
                <span v-if="index < BUILTIN_SCHEME_COUNT" class="tag">预置</span>
              </span>
              <span class="pref-summary">
                {{ item.day.bgValue }} / {{ item.day.textColor }}
              </span>
            </span>
            <span class="check">{{ theme.currentScheme === index ? "●" : "○" }}</span>
          </button>
          <button
            v-if="index >= BUILTIN_SCHEME_COUNT"
            class="delete"
            aria-label="删除"
            @click="removeScheme(index)"
          >
            <Icon name="ic_clear_all" :size="20" />
          </button>
        </div>
        <div class="add-row">
          <input v-model="newName" placeholder="新方案名称" />
          <button @click="addScheme">新增方案</button>
        </div>
      </section>

      <section v-if="scheme" class="pref-group">
        <h2>当前方案：{{ scheme.name }}</h2>
        <label class="pref-row">
          <span class="pref-text">
            <span class="pref-label">方案名称</span>
          </span>
          <input
            class="name-input"
            :value="scheme.name"
            @input="rename(($event.target as HTMLInputElement).value)"
          />
        </label>
        <template v-for="part in parts" :key="part.key">
          <label class="pref-row">
            <span class="pref-text">
              <span class="pref-label">{{ part.label }}背景色</span>
              <span class="pref-summary">{{ scheme[part.key].bgValue }}</span>
            </span>
            <input
              type="color"
              :value="scheme[part.key].bgValue"
              @input="update(part.key, 'bgValue', ($event.target as HTMLInputElement).value)"
            />
          </label>
          <label class="pref-row">
            <span class="pref-text">
              <span class="pref-label">{{ part.label }}文字色</span>
              <span class="pref-summary">{{ scheme[part.key].textColor }}</span>
            </span>
            <input
              type="color"
              :value="scheme[part.key].textColor"
              @input="update(part.key, 'textColor', ($event.target as HTMLInputElement).value)"
            />
          </label>
        </template>
      </section>

      <p v-if="message" class="message">{{ message }}</p>
    </main>
  </div>
</template>

<style scoped>
.schemes-page {
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
.scheme-row {
  display: flex;
  align-items: center;
  border-bottom: 1px solid var(--ac-divider);
}
.scheme-main {
  flex: 1;
  min-width: 0;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 20px;
  text-align: left;
  cursor: pointer;
}
.scheme-main:hover {
  background: var(--ac-transparent-10);
}
.swatch {
  width: 34px;
  height: 34px;
  border: 1px solid var(--ac-divider);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  flex: none;
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
}
.tag {
  margin-left: 6px;
  font-size: 11px;
  color: var(--ac-secondary-text);
}
.check {
  color: var(--ac-primary);
}
.delete {
  width: 44px;
  height: 44px;
  border: none;
  background: transparent;
  color: var(--ac-error);
  cursor: pointer;
}
.add-row {
  display: flex;
  gap: 8px;
  padding: 12px 20px;
}
.add-row input,
.name-input {
  flex: 1;
  min-width: 0;
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
}
.add-row button {
  border: 1px solid var(--ac-primary);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 8px 14px;
  cursor: pointer;
}
.pref-row {
  display: flex;
  align-items: center;
  gap: 12px;
  min-height: 52px;
  padding: 8px 20px;
  border-bottom: 1px solid var(--ac-divider);
}
input[type="color"] {
  width: 42px;
  height: 30px;
  border: 1px solid var(--ac-divider);
  background: transparent;
  padding: 1px;
}
.message {
  margin: 8px 20px 0;
  color: var(--ac-primary);
  font-size: 12px;
}
</style>
