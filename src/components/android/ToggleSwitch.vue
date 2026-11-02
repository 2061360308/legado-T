<script setup lang="ts">
withDefaults(
  defineProps<{
    checked: boolean;
    disabled?: boolean;
  }>(),
  {
    disabled: false,
  },
);

const emit = defineEmits<{ change: [value: boolean] }>();
</script>

<template>
  <label class="switch" :class="{ disabled }">
    <input
      type="checkbox"
      :checked="checked"
      :disabled="disabled"
      @change="emit('change', ($event.target as HTMLInputElement).checked)"
    />
    <span class="slider" />
  </label>
</template>

<style scoped>
.switch {
  position: relative;
  width: 36px;
  height: 20px;
  flex: none;
  cursor: pointer;
}
.switch.disabled {
  cursor: default;
  opacity: 0.45;
}
.switch input {
  position: absolute;
  inset: 0;
  margin: 0;
  opacity: 0;
  cursor: inherit;
}
.slider {
  position: absolute;
  inset: 0;
  border-radius: 10px;
  background: var(--ac-divider);
  transition: background 0.18s ease;
}
.slider::after {
  content: "";
  position: absolute;
  width: 16px;
  height: 16px;
  left: 2px;
  top: 2px;
  border-radius: 50%;
  background: #fff;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
  transition: transform 0.18s ease;
}
input:checked + .slider {
  background: var(--ac-primary);
}
input:checked + .slider::after {
  transform: translateX(16px);
}
</style>
