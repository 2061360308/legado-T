<script setup lang="ts">
import Icon from "@/components/android/Icon.vue";

defineProps<{
  mode: "stop" | "continue";
}>();

defineEmits<{
  stop: [];
  continue: [];
}>();
</script>

<template>
  <button
    class="fab"
    :aria-label="mode === 'stop' ? '停止' : '继续搜索'"
    @click="mode === 'stop' ? $emit('stop') : $emit('continue')"
  >
    <Icon :name="mode === 'stop' ? 'ic_stop_black_24dp' : 'ic_play_24dp'" :size="22" />
  </button>
</template>

<style scoped>
.fab {
  position: absolute;
  right: calc(16px + var(--safe-right, 0px));
  bottom: calc(16px + var(--safe-bottom, 0px));
  z-index: 8;
  width: 40px;
  height: 40px;
  border: none;
  border-radius: 50%;
  background: var(--ac-primary);
  color: #fff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  box-shadow: 0 3px 8px rgba(0, 0, 0, 0.28);
  cursor: pointer;
}
.fab:active {
  filter: brightness(0.9);
}
</style>
