<script setup lang="ts">
import { computed } from "vue";
import { imageSource } from "../canvas/drawPage";

const props = defineProps<{
  modelValue: boolean;
  src: string;
}>();

const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const url = computed(() => (props.src ? imageSource(props.src) : ""));

function close() {
  emit("update:modelValue", false);
}
</script>

<template>
  <div v-if="modelValue" class="image-preview" @click="close">
    <img v-if="url" :src="url" alt="" @click.stop />
    <button type="button" aria-label="关闭" @click.stop="close">×</button>
  </div>
</template>

<style scoped>
.image-preview {
  position: fixed;
  inset: 0;
  z-index: 80;
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(0, 0, 0, 0.86);
}
.image-preview img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.image-preview button {
  position: absolute;
  top: 12px;
  right: 12px;
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.16);
  color: #fff;
  font-size: 26px;
  line-height: 1;
}
</style>
