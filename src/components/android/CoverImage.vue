<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { platform } from "@/runtime";
import defaultCover from "@/assets/android/drawable/image_cover_default.jpg";
import { useAppearanceStore } from "@/stores/appearance";

const props = withDefaults(
  defineProps<{
    name: string;
    author?: string;
    src?: string | null;
    width?: number;
    height?: number;
    radius?: number;
    fill?: boolean;
  }>(),
  {
    author: "",
    src: null,
    width: 66,
    height: 90,
    radius: 2,
    fill: false,
  },
);

const appearance = useAppearanceStore();
const failed = ref(false);

function toImageUrl(value: string | null | undefined): string {
  const src = value?.trim();
  if (!src) return "";
  if (/^(https?:|data:|asset:|blob:)/i.test(src)) return src;
  try {
    return platform.assets.fileSrc(src);
  } catch {
    return src;
  }
}

const imageUrl = computed(() => toImageUrl(props.src));
const defaultCoverUrl = computed(() => {
  const custom = appearance.activeDefaultCover;
  return custom ? toImageUrl(custom) : defaultCover;
});
const showImage = computed(() => Boolean(imageUrl.value) && !failed.value);
const showName = computed(() => appearance.coverShowNameActive && Boolean(props.name.trim()));
const showAuthor = computed(
  () => showName.value && appearance.coverShowAuthorActive && Boolean(props.author?.trim()),
);
const showText = computed(() => !showImage.value && (showName.value || showAuthor.value));
const style = computed(() => {
  const base: Record<string, string> = {
    borderRadius: `${props.radius}px`,
    backgroundImage: `url("${defaultCoverUrl.value.replace(/"/g, "%22")}")`,
  };
  if (!props.fill) {
    base.width = `${props.width}px`;
    base.height = `${props.height}px`;
  }
  return base;
});

watch(
  () => [props.src, defaultCoverUrl.value],
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
    <span v-if="showText" class="cover-text">
      <span v-if="showName" class="cover-name">{{ name }}</span>
      <span v-if="showAuthor" class="cover-author">{{ author }}</span>
    </span>
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
.cover-text {
  position: relative;
  z-index: 1;
  max-width: 86%;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  text-align: center;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
}
.cover-name {
  font-size: 14px;
  font-weight: 600;
  line-height: 1.25;
  color: rgba(255, 255, 255, 0.94);
  display: -webkit-box;
  -webkit-line-clamp: 3;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.cover-author {
  font-size: 10px;
  line-height: 1.2;
  color: rgba(255, 255, 255, 0.82);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
