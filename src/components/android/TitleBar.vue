<script setup lang="ts">
import Icon from "./Icon.vue";

withDefaults(
  defineProps<{
    title?: string;
    subtitle?: string;
    back?: boolean;
    flat?: boolean;
    dark?: boolean;
    safeTop?: boolean;
    searchOnly?: boolean;
  }>(),
  {
    title: "",
    subtitle: "",
    back: true,
    flat: false,
    dark: false,
    safeTop: true,
    searchOnly: false,
  },
);

defineEmits<{ back: [] }>();
</script>

<template>
  <header
    class="title-bar"
    :class="{ flat, dark, 'search-only': searchOnly, 'with-nav': back }"
  >
    <div v-if="safeTop" class="status-bar-spacer" aria-hidden="true" />
    <div class="bar">
      <button v-if="back" class="nav-button" aria-label="返回" @click="$emit('back')">
        <Icon name="ic_arrow_back" :size="24" />
      </button>
      <slot name="navigation" />
      <div v-if="title || subtitle" class="titles">
        <h1>{{ title }}</h1>
        <p v-if="subtitle">{{ subtitle }}</p>
      </div>
      <slot name="content" />
      <div class="actions">
        <slot name="actions" />
      </div>
    </div>
    <div v-if="$slots.below" class="below">
      <slot name="below" />
    </div>
  </header>
</template>

<style scoped>
.title-bar {
  display: flex;
  flex-direction: column;
  background: var(--ac-primary);
  color: #ffffff;
  box-shadow: 0 var(--title-bar-elevation, 4px) var(--title-bar-elevation, 4px) rgba(0, 0, 0, 0.12);
  position: relative;
  z-index: 5;
}
.status-bar-spacer {
  flex: none;
  height: var(--safe-top, 0px);
  background: inherit;
}
.title-bar.flat {
  background: var(--ac-background);
  color: var(--ac-primary-text);
  box-shadow: 0 1px 0 var(--ac-divider);
}
.title-bar.dark {
  background: var(--ac-background-menu);
  color: var(--ac-primary-text);
}
.bar {
  display: flex;
  align-items: center;
  min-height: 56px;
  padding: 0 calc(4px + var(--safe-right, 0px)) 0 calc(4px + var(--safe-left, 0px));
}
.title-bar.search-only:not(.with-nav) .bar {
  padding-left: calc(16px + var(--safe-left, 0px));
}
.nav-button,
.actions :deep(button) {
  width: 44px;
  height: 44px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: inherit;
  border-radius: 50%;
  cursor: pointer;
}
.nav-button:hover,
.actions :deep(button:hover) {
  background: rgba(255, 255, 255, 0.14);
}
.title-bar.flat .nav-button:hover,
.title-bar.dark .nav-button:hover,
.title-bar.flat .actions :deep(button:hover),
.title-bar.dark .actions :deep(button:hover) {
  background: var(--ac-transparent-10);
}
.titles {
  min-width: 0;
  flex: 1;
  padding: 0 8px;
}
.titles h1 {
  margin: 0;
  font-size: 20px;
  font-weight: 500;
  line-height: 1.2;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.titles p {
  margin: 2px 0 0;
  font-size: 12px;
  opacity: 0.8;
}
.actions {
  display: flex;
  align-items: center;
}
.below {
  min-width: 0;
}
</style>
