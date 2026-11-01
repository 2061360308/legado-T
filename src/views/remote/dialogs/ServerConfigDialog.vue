<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from "vue";
import Icon from "@/components/android/Icon.vue";
import AndroidDialog from "@/views/reader/dialogs/AndroidDialog.vue";
import { testRemoteServer } from "@/core/application/remoteBooks";
import { saveRemoteServer } from "@/core/application/remoteServers";
import type {
  RemoteServer,
  RemoteServerType,
  S3ServerConfig,
  WebDavServerConfig,
} from "@/core/domain";

const props = defineProps<{
  modelValue: boolean;
  server: RemoteServer | null;
}>();

const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  saved: [server: RemoteServer];
}>();

const visible = computed({
  get: () => props.modelValue,
  set: (value: boolean) => emit("update:modelValue", value),
});

const formType = ref<RemoteServerType>("webdav");
const formName = ref("");
const formUrl = ref("");
const formUsername = ref("");
const formPassword = ref("");
const formEndpoint = ref("");
const formRegion = ref("us-east-1");
const formBucket = ref("");
const formAccessKey = ref("");
const formSecretKey = ref("");
const formSessionToken = ref("");
const formPrefix = ref("");
const formPathStyle = ref(false);
const saving = ref(false);
const saveError = ref("");

type TestState = "idle" | "testing" | "success" | "error";
const testState = ref<TestState>("idle");
const testMessage = ref("");
const TEST_TIMEOUT_MS = 15_000;
let testController: AbortController | null = null;
let testTimer: number | undefined;

function resetTestStatus() {
  testState.value = "idle";
  testMessage.value = "";
}

function cancelServerTest() {
  testController?.abort();
  testController = null;
  if (testTimer !== undefined) {
    window.clearTimeout(testTimer);
    testTimer = undefined;
  }
}

function fillForm(server: RemoteServer | null) {
  saveError.value = "";
  resetTestStatus();
  if (!server) {
    formType.value = "webdav";
    formName.value = "";
    formUrl.value = "";
    formUsername.value = "";
    formPassword.value = "";
    formEndpoint.value = "";
    formRegion.value = "us-east-1";
    formBucket.value = "";
    formAccessKey.value = "";
    formSecretKey.value = "";
    formSessionToken.value = "";
    formPrefix.value = "";
    formPathStyle.value = false;
    return;
  }
  formType.value = server.type;
  formName.value = server.name;
  if ("url" in server.config) {
    formUrl.value = server.config.url;
    formUsername.value = server.config.username;
    formPassword.value = server.config.password;
    return;
  }
  formEndpoint.value = server.config.endpoint;
  formRegion.value = server.config.region;
  formBucket.value = server.config.bucket;
  formAccessKey.value = server.config.accessKeyId;
  formSecretKey.value = server.config.secretAccessKey;
  formSessionToken.value = server.config.sessionToken ?? "";
  formPrefix.value = server.config.prefix ?? "";
  formPathStyle.value = server.config.pathStyle ?? false;
}

function validateForm(): string {
  if (formType.value === "webdav") {
    const url = formUrl.value.trim();
    if (!url) return "请填写服务器地址";
    try {
      new URL(url);
    } catch {
      return "服务器地址格式不正确";
    }
    return "";
  }
  if (!formBucket.value.trim()) return "请填写 Bucket";
  if (!formRegion.value.trim()) return "请填写 Region";
  if (!formAccessKey.value.trim()) return "请填写 Access Key";
  if (!formSecretKey.value) return "请填写 Secret Key";
  return "";
}

function buildServer(): RemoteServer {
  const config: WebDavServerConfig | S3ServerConfig =
    formType.value === "s3"
      ? {
          endpoint: formEndpoint.value,
          region: formRegion.value,
          bucket: formBucket.value,
          accessKeyId: formAccessKey.value,
          secretAccessKey: formSecretKey.value,
          sessionToken: formSessionToken.value.trim() || undefined,
          prefix: formPrefix.value,
          pathStyle: formPathStyle.value,
        }
      : {
          url: formUrl.value,
          username: formUsername.value,
          password: formPassword.value,
        };
  return {
    id: props.server?.id ?? Date.now(),
    name: formName.value.trim() || (formType.value === "s3" ? "S3" : "WebDAV"),
    type: formType.value,
    config,
    sortNumber: props.server?.sortNumber ?? Date.now(),
  };
}

function describeTestError(error: unknown, timedOut: boolean): string {
  if (timedOut) return "连接超时，请检查地址和网络";
  const text = error instanceof Error ? error.message : String(error);
  if (/WebDAV\s+(401|403)\b|S3\s+403\b/i.test(text)) {
    return "认证失败或无权限，请检查账号和密码";
  }
  if (/WebDAV\s+404\b/i.test(text)) return "服务器路径不存在";
  if (/Invalid URL|Failed to construct 'URL'/i.test(text)) {
    return "服务器地址格式不正确";
  }
  if (
    (error instanceof DOMException && error.name === "AbortError") ||
    String(error).includes("AbortError")
  ) {
    return "测试已取消";
  }
  return `连接失败：${text}`;
}

async function testServer() {
  if (testState.value === "testing" || saving.value) return;
  const validation = validateForm();
  if (validation) {
    testState.value = "error";
    testMessage.value = validation;
    return;
  }
  cancelServerTest();
  const controller = new AbortController();
  testController = controller;
  let timedOut = false;
  testState.value = "testing";
  testMessage.value = "正在测试连接…";
  testTimer = window.setTimeout(() => {
    timedOut = true;
    controller.abort();
  }, TEST_TIMEOUT_MS);
  try {
    await testRemoteServer(buildServer(), controller.signal);
    if (testController !== controller) return;
    testState.value = "success";
    testMessage.value = "连接成功";
  } catch (error) {
    if (testController !== controller) return;
    testState.value = "error";
    testMessage.value = describeTestError(error, timedOut);
  } finally {
    if (testController === controller) {
      if (testTimer !== undefined) {
        window.clearTimeout(testTimer);
        testTimer = undefined;
      }
      testController = null;
    }
  }
}

async function saveServer() {
  if (saving.value) return;
  const validation = validateForm();
  if (validation) {
    saveError.value = validation;
    return;
  }
  saving.value = true;
  saveError.value = "";
  try {
    const saved = await saveRemoteServer(buildServer());
    emit("saved", saved);
    visible.value = false;
  } catch (error) {
    saveError.value = String(error);
  } finally {
    saving.value = false;
  }
}

watch(
  () => props.server,
  (server) => fillForm(server),
  { immediate: true },
);

watch(
  () => props.modelValue,
  (open) => {
    if (open) {
      fillForm(props.server);
      return;
    }
    cancelServerTest();
    resetTestStatus();
  },
);

onBeforeUnmount(() => {
  cancelServerTest();
});
</script>

<template>
  <AndroidDialog
    v-model="visible"
    title="服务器配置"
    mode="center"
    width="min(100%, 640px)"
  >
    <template #actions>
      <button class="header-action" :disabled="saving" @click="saveServer">
        <Icon name="ic_save" :size="20" />
        <span>{{ saving ? "保存中…" : "保存" }}</span>
      </button>
    </template>
    <div class="form">
      <label>名称<input v-model="formName" /></label>
      <label>类型
        <select v-model="formType">
          <option value="webdav">WEBDAV</option>
          <option value="s3">S3</option>
        </select>
      </label>
      <template v-if="formType === 'webdav'">
        <label>服务器地址<input v-model="formUrl" placeholder="https://dav.example.com/dav/" /></label>
        <label>账号<input v-model="formUsername" /></label>
        <label>密码<input v-model="formPassword" type="password" /></label>
      </template>
      <template v-else>
        <label>Endpoint<input v-model="formEndpoint" placeholder="https://s3.us-east-1.amazonaws.com" /></label>
        <label>Region<input v-model="formRegion" /></label>
        <label>Bucket<input v-model="formBucket" /></label>
        <label>Prefix<input v-model="formPrefix" placeholder="可选，例如 books/" /></label>
        <label>Access Key<input v-model="formAccessKey" /></label>
        <label>Secret Key<input v-model="formSecretKey" type="password" /></label>
        <label>Session Token（可选）<input v-model="formSessionToken" type="password" /></label>
        <label class="checkbox"><input v-model="formPathStyle" type="checkbox" /> path-style endpoint</label>
      </template>
      <div class="form-actions">
        <button :disabled="saving || testState === 'testing'" @click="testServer">
          {{ testState === "testing" ? "测试中…" : "测试连接" }}
        </button>
      </div>
      <p
        v-if="testMessage"
        class="test-status"
        :class="testState"
        role="status"
        aria-live="polite"
      >
        {{ testMessage }}
      </p>
      <p v-if="saveError" class="save-error">{{ saveError }}</p>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.form label {
  display: flex;
  flex-direction: column;
  gap: 4px;
}
.form label.checkbox {
  flex-direction: row;
  align-items: center;
}
.form label.checkbox input {
  width: auto;
}
.form input,
.form select {
  width: 100%;
  box-sizing: border-box;
  border: 1px solid var(--ac-divider);
  border-radius: 4px;
  background: var(--ac-background-menu);
  color: var(--ac-primary-text);
  padding: 8px;
}
.form-actions {
  display: flex;
  gap: 8px;
}
.form-actions button {
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 6px 10px;
}
.form-actions button:disabled {
  opacity: 0.5;
}
.test-status {
  margin: 0;
  font-size: 12px;
  line-height: 1.5;
}
.test-status.testing {
  color: var(--ac-secondary-text);
}
.test-status.success {
  color: var(--ac-success);
}
.test-status.error {
  color: var(--ac-error);
}
.save-error {
  margin: 0;
  color: var(--ac-error);
  font-size: 12px;
  line-height: 1.5;
}
.header-action {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}
</style>
