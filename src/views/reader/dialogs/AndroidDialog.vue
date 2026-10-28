<script setup lang="ts">
import { computed } from "vue";
import Icon from "@/components/android/Icon.vue";

const props = withDefaults(
  defineProps<{
    modelValue: boolean;
    title?: string;
    mode?: "sheet" | "center" | "fullscreen";
    closeOnMask?: boolean;
    showHeader?: boolean;
    dim?: boolean;
    height?: string | number;
    width?: string;
    bodyPadding?: boolean;
  }>(),
  {
    title: "",
    mode: "sheet",
    closeOnMask: true,
    showHeader: undefined,
    dim: undefined,
    height: undefined,
    width: "",
    bodyPadding: true,
  },
);

const emit = defineEmits<{ "update:modelValue": [value: boolean] }>();

const headerVisible = computed(() => props.showHeader ?? Boolean(props.title));
const dimActive = computed(() => props.dim ?? props.mode !== "sheet");
const panelStyle = computed(() => ({
  height: typeof props.height === "number" ? `${props.height}px` : props.height,
  width: props.width || undefined,
}));

function close() {
  emit("update:modelValue", false);
}
</script>

<template>
  <Teleport to="body">
    <div
      v-if="modelValue"
      class="dialog-root"
      :class="[mode, { dim: dimActive }]"
      @click.self="closeOnMask && close()"
    >
      <div class="dialog-panel" :style="panelStyle">
        <header v-if="headerVisible" class="dialog-header">
          <h2>{{ title }}</h2>
          <button aria-label="关闭" @click="close">
            <Icon name="ic_baseline_close" :size="22" />
          </button>
        </header>
        <div class="dialog-body" :class="{ bare: !bodyPadding }">
          <slot />
        </div>
        <footer v-if="$slots.footer" class="dialog-footer">
          <slot name="footer" />
        </footer>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-root {
  position: fixed;
  inset: 0;
  z-index: 60;
  display: flex;
  justify-content: center;
}
.dialog-root.dim {
  background: rgba(0, 0, 0, 0.16);
}
.dialog-root.sheet {
  align-items: flex-end;
}
.dialog-root.center {
  align-items: center;
  padding: 16px;
}
.dialog-root.center.dim {
  background: rgba(0, 0, 0, 0.4);
}
.dialog-root.fullscreen {
  align-items: stretch;
}
.dialog-panel {
  background: var(--ac-background);
  color: var(--ac-primary-text);
  display: flex;
  flex-direction: column;
  width: min(100%, 760px);
  max-height: 86vh;
  overflow: hidden;
}
.dialog-root.sheet .dialog-panel {
  width: 100%;
  animation: dialog-up 0.2s ease;
}
.dialog-root.center .dialog-panel {
  width: min(100%, 420px);
  border-radius: 2px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
}
.dialog-root.fullscreen .dialog-panel {
  width: 100%;
  height: 100%;
  max-height: none;
}
@keyframes dialog-up {
  from {
    transform: translateY(100%);
  }
  to {
    transform: translateY(0);
  }
}
.dialog-header {
  flex: none;
  display: flex;
  align-items: center;
  min-height: 52px;
  padding: 0 8px 0 20px;
  border-bottom: 1px solid var(--ac-divider);
}
.dialog-header h2 {
  flex: 1;
  margin: 0;
  font-size: 17px;
  font-weight: 500;
}
.dialog-header button {
  width: 40px;
  height: 40px;
  border: none;
  background: transparent;
  color: var(--ac-secondary-text);
  border-radius: 50%;
  cursor: pointer;
}
.dialog-header button:hover {
  background: var(--ac-transparent-10);
}
.dialog-body {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  padding: 16px 20px;
}
.dialog-body.bare {
  padding: 0;
}
.dialog-footer {
  flex: none;
  padding: 10px 20px calc(10px + env(safe-area-inset-bottom));
  border-top: 1px solid var(--ac-divider);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}
</style>
