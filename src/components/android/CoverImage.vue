<script setup lang="ts">
import { computed } from "vue";
import defaultCover from "@/assets/android/drawable/image_cover_default.jpg";

const props = withDefaults(
  defineProps<{
    name: string;
    width?: number;
    height?: number;
    radius?: number;
    fill?: boolean;
  }>(),
  {
    width: 66,
    height: 90,
    radius: 2,
    fill: false,
  },
);

const text = computed(() => props.name.trim().slice(0, 2) || "书");
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
</script>

<template>
  <div class="cover" :class="{ fill }" :style="style">
    <span>{{ text }}</span>
  </div>
</template>

<style scoped>
.cover {
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
.cover span {
  font-size: 18px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.92);
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.45);
}
</style>
