import {
  nextTick,
  onActivated,
  onBeforeUnmount,
  onDeactivated,
  onMounted,
  type Ref,
} from "vue";
import { useRoute } from "vue-router";

const positions = new Map<string, number>();

export function useScrollMemory(
  element: Ref<HTMLElement | null>,
  key?: () => string,
) {
  const route = useRoute();
  const resolveKey = () => key?.() ?? route.fullPath;

  function save() {
    const el = element.value;
    if (!el) return;
    positions.set(resolveKey(), el.scrollTop);
  }

  async function restore() {
    const target = positions.get(resolveKey()) ?? 0;
    await nextTick();
    requestAnimationFrame(() => {
      const el = element.value;
      if (!el) return;
      el.scrollTop = Math.min(target, Math.max(0, el.scrollHeight - el.clientHeight));
    });
  }

  onMounted(restore);
  onActivated(restore);
  onDeactivated(save);
  onBeforeUnmount(save);

  return { save, restore };
}
