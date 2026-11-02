<script setup lang="ts">
withDefaults(
  defineProps<{
    active?: boolean;
  }>(),
  {
    active: false,
  },
);
</script>

<template>
  <div v-if="active" class="progress" aria-hidden="true">
    <span />
  </div>
</template>

<style scoped>
.progress {
  flex: none;
  height: 2px;
  overflow: hidden;
  background: var(--ac-transparent-10);
}
.progress span {
  display: block;
  width: 38%;
  height: 100%;
  background: var(--ac-primary);
  animation: slide 1.1s ease-in-out infinite;
}
@keyframes slide {
  0% {
    transform: translateX(-120%);
  }
  100% {
    transform: translateX(360%);
  }
}
</style>
