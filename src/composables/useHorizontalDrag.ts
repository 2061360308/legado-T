import { ref } from "vue";

export function useHorizontalDrag(onDragStart?: () => void) {
  const dragging = ref(false);
  let start: { x: number; y: number; left: number } | null = null;
  let suppressClickUntil = 0;

  function onPointerDown(event: PointerEvent) {
    if (event.pointerType === "mouse" && event.button !== 0) return;
    const element = event.currentTarget as HTMLElement | null;
    if (!element) return;
    start = { x: event.clientX, y: event.clientY, left: element.scrollLeft };
    dragging.value = false;
  }

  function onPointerMove(event: PointerEvent) {
    if (!start) return;
    const element = event.currentTarget as HTMLElement | null;
    if (!element) return;
    const dx = event.clientX - start.x;
    const dy = event.clientY - start.y;
    if (!dragging.value) {
      if (Math.abs(dx) < 6) return;
      if (Math.abs(dy) > Math.abs(dx)) {
        start = null;
        return;
      }
      dragging.value = true;
      onDragStart?.();
      try {
        element.setPointerCapture?.(event.pointerId);
      } catch {
        // ignore unsupported pointer capture
      }
    }
    event.preventDefault();
    element.scrollLeft = start.left - dx;
  }

  function finish(event: PointerEvent) {
    const element = event.currentTarget as HTMLElement | null;
    if (dragging.value && element) {
      try {
        element.releasePointerCapture?.(event.pointerId);
      } catch {
        // ignore
      }
      suppressClickUntil = performance.now() + 80;
    }
    dragging.value = false;
    start = null;
  }

  function onClickCapture(event: MouseEvent) {
    if (performance.now() >= suppressClickUntil) return;
    event.stopPropagation();
    event.preventDefault();
  }

  function wasDragged(): boolean {
    return performance.now() < suppressClickUntil;
  }

  return {
    dragging,
    onPointerDown,
    onPointerMove,
    onPointerUp: finish,
    onPointerCancel: finish,
    onClickCapture,
    wasDragged,
  };
}
