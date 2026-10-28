<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { convertFileSrc } from "@tauri-apps/api/core";
import defaultCover from "@/assets/android/drawable/image_cover_default.jpg";

const props = withDefaults(
  defineProps<{
    name: string;
    src?: string | null;
    width?: number;
    height?: number;
    radius?: number;
    fill?: boolean;
  }>(),
  {
    src: null,
    width: 66,
    height: 90,
    radius: 2,
    fill: false,
  },
);

const failed = ref(false);
const text = computed(() => props.name.trim().slice(0, 2) || "书");
const imageUrl = computed(() => {
  const src = props.src?.trim();
  if (!src) return "";
  if (/^(https?:|data:|asset:|blob:)/i.test(src)) return src;
  try {
    return convertFileSrc(src);
  } catch {
    return src;
  }
});
const showImage = computed(() => Boolean(imageUrl.value) && !failed.value);
const style = computed(() => {
  const base: Record<string, string> = {
    borderRadius: `${props.radius}px`,
    backgroundImage: `url(${defaultCover})`,
  };
  if (!props.fill) {
    base.width = `${props.width}px`;
    base.height = `${props.height}px`;
  }
  return base;
});

watch(
  () => props.src,
  () => {
    failed.value = false;
  },
);
</script>

<template>
  <div class="cover" :class="{ fill }" :style="style">
    <img
      v-if="showImage"
      class="cover-image"
      :src="imageUrl"
      :alt="name"
      @error="failed = true"
    />
    <span v-if="!showImage">{{ text }}</span>
  </div>
</template>

<style scoped>
.cover {
  position: relative;
  flex: none;
  background-size: cover;
  background-position: center;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.18);
}
.cover.fill {
  width: 100%;
  aspect-ratio: 3 / 4;
}
.cover-image {
  position: absolute;
  inset: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
}
.cover span {
  font-size: 18px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.92);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
}
</style>
