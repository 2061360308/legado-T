<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { ReplaceRule } from "@/core/domain";
import { replaceRuleSave, replaceRules } from "@/core/application/rules";
import { readerInvalidateAll } from "@/core/application/reader";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";

const route = useRoute();
const router = useRouter();
const rule = ref<ReplaceRule>(emptyRule());
const message = ref("");
const saving = ref(false);

function emptyRule(): ReplaceRule {
  return {
    id: 0,
    name: "",
    group: "",
    pattern: "",
    replacement: "",
    scope: "",
    excludeScope: "",
    scopeTitle: false,
    scopeContent: true,
    isEnabled: true,
    isRegex: true,
    timeoutMillisecond: 3000,
    sortOrder: 0,
  };
}

async function load() {
  const id = Number(route.query.id || 0);
  const list = await replaceRules();
  if (id > 0) {
    const found = list.find((item) => item.id === id);
    if (found) rule.value = { ...found };
  } else {
    rule.value.sortOrder = (list.at(-1)?.sortOrder ?? 0) + 1;
  }
}

async function save() {
  if (!rule.value.name.trim() || !rule.value.pattern) {
    message.value = "规则名与匹配内容不能为空";
    return;
  }
  saving.value = true;
  try {
    await replaceRuleSave({ ...rule.value });
    await readerInvalidateAll();
    router.back();
  } catch (e) {
    message.value = String(e);
  } finally {
    saving.value = false;
  }
}

onMounted(load);
</script>

<template>
  <div class="edit-page">
    <TitleBar :title="rule.id ? '编辑替换规则' : '新建替换规则'" @back="router.back()">
      <template #actions>
        <button aria-label="保存" :disabled="saving" @click="save">
          <Icon name="ic_save" :size="24" />
        </button>
      </template>
    </TitleBar>
    <main class="content">
      <label class="field">
        <span>规则名称</span>
        <input v-model="rule.name" placeholder="规则名称" />
      </label>
      <label class="field">
        <span>分组</span>
        <input v-model="rule.group" placeholder="分组，可留空" />
      </label>
      <label class="field">
        <span>匹配规则</span>
        <textarea v-model="rule.pattern" rows="4" />
      </label>
      <label class="field">
        <span>替换为</span>
        <textarea v-model="rule.replacement" rows="3" />
      </label>
      <label class="field">
        <span>作用范围</span>
        <input v-model="rule.scope" placeholder="书名包含，可留空" />
      </label>
      <label class="field">
        <span>排除范围</span>
        <input v-model="rule.excludeScope" placeholder="书名包含则跳过" />
      </label>
      <div class="switch-grid">
        <label><input v-model="rule.isEnabled" type="checkbox" />启用</label>
        <label><input v-model="rule.isRegex" type="checkbox" />正则</label>
        <label><input v-model="rule.scopeTitle" type="checkbox" />作用于标题</label>
        <label><input v-model="rule.scopeContent" type="checkbox" />作用于正文</label>
      </div>
      <label class="field">
        <span>超时（毫秒）</span>
        <input v-model.number="rule.timeoutMillisecond" type="number" min="100" />
      </label>
      <label class="field">
        <span>排序</span>
        <input v-model.number="rule.sortOrder" type="number" />
      </label>
      <p v-if="message" class="message">{{ message }}</p>
    </main>
  </div>
</template>

<style scoped>
.edit-page {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--ac-background);
}
.content {
  flex: 1;
  overflow-y: auto;
  padding: 12px 16px 24px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.field {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-size: 12px;
  color: var(--ac-secondary-text);
}
.field input,
.field textarea {
  width: 100%;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 9px 10px;
  font: inherit;
  resize: vertical;
}
.switch-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
  font-size: 13px;
}
.switch-grid label {
  display: flex;
  align-items: center;
  gap: 6px;
}
.message {
  margin: 0;
  color: var(--ac-error);
  font-size: 12px;
}
</style>
