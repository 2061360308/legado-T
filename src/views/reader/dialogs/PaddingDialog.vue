<script setup lang="ts">
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import AndroidDialog from "./AndroidDialog.vue";
import DetailSeekBar from "@/components/android/DetailSeekBar.vue";

defineProps<{ modelValue: boolean }>();
const emit = defineEmits<{
  "update:modelValue": [value: boolean];
  apply: [];
}>();

const prefs = useReaderPrefsStore();

function update(patch: Parameters<typeof prefs.patch>[0]) {
  prefs.patch(patch);
  emit("apply");
}
</script>

<template>
  <AndroidDialog
    :model-value="modelValue"
    mode="center"
    title="边距"
    width="min(92vw, 520px)"
    :body-padding="false"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <div class="padding-panel">
      <section class="block">
        <h3>页眉</h3>
        <label class="line-row">
          <span>显示分隔线</span>
          <input
            type="checkbox"
            :checked="prefs.showHeaderLine"
            @change="update({ showHeaderLine: ($event.target as HTMLInputElement).checked })"
          />
        </label>
        <DetailSeekBar
          title="上边距"
          :model-value="prefs.headerPaddingTop"
          :max="100"
          @update:model-value="(value) => update({ headerPaddingTop: value })"
        />
        <DetailSeekBar
          title="下边距"
          :model-value="prefs.headerPaddingBottom"
          :max="100"
          @update:model-value="(value) => update({ headerPaddingBottom: value })"
        />
        <DetailSeekBar
          title="左边距"
          :model-value="prefs.headerPaddingLeft"
          :max="100"
          @update:model-value="(value) => update({ headerPaddingLeft: value })"
        />
        <DetailSeekBar
          title="右边距"
          :model-value="prefs.headerPaddingRight"
          :max="100"
          @update:model-value="(value) => update({ headerPaddingRight: value })"
        />
      </section>

      <section class="block">
        <h3>正文</h3>
        <DetailSeekBar
          title="上边距"
          :model-value="prefs.paddingTop"
          :max="200"
          @update:model-value="(value) => update({ paddingTop: value })"
        />
        <DetailSeekBar
          title="下边距"
          :model-value="prefs.paddingBottom"
          :max="100"
          @update:model-value="(value) => update({ paddingBottom: value })"
        />
        <DetailSeekBar
          title="左边距"
          :model-value="prefs.paddingLeft"
          :max="100"
          @update:model-value="(value) => update({ paddingLeft: value })"
        />
        <DetailSeekBar
          title="右边距"
          :model-value="prefs.paddingRight"
          :max="100"
          @update:model-value="(value) => update({ paddingRight: value })"
        />
      </section>

      <section class="block">
        <h3>页脚</h3>
        <label class="line-row">
          <span>显示分隔线</span>
          <input
            type="checkbox"
            :checked="prefs.showFooterLine"
            @change="update({ showFooterLine: ($event.target as HTMLInputElement).checked })"
          />
        </label>
        <DetailSeekBar
          title="上边距"
          :model-value="prefs.footerPaddingTop"
          :max="100"
          @update:model-value="(value) => update({ footerPaddingTop: value })"
        />
        <DetailSeekBar
          title="下边距"
          :model-value="prefs.footerPaddingBottom"
          :max="100"
          @update:model-value="(value) => update({ footerPaddingBottom: value })"
        />
        <DetailSeekBar
          title="左边距"
          :model-value="prefs.footerPaddingLeft"
          :max="100"
          @update:model-value="(value) => update({ footerPaddingLeft: value })"
        />
        <DetailSeekBar
          title="右边距"
          :model-value="prefs.footerPaddingRight"
          :max="100"
          @update:model-value="(value) => update({ footerPaddingRight: value })"
        />
      </section>
    </div>
  </AndroidDialog>
</template>

<style scoped>
.padding-panel {
  padding: 6px 0 12px;
}
.block {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--ac-divider);
}
.block:last-child {
  border-bottom: none;
}
h3 {
  margin: 8px 16px 4px;
  font-size: 13px;
  color: var(--ac-primary);
}
.line-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 6px 16px;
  font-size: 13px;
  color: var(--ac-primary-text);
}
.line-row input {
  width: 18px;
  height: 18px;
  accent-color: var(--ac-primary);
}
</style>
