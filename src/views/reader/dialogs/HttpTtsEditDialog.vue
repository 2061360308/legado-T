<script setup lang="ts">
import { ref, watch } from "vue";
import type { HttpTts } from "@/core/domain";
import { httpTtsSave } from "@/core/application/rules";
import AndroidDialog from "./AndroidDialog.vue";

const props = defineProps<{ modelValue: boolean; engine: HttpTts | null }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  saved: [];
}>();

const form = ref<HttpTts>(empty());

function empty(): HttpTts {
  return {
    id: 0,
    name: "",
    url: "",
    contentType: "audio/mpeg",
    concurrentRate: "0",
    loginUrl: "",
    loginUi: "",
    header: "",
    jsLib: "",
    enabledCookieJar: false,
    loginCheckJs: "",
    lastUpdateTime: 0,
  };
}

watch(
  () => props.engine,
  (engine) => {
    form.value = engine ? { ...engine } : empty();
  },
  { immediate: true },
);

async function save() {
  if (!form.value.name.trim() || !form.value.url.trim()) return;
  form.value = await httpTtsSave({ ...form.value });
  emit("saved");
  emit("update:modelValue", false);
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="fullscreen"
    title="在线语音引擎"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="form">
      <label>名称<input v-model="form.name" /></label>
      <label>合成 URL<input v-model="form.url" placeholder="...{{speakText}}...{{speakSpeed}}" /></label>
      <div class="grid">
        <label>Content-Type<input v-model="form.contentType" /></label>
        <label>并发率<input v-model="form.concurrentRate" /></label>
      </div>
      <label>请求头<input v-model="form.header" /></label>
      <label>登录地址<input v-model="form.loginUrl" /></label>
      <label>登录界面<input v-model="form.loginUi" /></label>
      <label>JS 库<input v-model="form.jsLib" /></label>
      <label>登录校验 JS<input v-model="form.loginCheckJs" /></label>
      <label class="check">
        <input v-model="form.enabledCookieJar" type="checkbox" />
        启用 CookieJar
      </label>
    </div>
    <template #footer>
      <button class="btn" @click="emit('update:modelValue', false)">取消</button>
      <button class="btn primary" @click="save">保存</button>
    </template>
  </AndroidDialog>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  font-size: 13px;
}
label.check {
  flex-direction: row;
  align-items: center;
  gap: 6px;
}
input:not([type="checkbox"]) {
  padding: 8px 10px;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 2px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 10px;
}
.btn {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 2px;
  padding: 8px 16px;
  cursor: pointer;
}
.btn.primary {
  color: var(--ac-primary);
  border-color: var(--ac-primary);
}
</style>
