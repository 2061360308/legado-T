<script setup lang="ts">
import { nextTick, onBeforeUnmount, ref, watch } from "vue";
import QRCode from "qrcode";
import jsQR from "jsqr";

const props = defineProps<{
  modelValue: boolean;
  mode: "share" | "import";
  text?: string;
}>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  import: [text: string];
}>();

const dataUrl = ref("");
const manual = ref("");
const message = ref("");
const fileInput = ref<HTMLInputElement | null>(null);
const video = ref<HTMLVideoElement | null>(null);
const cameraOn = ref(false);
let stream: MediaStream | null = null;
let frame = 0;

function close() {
  stopCamera();
  emit("update:modelValue", false);
}

async function generate() {
  dataUrl.value = "";
  message.value = "";
  const text = String(props.text || "").trim();
  if (!text) return;
  try {
    dataUrl.value = await QRCode.toDataURL(text, {
      width: 320,
      margin: 1,
      errorCorrectionLevel: "M",
      color: { dark: "#000000", light: "#ffffff" },
    });
  } catch (error) {
    message.value = `二维码生成失败：${error}`;
  }
}

function accept(text: string) {
  const value = text.trim();
  if (!value) return;
  try {
    const parsed = JSON.parse(value) as unknown;
    if (!parsed || typeof parsed !== "object") throw new Error("不是 JSON 对象");
  } catch (error) {
    message.value = `二维码内容不是书源 JSON：${error}`;
    return;
  }
  emit("import", value);
  close();
}

function pickFile() {
  fileInput.value?.click();
}

async function onFile(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  message.value = "";
  try {
    const image = new Image();
    const url = URL.createObjectURL(file);
    image.src = url;
    await image.decode();
    const canvas = document.createElement("canvas");
    canvas.width = image.naturalWidth;
    canvas.height = image.naturalHeight;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("无法读取图片");
    context.drawImage(image, 0, 0);
    URL.revokeObjectURL(url);
    const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
    const code = jsQR(pixels.data, pixels.width, pixels.height);
    if (!code?.data) throw new Error("图片中没有二维码");
    accept(code.data);
  } catch (error) {
    message.value = String(error);
  } finally {
    if (event.target instanceof HTMLInputElement) event.target.value = "";
  }
}

function stopCamera() {
  cameraOn.value = false;
  if (frame) cancelAnimationFrame(frame);
  frame = 0;
  stream?.getTracks().forEach((track) => track.stop());
  stream = null;
}

async function startCamera() {
  message.value = "";
  try {
    stopCamera();
    stream = await navigator.mediaDevices.getUserMedia({
      video: { facingMode: "environment" },
      audio: false,
    });
    cameraOn.value = true;
    await nextTick();
    const element = video.value;
    if (!element) throw new Error("视频组件未就绪");
    element.srcObject = stream;
    await element.play();
    const canvas = document.createElement("canvas");
    const context = canvas.getContext("2d");
    if (!context) throw new Error("无法创建画布");
    const tick = () => {
      if (!cameraOn.value || !video.value) return;
      const videoElement = video.value;
      if (videoElement.readyState >= 2) {
        canvas.width = videoElement.videoWidth;
        canvas.height = videoElement.videoHeight;
        context.drawImage(videoElement, 0, 0, canvas.width, canvas.height);
        const pixels = context.getImageData(0, 0, canvas.width, canvas.height);
        const code = jsQR(pixels.data, pixels.width, pixels.height);
        if (code?.data) {
          stopCamera();
          accept(code.data);
          return;
        }
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
  } catch (error) {
    stopCamera();
    message.value = `无法打开摄像头：${error}`;
  }
}

async function copyText() {
  const value = props.mode === "share" ? String(props.text || "") : manual.value;
  if (!value.trim()) return;
  await navigator.clipboard.writeText(value);
  message.value = "已复制";
}

watch(
  () => props.modelValue,
  async (open) => {
    if (!open) {
      stopCamera();
      return;
    }
    manual.value = "";
    message.value = "";
    if (props.mode === "share") await generate();
  },
  { immediate: true },
);

onBeforeUnmount(stopCamera);
</script>

<template>
  <div v-if="modelValue" class="overlay" @click.self="close">
    <section class="panel">
      <header>
        <span>{{ mode === "share" ? "书源二维码" : "二维码导入" }}</span>
        <button @click="close">关闭</button>
      </header>

      <template v-if="mode === 'share'">
        <div v-if="dataUrl" class="qr-wrap">
          <img :src="dataUrl" alt="书源二维码" />
        </div>
        <p v-else class="hint">内容过长或生成失败，无法生成二维码。</p>
        <textarea :value="text || ''" readonly class="json" />
        <div class="row">
          <button @click="copyText">复制书源 JSON</button>
        </div>
      </template>

      <template v-else>
        <p class="hint">扫码、选择二维码图片，或直接粘贴书源 JSON。</p>
        <textarea v-model="manual" class="json" placeholder='[{"bookSourceName":"..."}]' />
        <div class="row">
          <button @click="pickFile">选择图片</button>
          <button v-if="!cameraOn" @click="startCamera">打开摄像头</button>
          <button v-else @click="stopCamera">停止摄像头</button>
          <button @click="accept(manual)">导入文本</button>
          <button @click="copyText">复制</button>
        </div>
        <video v-if="cameraOn" ref="video" class="camera" muted playsinline />
        <input ref="fileInput" class="file" type="file" accept="image/*" @change="onFile" />
      </template>

      <p v-if="message" class="message">{{ message }}</p>
    </section>
  </div>
</template>

<style scoped>
.overlay {
  position: fixed;
  inset: 0;
  z-index: 45;
  display: flex;
  align-items: flex-end;
  background: rgba(0, 0, 0, 0.5);
}
.panel {
  width: 100%;
  max-height: 88vh;
  overflow-y: auto;
  padding: 10px 16px calc(16px + env(safe-area-inset-bottom));
  border-radius: 12px 12px 0 0;
  background: var(--ac-background);
  display: flex;
  flex-direction: column;
  gap: 8px;
}
header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--ac-primary-text);
  font-size: 15px;
}
header button,
.row button {
  border: none;
  background: transparent;
  color: var(--ac-primary);
  font-size: 13px;
  padding: 6px;
}
.row {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}
.qr-wrap {
  display: flex;
  justify-content: center;
  padding: 6px;
  background: #fff;
  border-radius: 6px;
}
.qr-wrap img {
  width: min(320px, 80vw);
  height: min(320px, 80vw);
}
.hint,
.message {
  margin: 0;
  color: var(--ac-secondary-text);
  font-size: 12px;
}
.json {
  width: 100%;
  min-height: 70px;
  max-height: 180px;
  resize: vertical;
  border: 1px solid var(--ac-divider);
  background: var(--ac-background);
  color: var(--ac-primary-text);
  padding: 8px;
  font-family: Consolas, monospace;
  font-size: 12px;
}
.camera {
  width: 100%;
  max-height: 280px;
  object-fit: cover;
  background: #000;
  border-radius: 6px;
}
.file {
  display: none;
}
</style>
