<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from "vue";
import { useRouter } from "vue-router";
import { platform } from "@/runtime";
import { useAppearanceStore } from "@/stores/appearance";
import iconBook from "@/assets/welcome/icon_read_book.png";

const router = useRouter();
const appearance = useAppearanceStore();
let timer: number | undefined;

const customActive = computed(
  () => appearance.customWelcome && Boolean(appearance.activeWelcomeImage),
);
const showText = computed(() =>
  customActive.value ? appearance.welcomeShowTextActive : true,
);
const showIcon = computed(() =>
  customActive.value ? appearance.welcomeShowIconActive : true,
);
const backgroundImage = computed(() => {
  if (!customActive.value) return "";
  const path = appearance.activeWelcomeImage;
  if (!path) return "";
  if (/^(https?:|data:|asset:|blob:)/i.test(path)) return path;
  try {
    return platform.assets.fileSrc(path);
  } catch {
    return path;
  }
});
const pageStyle = computed(() =>
  backgroundImage.value
    ? {
        backgroundImage: `url("${backgroundImage.value.replace(/"/g, "%22")}")`,
      }
    : undefined,
);

onMounted(() => {
  timer = window.setTimeout(() => {
    router.replace("/");
  }, 600);
});

onBeforeUnmount(() => window.clearTimeout(timer));
</script>

<template>
  <div class="welcome-page" :class="{ 'has-image': Boolean(backgroundImage) }" :style="pageStyle">
    <div class="welcome-content">
      <div v-if="showText" class="title-block">
        <span class="title-line" />
        <div class="title-text">
          <span class="title">阅读</span>
          <span class="subtitle">享受美好时光</span>
        </div>
      </div>
      <span
        v-if="showIcon"
        class="book-icon"
        :style="{ maskImage: `url(${iconBook})`, WebkitMaskImage: `url(${iconBook})` }"
        aria-hidden="true"
      />
      <p v-if="showText" class="footer-text">
        关注公众号[开源阅读]<br />
        看文章点广告支持作者
      </p>
    </div>
  </div>
</template>

<style scoped>
.welcome-page {
  height: 100%;
  width: 100%;
  overflow: hidden;
  background-color: var(--ac-background);
  background-position: center;
  background-size: cover;
  background-repeat: no-repeat;
  color: var(--ac-accent);
  display: flex;
  align-items: center;
  justify-content: center;
}
.welcome-page.has-image {
  color: #ffffff;
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.5);
}
.welcome-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: calc(24px + var(--safe-top, 0px)) 24px calc(24px + var(--safe-bottom, 0px));
  box-sizing: border-box;
}
.title-block {
  display: flex;
  align-items: stretch;
  gap: 8px;
}
.title-line {
  width: 6px;
  flex: none;
  background: currentColor;
}
.title-text {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
}
.title {
  font-size: 49px;
  line-height: 1;
  font-weight: 400;
}
.subtitle {
  margin-top: 60px;
  font-size: 16px;
  line-height: 1.2;
}
.book-icon {
  width: 120px;
  height: 120px;
  margin-top: 32px;
  background-color: currentColor;
  mask-position: center;
  mask-repeat: no-repeat;
  mask-size: contain;
  -webkit-mask-position: center;
  -webkit-mask-repeat: no-repeat;
  -webkit-mask-size: contain;
  opacity: 0.94;
}
.footer-text {
  margin: 32px 0 0;
  font-size: 14px;
  line-height: 1.5;
  text-align: center;
  white-space: normal;
}
</style>
