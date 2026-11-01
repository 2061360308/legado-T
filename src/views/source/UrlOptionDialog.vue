<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  confirm: [json: string];
}>();

const method = ref("");
const charset = ref("");
const headers = ref("");
const body = ref("");
const retry = ref("");
const type = ref("");
const webView = ref(false);
const webJs = ref("");
const js = ref("");
const webViewDelayTime = ref("");

watch(
  () => props.modelValue,
  (open) => {
    if (!open) return;
    method.value = "";
    charset.value = "";
    headers.value = "";
    body.value = "";
    retry.value = "";
    type.value = "";
    webView.value = false;
    webJs.value = "";
    js.value = "";
    webViewDelayTime.value = "";
  },
);

function close() {
  emit("update:modelValue", false);
}

function build() {
  const option: Record<string, unknown> = {};
  if (method.value.trim()) option.method = method.value.trim();
  if (charset.value.trim()) option.charset = charset.value.trim();
  if (headers.value.trim()) option.headers = headers.value.trim();
  if (body.value.trim()) option.body = body.value;
  if (retry.value.trim()) option.retry = Number(retry.value) || undefined;
  if (type.value.trim()) option.type = type.value.trim();
  if (webView.value) option.webView = true;
  if (webJs.value.trim()) option.webJs = webJs.value;
  if (js.value.trim()) option.js = js.value;
  if (webViewDelayTime.value.trim()) {
    option.webViewDelayTime = Number(webViewDelayTime.value) || undefined;
  }
  for (const key of Object.keys(option)) {
    if (option[key] === undefined) delete option[key];
  }
  emit("confirm", JSON.stringify(option));
  close();
}
</script>

<template>
  <div v-if="modelValue" class="overlay" @click.self="close">
    <section class="panel">
      <header>
        <span>URL 参数</span>
        <button @click="close">关闭</button>
      </header>
      <label>Method<input v-model="method" placeholder="GET / POST" /></label>
      <label>Charset<input v-model="charset" placeholder="UTF-8" /></label>
      <label>Headers<textarea v-model="headers" class="small" placeholder='{"User-Agent":"..."}' /></label>
      <label>Body<textarea v-model="body" class="small" placeholder="POST body" /></label>
      <div class="grid">
        <label>Retry<input v-model="retry" type="number" placeholder="0" /></label>
        <label>Type<input v-model="type" placeholder="可选" /></label>
        <label>WebView 延迟 ms<input v-model="webViewDelayTime" type="number" placeholder="0" /></label>
      </div>
      <label class="check"><input v-model="webView" type="checkbox" /> 使用 WebView</label>
      <label>Web JS<textarea v-model="webJs" class="small" placeholder="WebView 中执行" /></label>
      <label>JS<textarea v-model="js" class="small" placeholder="URL 解析完成后执行，返回值作为 URL" /></label>
      <footer>
        <button @click="close">取消</button>
        <button class="primary" @click="build">确定</button>
      </footer>
    </section>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 40;
  display: flex;
  align-items: flex-end;
  background: rgba(0, 0, 0, 0.48);
}
.panel {
  width: 100%;
  max-height: 86vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 10px 16px calc(16px + env(safe-area-inset-bottom));
  border-radius: 12px 12px 0 0;
  background: var(--ac-background);
}
.panel header,
.panel footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--ac-primary-text);
  font-size: 15px;
}
.panel header button,
.panel footer button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 13px;
  padding: 6px;
}
.panel footer {
  justify-content: flex-end;
  gap: 8px;
  padding-top: 4px;
}
.panel footer button.primary {
  background: var(--ac-primary);
  color: #fff;
  border-radius: 4px;
  padding: 7px 16px;
}
label {
  display: flex;
  flex-direction: column;
  gap: 4px;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
input:not([type="checkbox"]),
textarea {
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 7px 9px;
  border-radius: 3px;
  font-size: 13px;
}
textarea.small {
  min-height: 58px;
  resize: vertical;
  font-family: Consolas, monospace;
  font-size: 12px;
}
.grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 8px;
}
.check {
  flex-direction: row;
  align-items: center;
  gap: 6px;
}
input[type="checkbox"] {
  accent-color: var(--ac-primary);
}
</style>
