<script setup lang="ts">
import { ref } from "vue";
import Icon from "./Icon.vue";

withDefaults(
  defineProps<{
    modelValue: string;
    placeholder?: string;
    icon?: string;
    submitButton?: boolean;
  }>(),
  {
    placeholder: "搜索",
    icon: "ic_search_hint",
    submitButton: false,
  },
);

const emit = defineEmits<{
  "update:modelValue": [value: string];
  submit: [];
  focus: [];
  blur: [];
}>();

const inputRef = ref<HTMLInputElement | null>(null);

function focus() {
  inputRef.value?.focus();
}

function blur() {
  inputRef.value?.blur();
}

defineExpose({ focus, blur });
</script>

<template>
  <label class="search-bar">
    <Icon v-if="icon" :name="icon" :size="18" class="search-icon" />
    <input
      ref="inputRef"
      :value="modelValue"
      type="search"
      :placeholder="placeholder"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
      @keydown.enter="emit('submit')"
      @focus="emit('focus')"
      @blur="emit('blur')"
    />
    <button v-if="submitButton" type="button" class="submit" aria-label="搜索" @click="emit('submit')">
      <Icon name="ic_search" :size="20" />
    </button>
  </label>
</template>

<style scoped>
.search-bar {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  height: 30px;
  margin: 10px 0;
  padding: 0 10px;
  border: 0.5px solid var(--ac-transparent-10);
  border-radius: 35px;
  background: var(--ac-transparent-10);
  color: inherit;
  box-sizing: border-box;
}
.search-icon {
  flex: none;
  margin-right: 4px;
  opacity: 0.72;
}
input {
  flex: 1;
  min-width: 0;
  border: none;
  outline: none;
  background: transparent;
  color: inherit;
  font-size: 13px;
  padding: 0;
}
input::placeholder {
  color: inherit;
  opacity: 0.65;
}
.submit {
  flex: none;
  width: 28px;
  height: 28px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: inherit;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}
.submit:hover {
  background: var(--ac-transparent-10);
}
</style>
