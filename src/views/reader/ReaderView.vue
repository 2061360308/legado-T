<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import type { LayoutPage, ReaderSnapshot } from "@/api/bindings";
import { readerOpen, readerSaveProgress } from "@/api/reader";
import { bookmarkAdd } from "@/api/bookmarks";
import { useReaderPrefsStore } from "@/stores/readerPrefs";
import { useThemeStore } from "@/stores/theme";
import ReaderCanvas from "./canvas/ReaderCanvas.vue";
import PageTurnOverlay from "./canvas/PageTurnOverlay.vue";
import SelectionHighlight from "./selection/SelectionHighlight.vue";
import {
  buildSelection,
  hitTestText,
  selectionBounds,
  selectionHandles as selectionHandlesFor,
  wordSelection,
  type SelectionPos,
  type TextSelection,
} from "./selection/textSelection";
import ReadAloudPanel from "./config/ReadAloudPanel.vue";
import DictDialog from "../dict/DictDialog.vue";
import ReadStyleDialog from "./dialogs/ReadStyleDialog.vue";
import BgTextDialog from "./dialogs/BgTextDialog.vue";
import PaddingDialog from "./dialogs/PaddingDialog.vue";
import AutoReadDialog from "./dialogs/AutoReadDialog.vue";
import ReadAloudConfigDialog from "./dialogs/ReadAloudConfigDialog.vue";
import SpeakEngineDialog from "./dialogs/SpeakEngineDialog.vue";
import ClickActionDialog from "./dialogs/ClickActionDialog.vue";
import PageKeyDialog from "./dialogs/PageKeyDialog.vue";
import MoreConfigDialog from "./dialogs/MoreConfigDialog.vue";
import SimulatedReadingDialog from "./dialogs/SimulatedReadingDialog.vue";
import EffectiveReplacesDialog from "./dialogs/EffectiveReplacesDialog.vue";
import ContentEditDialog from "./dialogs/ContentEditDialog.vue";
import FontSelectDialog from "./dialogs/FontSelectDialog.vue";
import {
  buildInfoBarStyle,
  measureInfoBarHeights,
  type InfoBarContext,
  type InfoBarPayload,
} from "./infoBar";
import TitleBar from "@/components/android/TitleBar.vue";
import Icon from "@/components/android/Icon.vue";
import PopupMenu, { type PopupMenuItem } from "@/components/android/PopupMenu.vue";
import { ThemeMode } from "@/stores/theme";

const route = useRoute();
const router = useRouter();
const prefs = useReaderPrefsStore();
const theme = useThemeStore();

const bookUrl = computed(() => String(route.query.bookUrl || ""));
const snapshot = ref<ReaderSnapshot | null>(null);
const loading = ref(false);
const error = ref("");
type DialogName =
  | ""
  | "style"
  | "bg"
  | "padding"
  | "auto"
  | "aloudConfig"
  | "engine"
  | "click"
  | "pageKey"
  | "more"
  | "simulated"
  | "effective"
  | "contentEdit"
  | "fonts";

const menuVisible = ref(false);
const activeDialog = ref<DialogName>("");
const moreMenuVisible = ref(false);
const isOnline = computed(() => /^https?:/i.test(snapshot.value?.book.bookUrl ?? ""));
const isTxt = computed(() => /\.txt$/i.test(snapshot.value?.book.bookUrl ?? ""));
const isEpub = computed(() => /\.epub$/i.test(snapshot.value?.book.bookUrl ?? ""));

// book_read.xml：never 项按原版顺序；反转内容(在线)/删除ruby、删除h(epub) 为动态组
const moreMenuItems = computed<PopupMenuItem[]>(() => {
  const items: PopupMenuItem[] = [
    { key: "bookmark", label: "添加书签" },
    { key: "edit", label: "编辑内容" },
    { key: "pageAnim", label: "翻页动画（本书）" },
  ];
  if (isOnline.value) items.push({ key: "reverse", label: "反转内容" });
  items.push({ key: "simulated", label: "模拟追读" });
  items.push({ key: "replace", label: "替换净化", icon: "ic_find_replace" });
  items.push({ key: "sameTitleRemoved", label: "移除重复标题" });
  items.push({ key: "reSegment", label: "重新分段" });
  if (isEpub.value) {
    items.push({ key: "delRuby", label: "删除ruby标签" });
    items.push({ key: "delH", label: "删除h标签" });
  }
  items.push({ key: "imageStyle", label: "图片样式" });
  items.push({ key: "updateToc", label: "更新目录" });
  items.push({ key: "effective", label: "起效的替换" });
  items.push({ key: "log", label: "日志", muted: true });
  items.push({ key: "help", label: "帮助" });
  return items;
});
const dictVisible = ref(false);
const dictWord = ref("");
const selection = ref<TextSelection | null>(null);
const selectionMenuVisible = ref(false);
let selectionHandle: "start" | "end" | null = null;
let selectionDragging = false;
let longPressTimer: number | undefined;
let longPressStart: { x: number; y: number } | null = null;
const selectionHandles = computed(() => {
  const page = currentPage.value;
  const value = selection.value;
  if (!page || !value) return null;
  return selectionHandlesFor(page, value);
});
const selectionMenuStyle = computed(() => {
  const page = currentPage.value;
  const value = selection.value;
  if (!page || !value) return {};
  const box = selectionBounds(page, value);
  if (!box) return {};
  const menuWidth = 236;
  const menuHeight = 44;
  const left = Math.min(
    Math.max((box.left + box.right) / 2 - menuWidth / 2, 8),
    Math.max(8, viewWidth.value - menuWidth - 8),
  );
  const below = box.bottom + 12;
  const top =
    below + menuHeight <= viewHeight.value
      ? below
      : Math.max(8, box.top - menuHeight - 12);
  return { left: `${left}px`, top: `${top}px` };
});
const selectionHandleStyle = computed(() => {
  const page = currentPage.value;
  const value = selection.value;
  if (!page || !value) return null;
  const handles = selectionHandlesFor(page, value);
  if (!handles) return null;
  return {
    start: {
      left: `${handles.start.x - 7}px`,
      top: `${handles.start.y - 7}px`,
    },
    end: {
      left: `${handles.end.x - 7}px`,
      top: `${handles.end.y - 7}px`,
    },
  };
});
const aloudVisible = ref(false);
const aloudPosition = ref(-1);
const brightness = ref(1);
const brightnessSide = ref<"left" | "right">("left");
const viewWidth = ref(360);
const viewHeight = ref(640);
const toast = ref("");
const animDir = ref<"next" | "prev">("next");
let toastTimer: number | undefined;
let resizeTimer: number | undefined;
let prefsTimer: number | undefined;
let clickTimer: number | undefined;
let autoTimer: number | undefined;
let wheelNextTimer: number | undefined;
let wheelPrevTimer: number | undefined;
let lastTurnStartedAt = 0;

const currentPage = computed(() => {
  const snap = snapshot.value;
  if (!snap) return null;
  return snap.pages[snap.pageIndex] ?? null;
});

const anyDialogOpen = computed(() => activeDialog.value !== "");


const useTurnOverlay = computed(
  () => !theme.isEink && (prefs.pageAnim === 0 || prefs.pageAnim === 2),
);
const turnMode = computed<"cover" | "simulation">(() =>
  prefs.pageAnim === 2 ? "simulation" : "cover",
);
const slideMode = computed(() => !theme.isEink && prefs.pageAnim === 1);
const transitionName = computed(() => {
  if (!slideMode.value) return "";
  return `page-slide-${animDir.value}`;
});

const highlightLine = computed(() => {
  const page = currentPage.value;
  if (!page || aloudPosition.value < 0) return -1;
  return page.lines.findIndex(
    (line) => aloudPosition.value >= line.startChar && aloudPosition.value < line.endChar,
  );
});

function withAlpha(color: string, alpha: number) {
  if (alpha >= 255) return color;
  const hex = color.replace("#", "");
  if (hex.length !== 6) return color;
  const r = parseInt(hex.slice(0, 2), 16);
  const g = parseInt(hex.slice(2, 4), 16);
  const b = parseInt(hex.slice(4, 6), 16);
  return `rgba(${r}, ${g}, ${b}, ${(alpha / 255).toFixed(3)})`;
}
const readerBackground = computed(() =>
  withAlpha(theme.readerColors.bgValue, theme.readerColors.bgAlpha),
);
const readerTextColor = computed(() => theme.readerColors.textColor);
const readerImageUrl = computed(() => theme.readerColors.bgImage ?? "");
const readerImage = ref<HTMLImageElement | null>(null);
const readerBackgroundAlpha = computed(() => theme.readerColors.bgAlpha / 255);

watch(
  readerImageUrl,
  (url) => {
    if (!url) {
      readerImage.value = null;
      return;
    }
    const image = new Image();
    image.onload = () => {
      readerImage.value = image;
    };
    image.onerror = () => {
      readerImage.value = null;
    };
    image.src = url;
  },
  { immediate: true },
);

const now = ref(new Date());
const batteryLevel = ref<number | null>(null);
let clockTimer: number | undefined;

const clock = computed(() =>
  now.value.toLocaleTimeString("zh-CN", { hour: "2-digit", minute: "2-digit" }),
);

function currentDividerColor(): string {
  void theme.mode;
  void theme.currentScheme;
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue("--ac-divider")
    .trim();
  return value || "rgba(0,0,0,0.12)";
}

const infoBarStyle = computed(() =>
  buildInfoBarStyle(prefs, {
    textColor: readerTextColor.value,
    dividerColor: currentDividerColor(),
  }),
);

function infoPayloadFor(page: LayoutPage | null): InfoBarPayload | null {
  const snap = snapshot.value;
  if (!snap || !page) return null;
  const context: InfoBarContext = {
    bookName: snap.book.name,
    chapterTitle: page.chapterTitle || snap.chapterTitle,
    pageIndex: page.index,
    pageSize: snap.pages.length,
    chapterIndex: snap.chapterIndex,
    chapterSize: snap.totalChapters,
    time: clock.value,
    battery: batteryLevel.value,
  };
  return { style: infoBarStyle.value, context };
}

const currentInfoBar = computed(() =>
  infoPayloadFor(currentPage.value),
);

const turnFromInfoBar = ref<InfoBarPayload | null>(null);
const turnToInfoBar = ref<InfoBarPayload | null>(null);

async function loadBattery() {
  const nav = navigator as Navigator & {
    getBattery?: () => Promise<{ level: number }>;
  };
  try {
    const manager = await nav.getBattery?.();
    if (manager) batteryLevel.value = Math.round(manager.level * 100);
  } catch {
    batteryLevel.value = null;
  }
}
const readerUnderline = computed(() => theme.readerColors.underline);
const brightnessStyle = computed(() => ({ filter: `brightness(${brightness.value})` }));

function showToast(message: string) {
  toast.value = message;
  window.clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => (toast.value = ""), 1600);
}

function buildStyle() {
  const infoBarHeights = measureInfoBarHeights(infoBarStyle.value);
  return {
    fontSize: prefs.fontSize,
    titleFontSize: prefs.fontSize + prefs.titleSize,
    titleBold: prefs.titleBold,
    titleMode: prefs.titleMode,
    titleTopSpacing: prefs.titleTopSpacing,
    titleBottomSpacing: prefs.titleBottomSpacing,
    lineSpacing: prefs.lineSpacing,
    paragraphSpacing: (prefs.paragraphSpacing * prefs.fontSize) / 10,
    textBold: prefs.textBold,
    letterSpacing: prefs.letterSpacing,
    paragraphIndent: prefs.paragraphIndent,
    paddingLeft: prefs.paddingLeft,
    paddingRight: prefs.paddingRight,
    paddingTop: prefs.paddingTop + infoBarHeights.header,
    paddingBottom: prefs.paddingBottom + infoBarHeights.footer,
    fullJustify: prefs.fullJustify,
    bottomJustify: prefs.bottomJustify,
    useZhLayout: prefs.useZhLayout,
    viewWidth: viewWidth.value,
    viewHeight: viewHeight.value,
    imageStyle: "DEFAULT",
  };
}

function measureViewport() {
  viewWidth.value = Math.max(240, window.innerWidth);
  viewHeight.value = Math.max(240, window.innerHeight);
}

async function open(chapterIndex: number | null = null, chapterPos: number | null = null) {
  clearSelection();
  if (!bookUrl.value) {
    error.value = "缺少书籍地址";
    return;
  }
  loading.value = true;
  error.value = "";
  try {
    snapshot.value = await readerOpen(bookUrl.value, chapterIndex, chapterPos, buildStyle());
    if (snapshot.value.marginFellBack) {
      showToast("边距过大，已回退默认边距");
    }
    persistProgress();
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

function persistProgress() {
  const snap = snapshot.value;
  if (!snap) return;
  const page = snap.pages[snap.pageIndex];
  const pos = page ? page.startChar : snap.pageStartChar;
  readerSaveProgress(bookUrl.value, snap.chapterIndex, pos, snap.chapterTitle).catch(
    () => undefined,
  );
}

const turnActive = ref(false);
const turnDirection = ref<"next" | "prev">("next");
const turnFromPage = ref<LayoutPage | null>(null);
const turnToPage = ref<LayoutPage | null>(null);
const turnOverlay = ref<InstanceType<typeof PageTurnOverlay> | null>(null);
const canvasWrapRef = ref<HTMLElement | null>(null);
let turnApply: (() => void) | null = null;
let dragDirection: "next" | "prev" | null = null;
let dragBoundary: "next" | "prev" | null = null;
let dragStartPoint: { x: number; y: number } | null = null;
let lastDownPoint = { x: 0, y: 0 };
let pointerOutside = false;
let turnWatchdog: number | undefined;

function clearTurnWatchdog() {
  if (turnWatchdog) {
    window.clearTimeout(turnWatchdog);
    turnWatchdog = undefined;
  }
}

function armTurnWatchdog(ms: number) {
  clearTurnWatchdog();
  turnWatchdog = window.setTimeout(() => {
    if (turnActive.value) onTurnDone(false);
  }, ms);
}

function startTurn(
  direction: "next" | "prev",
  from: LayoutPage | null,
  to: LayoutPage | null,
  apply: (() => void) | null,
  gesture?: { downX: number; downY: number; startX: number; startY: number },
  speed = 300,
) {
  if (turnActive.value || !from) return;
  clearSelection();
  turnDirection.value = direction;
  turnFromPage.value = from;
  turnToPage.value = to;
  turnApply = apply;
  turnFromInfoBar.value = infoPayloadFor(from);
  turnToInfoBar.value = infoPayloadFor(to);
  turnActive.value = true;
  lastTurnStartedAt = performance.now();
  if (!gesture) armTurnWatchdog(1200);
  nextTick(() => {
    const overlay = turnOverlay.value;
    if (!overlay) {
      onTurnDone(true);
      return;
    }
    if (gesture) {
      overlay.beginDrag(direction, gesture.downX, gesture.downY, gesture.startX, gesture.startY);
    } else {
      overlay.autoFrom(direction, lastDownPoint.x, lastDownPoint.y, speed);
    }
  });
}

function onTurnDone(completed: boolean) {
  const apply = turnApply;
  turnApply = null;
  clearTurnWatchdog();
  try {
    if (completed && apply) apply();
  } finally {
    nextTick(() => {
      turnActive.value = false;
      turnFromPage.value = null;
      turnToPage.value = null;
      turnFromInfoBar.value = null;
      turnToInfoBar.value = null;
    });
  }
}

function startChapterTurn(direction: "next" | "prev", speed = 300) {
  const snap = snapshot.value;
  if (!snap || turnActive.value) return;
  const target = snap.chapterIndex + (direction === "next" ? 1 : -1);
  if (target < 0 || target >= snap.totalChapters) {
    showToast(direction === "next" ? "已经是最后一章" : "已经是第一章");
    return;
  }
  const from = currentPage.value;
  if (!from) return;
  clearSelection();
  turnDirection.value = direction;
  turnFromPage.value = from;
  turnToPage.value = null;
  turnFromInfoBar.value = infoPayloadFor(from);
  turnToInfoBar.value = null;
  turnApply = () => {};
  turnActive.value = true;
  lastTurnStartedAt = performance.now();
  armTurnWatchdog(15000);
  nextTick(async () => {
    turnOverlay.value?.showStatic();
    await open(target, direction === "next" ? 0 : 2147483647);
    const to = currentPage.value;
    if (error.value || !to) {
      onTurnDone(false);
      return;
    }
    turnToPage.value = to;
    turnToInfoBar.value = infoPayloadFor(to);
    await nextTick();
    armTurnWatchdog(1500);
    turnOverlay.value?.autoFrom(direction, lastDownPoint.x, lastDownPoint.y, speed);
  });
}

function nextPage(speed = 300) {
  const snap = snapshot.value;
  if (!snap) return;
  if (useTurnOverlay.value) {
    if (turnActive.value) return;
    if (snap.pageIndex + 1 < snap.pages.length) {
      const from = currentPage.value;
      const to = snap.pages[snap.pageIndex + 1];
      startTurn("next", from, to, () => {
        snap.pageIndex += 1;
        persistProgress();
      }, undefined, speed);
      return;
    }
    if (snap.chapterIndex + 1 < snap.totalChapters) {
      startChapterTurn("next", speed);
      return;
    }
    showToast("已经是最后一页");
    return;
  }
  lastTurnStartedAt = performance.now();
  animDir.value = "next";
  if (snap.pageIndex + 1 < snap.pages.length) {
    snap.pageIndex += 1;
    persistProgress();
    return;
  }
  if (snap.chapterIndex + 1 < snap.totalChapters) {
    open(snap.chapterIndex + 1, 0);
    return;
  }
  showToast("已经是最后一页");
}

function prevPage(speed = 300) {
  const snap = snapshot.value;
  if (!snap) return;
  if (useTurnOverlay.value) {
    if (turnActive.value) return;
    if (snap.pageIndex > 0) {
      const from = currentPage.value;
      const to = snap.pages[snap.pageIndex - 1];
      startTurn("prev", from, to, () => {
        snap.pageIndex -= 1;
        persistProgress();
      }, undefined, speed);
      return;
    }
    if (snap.chapterIndex > 0) {
      startChapterTurn("prev", speed);
      return;
    }
    showToast("已经是第一页");
    return;
  }
  lastTurnStartedAt = performance.now();
  animDir.value = "prev";
  if (snap.pageIndex > 0) {
    snap.pageIndex -= 1;
    persistProgress();
    return;
  }
  if (snap.chapterIndex > 0) {
    open(snap.chapterIndex - 1, 2147483647);
    return;
  }
  showToast("已经是第一页");
}

function jumpPage(index: number) {
  const snap = snapshot.value;
  if (!snap) return;
  animDir.value = "next";
  snap.pageIndex = Math.min(Math.max(index, 0), snap.pages.length - 1);
  persistProgress();
}

function changeChapter(delta: number) {
  const snap = snapshot.value;
  if (!snap) return;
  if (useTurnOverlay.value) {
    startChapterTurn(delta > 0 ? "next" : "prev");
    return;
  }
  const target = snap.chapterIndex + delta;
  if (target < 0 || target >= snap.totalChapters) {
    showToast(delta > 0 ? "已经是最后一章" : "已经是第一章");
    return;
  }
  lastTurnStartedAt = performance.now();
  animDir.value = delta > 0 ? "next" : "prev";
  open(target, 0);
}

function changeFont(delta: number) {
  const next = Math.min(Math.max(prefs.fontSize + delta, 12), 40);
  if (next === prefs.fontSize) return;
  const page = currentPage.value;
  const pos = page ? page.startChar : 0;
  prefs.patch({ fontSize: next });
  const snap = snapshot.value;
  open(snap?.chapterIndex ?? null, pos);
}

function applyClick(x: number, y: number, width: number, height: number) {
  if (menuVisible.value) {
    menuVisible.value = false;
    return;
  }
  const col = Math.min(2, Math.max(0, Math.floor((x / width) * 3)));
  const row = Math.min(2, Math.max(0, Math.floor((y / height) * 3)));
  const action = prefs.clickActions[row * 3 + col] ?? "next";
  switch (action) {
    case "prev":
      prevPage();
      break;
    case "next":
      nextPage();
      break;
    case "menu":
      menuVisible.value = true;
      break;
    case "prevChapter":
      changeChapter(-1);
      break;
    case "nextChapter":
      changeChapter(1);
      break;
    case "chapterList":
      goToc();
      break;
    case "search":
      searchChapter();
      break;
    case "addBookmark":
      addBookmark();
      break;
    case "editContent":
      activeDialog.value = "contentEdit";
      break;
    case "replaceState":
      activeDialog.value = "effective";
      break;
    case "syncProgress":
      router.push("/webdav");
      break;
    case "readAloudPrev":
    case "readAloudNext":
    case "readAloudToggle":
      aloudVisible.value = true;
      break;
    default:
      break;
  }
}

let pointerStart: { x: number; y: number } | null = null;
let swiped = false;
let ignoreClickUntil = 0;

function pointerOffset(event: PointerEvent) {
  const rect = canvasWrapRef.value?.getBoundingClientRect();
  return {
    x: event.clientX - (rect?.left ?? 0),
    y: event.clientY - (rect?.top ?? 0),
  };
}

function clearLongPress() {
  if (longPressTimer) {
    window.clearTimeout(longPressTimer);
    longPressTimer = undefined;
  }
  longPressStart = null;
}

function clearSelection() {
  selection.value = null;
  selectionMenuVisible.value = false;
  selectionDragging = false;
  selectionHandle = null;
}

function beginLongPress(event: PointerEvent) {
  clearLongPress();
  const page = currentPage.value;
  if (!page || turnActive.value) return;
  longPressStart = { x: event.clientX, y: event.clientY };
  longPressTimer = window.setTimeout(() => {
    longPressTimer = undefined;
    longPressStart = null;
    const current = currentPage.value;
    if (!current || turnActive.value) return;
    const point = pointerOffset(event);
    const hit = hitTestText(current, point.x, point.y);
    if (!hit) return;
    const word = wordSelection(current, hit.line, hit.char);
    if (!word) return;
    selection.value = word;
    selectionMenuVisible.value = false;
    selectionDragging = true;
    swiped = true;
    ignoreClickUntil = performance.now() + 1500;
    pointerStart = null;
    window.setTimeout(() => (swiped = false), 80);
  }, 500);
}

function updateSelection(page: LayoutPage, start: SelectionPos, end: SelectionPos) {
  const next = buildSelection(page, start, end);
  if (next) selection.value = next;
}

function onSelectionHandleDown(event: PointerEvent, which: "start" | "end") {
  event.preventDefault();
  event.stopPropagation();
  selectionHandle = which;
  selectionDragging = true;
  selectionMenuVisible.value = false;
  const target = event.currentTarget as HTMLElement | null;
  try {
    target?.setPointerCapture?.(event.pointerId);
  } catch {
    // ignore
  }
}

function onSelectionHandleMove(event: PointerEvent) {
  const which = selectionHandle;
  const page = currentPage.value;
  const current = selection.value;
  if (!which || !page || !current) return;
  event.preventDefault();
  event.stopPropagation();
  const point = pointerOffset(event);
  const hit = hitTestText(page, point.x, point.y);
  if (!hit) return;
  if (which === "start") {
    updateSelection(page, hit, current.end);
  } else {
    updateSelection(page, current.start, hit);
  }
}

function onSelectionHandleUp(event: PointerEvent) {
  event.preventDefault();
  event.stopPropagation();
  const target = event.currentTarget as HTMLElement | null;
  try {
    target?.releasePointerCapture?.(event.pointerId);
  } catch {
    // ignore
  }
  selectionHandle = null;
  selectionDragging = false;
  if (selection.value) {
    selectionMenuVisible.value = true;
    ignoreClickUntil = performance.now() + 300;
  }
}

async function copySelectionText(text: string) {
  try {
    await navigator.clipboard.writeText(text);
  } catch {
    const textarea = document.createElement("textarea");
    textarea.value = text;
    textarea.style.position = "fixed";
    textarea.style.opacity = "0";
    document.body.appendChild(textarea);
    textarea.select();
    document.execCommand("copy");
    textarea.remove();
  }
  showToast("已复制");
}

function jumpToText(keyword: string) {
  const snap = snapshot.value;
  if (!snap) return false;
  const key = keyword.trim();
  if (!key) return false;
  let offset = 0;
  for (const paragraph of snap.chapterParagraphs) {
    const index = paragraph.indexOf(key);
    if (index >= 0) {
      const pos = offset + index;
      const pageIndex = snap.pages.findIndex(
        (page) => pos >= page.startChar && pos < page.endChar,
      );
      jumpPage(pageIndex >= 0 ? pageIndex : 0);
      return true;
    }
    offset += paragraph.length + 1;
  }
  return false;
}

function onSelectionAction(key: string) {
  const current = selection.value;
  if (!current) return;
  const text = current.text.trim();
  selectionMenuVisible.value = false;
  if (key === "copy") {
    void copySelectionText(current.text);
  } else if (key === "search") {
    if (!jumpToText(text) && !jumpToText(text.replace(/\s+/g, ""))) {
      showToast("本章未找到匹配内容");
    }
  } else if (key === "bookmark") {
    const page = currentPage.value;
    const line = page?.lines[current.start.line];
    void addBookmark(current.text, line ? line.startChar + current.start.char : undefined);
  } else if (key === "dict") {
    dictWord.value = text;
    dictVisible.value = true;
  }
  clearSelection();
}


function onWindowPointerOut(event: PointerEvent) {
  if (event.relatedTarget === null) pointerOutside = true;
}

function onWindowPointerOver() {
  pointerOutside = false;
}

function onDocumentLeave() {
  pointerOutside = true;
}

function onDocumentEnter() {
  pointerOutside = false;
}

// 鼠标离开窗口后冻结在边缘；松手/丢失指针捕获时再播放剩余动画
function requestDragRelease() {
  if (turnActive.value && dragStartPoint) {
    dragStartPoint = null;
    dragDirection = null;
    dragBoundary = null;
    pointerOutside = false;
    const overlay = turnOverlay.value;
    if (overlay) {
      armTurnWatchdog(1500);
      overlay.release();
    } else {
      onTurnDone(true);
    }
    return;
  }
  if (dragBoundary) {
    const direction = dragBoundary;
    dragBoundary = null;
    dragDirection = null;
    pointerOutside = false;
    startChapterTurn(direction);
  }
}

function onLostPointerCapture() {
  requestDragRelease();
}

function onWindowBlur() {
  requestDragRelease();
}

window.addEventListener("pointerout", onWindowPointerOut);
window.addEventListener("pointerover", onWindowPointerOver);
window.addEventListener("blur", onWindowBlur);
document.documentElement.addEventListener("mouseleave", onDocumentLeave);
document.documentElement.addEventListener("mouseenter", onDocumentEnter);

function onPointerDown(event: PointerEvent) {
  if (selectionMenuVisible.value || selection.value) {
    clearSelection();
    swiped = true;
    ignoreClickUntil = performance.now() + 300;
    window.setTimeout(() => (swiped = false), 180);
    return;
  }
  pointerStart = { x: event.clientX, y: event.clientY };
  lastDownPoint = pointerOffset(event);
  pointerOutside = false;
  swiped = false;
  dragDirection = null;
  dragBoundary = null;
  dragStartPoint = null;
  beginLongPress(event);
  const target = event.currentTarget as HTMLElement | null;
  try {
    target?.setPointerCapture?.(event.pointerId);
  } catch {
    // ignore unsupported pointer capture
  }
}

function onPointerCancel(event: PointerEvent) {
  clearLongPress();
  const target = event.currentTarget as HTMLElement | null;
  try {
    target?.releasePointerCapture?.(event.pointerId);
  } catch {
    // ignore
  }
  pointerStart = null;
  selectionDragging = false;
  selectionHandle = null;
  requestDragRelease();
}

function onPointerMove(event: PointerEvent) {
  if (selectionDragging && selection.value && currentPage.value) {
    const current = selection.value;
    const point = pointerOffset(event);
    const hit = hitTestText(currentPage.value, point.x, point.y);
    if (hit) updateSelection(currentPage.value, current.start, hit);
    return;
  }
  if (!pointerStart) return;
  if (longPressStart) {
    const dx = event.clientX - longPressStart.x;
    const dy = event.clientY - longPressStart.y;
    if (Math.hypot(dx, dy) > prefs.pageTouchSlop) clearLongPress();
  }
  if (turnActive.value) {
    const outside =
      event.clientX <= 0 ||
      event.clientY <= 0 ||
      event.clientX >= window.innerWidth ||
      event.clientY >= window.innerHeight;
    if (outside) {
      pointerOutside = true;
      return;
    }
    pointerOutside = false;
    if (dragStartPoint && dragDirection && !dragBoundary) {
      const point = pointerOffset(event);
      turnOverlay.value?.follow(point.x, point.y);
    }
    return;
  }
  if (pointerOutside) return;
  const dx = event.clientX - pointerStart.x;
  const dy = event.clientY - pointerStart.y;
  if (!dragDirection) {
    if (Math.abs(dx) < prefs.pageTouchSlop || Math.abs(dx) < Math.abs(dy) * 1.2) return;
    if (!useTurnOverlay.value) return;
    const snap = snapshot.value;
    if (!snap) return;
    const direction: "next" | "prev" = dx < 0 ? "next" : "prev";
    dragDirection = direction;
    const hasPage =
      direction === "next"
        ? snap.pageIndex + 1 < snap.pages.length
        : snap.pageIndex > 0;
    if (!hasPage) {
      const hasChapter =
        direction === "next"
          ? snap.chapterIndex + 1 < snap.totalChapters
          : snap.chapterIndex > 0;
      dragBoundary = hasChapter ? direction : null;
      return;
    }
    const from = currentPage.value;
    const to = snap.pages[direction === "next" ? snap.pageIndex + 1 : snap.pageIndex - 1];
    dragStartPoint = pointerOffset(event);
    clearLongPress();
    clearSelection();
    startTurn(
      direction,
      from,
      to,
      () => {
        if (direction === "next") snap.pageIndex += 1;
        else snap.pageIndex -= 1;
        persistProgress();
      },
      {
        downX: lastDownPoint.x,
        downY: lastDownPoint.y,
        startX: dragStartPoint.x,
        startY: dragStartPoint.y,
      },
    );
    return;
  }
  if (dragBoundary) return;
  const point = pointerOffset(event);
  turnOverlay.value?.follow(point.x, point.y);
}

function onPointerUp(event: PointerEvent) {
  clearLongPress();
  const target = event.currentTarget as HTMLElement | null;
  try {
    target?.releasePointerCapture?.(event.pointerId);
  } catch {
    // ignore
  }
  if (selectionDragging || selection.value) {
    selectionDragging = false;
    pointerStart = null;
    if (selection.value) {
      selectionMenuVisible.value = true;
      ignoreClickUntil = performance.now() + 300;
    }
    return;
  }
  if (!pointerStart) return;
  const dx = event.clientX - pointerStart.x;
  const dy = event.clientY - pointerStart.y;
  pointerStart = null;

  if (turnActive.value && dragStartPoint && dragDirection && !dragBoundary) {
    requestDragRelease();
    swiped = true;
    window.setTimeout(() => (swiped = false), 80);
    return;
  }

  if (dragBoundary) {
    const direction = dragBoundary;
    dragDirection = null;
    dragBoundary = null;
    dragStartPoint = null;
    if (Math.abs(dx) > prefs.pageTouchSlop && Math.abs(dx) > Math.abs(dy) * 1.2) {
      swiped = true;
      startChapterTurn(direction);
      window.setTimeout(() => (swiped = false), 80);
    }
    return;
  }

  dragDirection = null;
  dragStartPoint = null;
  if (turnActive.value) return;
  if (Math.abs(dx) > prefs.pageTouchSlop && Math.abs(dx) > Math.abs(dy) * 1.2) {
    swiped = true;
    if (dx < 0) nextPage();
    else prevPage();
    window.setTimeout(() => (swiped = false), 80);
  }
}

function onClick(event: MouseEvent) {
  if (swiped || turnActive.value || performance.now() < ignoreClickUntil) return;
  if (selectionMenuVisible.value) {
    selectionMenuVisible.value = false;
    return;
  }
  if (selection.value) {
    clearSelection();
    return;
  }
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  window.clearTimeout(clickTimer);
  clickTimer = window.setTimeout(() => applyClick(x, y, rect.width, rect.height), 220);
}

function onDoubleClick(event: MouseEvent) {
  if (turnActive.value || selection.value) return;
  window.clearTimeout(clickTimer);
  const page = currentPage.value;
  if (!page) return;
  const target = event.currentTarget as HTMLElement;
  const rect = target.getBoundingClientRect();
  const x = event.clientX - rect.left;
  const y = event.clientY - rect.top;
  const line = page.lines.find((l) => y >= l.y && y <= l.y + l.height);
  if (!line) return;
  let hit = -1;
  for (let i = 0; i < line.chars.length; i++) {
    const ch = line.chars[i];
    if (x >= ch.x && x <= ch.x + ch.width) {
      hit = i;
      break;
    }
  }
  if (hit < 0) return;
  const isWordChar = (s: string) => /[\u4e00-\u9fffA-Za-z0-9]/.test(s);
  let start = hit;
  let end = hit + 1;
  while (start > 0 && isWordChar(line.chars[start - 1].ch)) start -= 1;
  while (end < line.chars.length && isWordChar(line.chars[end].ch)) end += 1;
  const word = line.chars
    .slice(start, end)
    .map((c) => c.ch)
    .join("")
    .trim();
  if (!word) {
    showToast("请选中文字");
    return;
  }
  dictWord.value = word;
  dictVisible.value = true;
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === "Escape") {
    menuVisible.value = false;
    activeDialog.value = "";
    clearSelection();
    return;
  }
  if (anyDialogOpen.value || aloudVisible.value || dictVisible.value) return;
  if (selection.value || selectionMenuVisible.value) return;
  if (event.key === prefs.pageKeyPrev) {
    event.preventDefault();
    prevPage(100);
  } else if (event.key === prefs.pageKeyNext) {
    event.preventDefault();
    nextPage(100);
  }
}

function onResize() {
  if (turnActive.value) {
    clearTurnWatchdog();
    turnOverlay.value?.stop();
    turnActive.value = false;
    turnFromPage.value = null;
    turnToPage.value = null;
    turnApply = null;
  }
  window.clearTimeout(resizeTimer);
  resizeTimer = window.setTimeout(() => {
    measureViewport();
    const snap = snapshot.value;
    const page = currentPage.value;
    open(snap?.chapterIndex ?? null, page ? page.startChar : null);
  }, 260);
}

function onPrefsApplied() {
  window.clearTimeout(prefsTimer);
  prefsTimer = window.setTimeout(() => {
    const snap = snapshot.value;
    const page = currentPage.value;
    open(snap?.chapterIndex ?? null, page ? page.startChar : null);
  }, 180);
}

async function addBookmark(textOverride?: string, chapterPos?: number) {
  const snap = snapshot.value;
  const page = currentPage.value;
  if (!snap || !page) return;
  const text = textOverride ?? page.lines.map((l) => l.text).join("");
  await bookmarkAdd({
    bookUrl: bookUrl.value,
    bookName: snap.book.name,
    bookAuthor: snap.book.author,
    chapterIndex: snap.chapterIndex,
    chapterPos: chapterPos ?? page.startChar,
    chapterName: snap.chapterTitle,
    bookText: text.slice(0, 200),
  });
  showToast("书签已添加");
}

function simulateFinish() {
  const snap = snapshot.value;
  if (!snap) return;
  menuVisible.value = false;
  open(snap.totalChapters - 1, 2147483647);
  showToast("已模拟读完，进度归位末章末页");
}

function onMoreSelect(key: string) {
  if (key === "bookmark") addBookmark();
  else if (key === "edit") activeDialog.value = "contentEdit";
  else if (key === "pageAnim") activeDialog.value = "style";
  else if (key === "simulated") activeDialog.value = "simulated";
  else if (key === "replace" || key === "effective") activeDialog.value = "effective";
  else if (key === "updateToc") goToc();
  else if (key === "reverse") showToast("反转内容暂未实现");
  else if (key === "sameTitleRemoved") showToast("移除重复标题暂未实现");
  else if (key === "reSegment") showToast("重新分段暂未实现");
  else if (key === "delRuby" || key === "delH") showToast("EPUB 标签处理暂未实现");
  else if (key === "imageStyle") showToast("图片样式暂未实现");
  else if (key === "log") showToast("日志暂未实现");
  else if (key === "help") showToast("阅读器帮助暂未实现");
}

function onChangeSource() {
  router.push("/book-source");
}

function onRefreshToc() {
  goToc();
}

function onCacheBook() {
  router.push("/cache-books");
}

function onSimulatedJump(chapter: number, pos: number) {
  activeDialog.value = "";
  open(chapter, pos);
}

function onContentSaved() {
  const snap = snapshot.value;
  const page = currentPage.value;
  open(snap?.chapterIndex ?? null, page ? page.startChar : null);
  showToast("正文已保存并重新排版");
}

function updateAutoRead() {
  window.clearInterval(autoTimer);
  autoTimer = undefined;
  const overlayOpen =
    menuVisible.value ||
    anyDialogOpen.value ||
    dictVisible.value ||
    aloudVisible.value ||
    selection.value !== null ||
    selectionMenuVisible.value;
  if (!snapshot.value || overlayOpen || prefs.autoReadSeconds <= 0) return;
  autoTimer = window.setInterval(() => nextPage(), prefs.autoReadSeconds * 1000);
}

function toggleAutoPage() {
  prefs.patch({ autoReadSeconds: prefs.autoReadSeconds > 0 ? 0 : 30 });
  showToast(prefs.autoReadSeconds > 0 ? "已开启自动翻页（30 秒）" : "已关闭自动翻页");
}

function toggleNight() {
  theme.setMode(
    theme.effectiveMode === ThemeMode.Night ? ThemeMode.Day : ThemeMode.Night,
  );
}

function adjustBrightness(value: number) {
  brightness.value = Math.min(Math.max(value, 0.4), 1.4);
}

function resetBrightness() {
  brightness.value = 1;
}

function toggleBrightnessSide() {
  brightnessSide.value = brightnessSide.value === "left" ? "right" : "left";
}

function searchChapter() {
  menuVisible.value = false;
  const keyword = window.prompt("本章搜索");
  const key = keyword?.trim();
  if (!key) return;
  if (jumpToText(key)) {
    showToast(`已定位到「${key}」`);
  } else {
    showToast("本章未找到匹配内容");
  }
}

function onAloudPosition(pos: number) {
  aloudPosition.value = pos;
}

function onAloudFinished() {
  const snap = snapshot.value;
  if (!snap) return;
  if (snap.chapterIndex + 1 < snap.totalChapters) {
    open(snap.chapterIndex + 1, 0);
  } else {
    showToast("已读完最后一章");
  }
}

function goToc() {
  router.push({ path: "/toc", query: { bookUrl: bookUrl.value } });
}

function openTocFromMenu() {
  menuVisible.value = false;
  goToc();
}

function openAloudFromMenu() {
  menuVisible.value = false;
  aloudVisible.value = true;
}

function openStyleFromMenu() {
  menuVisible.value = false;
  activeDialog.value = "style";
}

function openMoreFromMenu() {
  menuVisible.value = false;
  activeDialog.value = "more";
}

function goBack() {
  router.push("/");
}

function onAloudShowMenu() {
  aloudVisible.value = false;
  menuVisible.value = true;
}

function onAloudBackstage() {
  aloudVisible.value = false;
  goBack();
}

function onWheel(event: WheelEvent) {
  if (!prefs.mouseWheelPage || menuVisible.value) return;
  if (selection.value || selectionMenuVisible.value) return;
  if (!event.deltaY) return;
  const isNext = event.deltaY > 0;
  window.clearTimeout(isNext ? wheelNextTimer : wheelPrevTimer);
  const timer = window.setTimeout(() => {
    if (turnActive.value) return;
    if (performance.now() - lastTurnStartedAt < 220) return;
    if (isNext) nextPage(100);
    else prevPage(100);
  }, 200);
  if (isNext) wheelNextTimer = timer;
  else wheelPrevTimer = timer;
}

function initialTarget(): { chapter: number | null; pos: number | null } {
  const raw = route.query.chapter;
  if (raw === undefined || raw === null || raw === "") {
    return { chapter: null, pos: null };
  }
  const chapter = Number(raw);
  return Number.isFinite(chapter) ? { chapter, pos: 0 } : { chapter: null, pos: null };
}

watch(aloudVisible, (open) => {
  if (!open) aloudPosition.value = -1;
  updateAutoRead();
});

watch(
  [
    () => prefs.autoReadSeconds,
    () => snapshot.value,
    menuVisible,
    anyDialogOpen,
    dictVisible,
    aloudVisible,
    selection,
    selectionMenuVisible,
  ],
  updateAutoRead,
  { flush: "post", immediate: true },
);

watch(
  () => [route.query.bookUrl, route.query.chapter],
  () => {
    snapshot.value = null;
    menuVisible.value = false;
    const target = initialTarget();
    open(target.chapter, target.pos);
  },
);

onMounted(() => {
  measureViewport();
  const target = initialTarget();
  open(target.chapter, target.pos);
  window.addEventListener("keydown", onKeydown);
  window.addEventListener("resize", onResize);
  loadBattery();
  clockTimer = window.setInterval(() => (now.value = new Date()), 30_000);
});

onBeforeUnmount(() => {
  persistProgress();
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("resize", onResize);
  window.removeEventListener("pointerout", onWindowPointerOut);
  window.removeEventListener("pointerover", onWindowPointerOver);
  window.removeEventListener("blur", onWindowBlur);
  document.documentElement.removeEventListener("mouseleave", onDocumentLeave);
  document.documentElement.removeEventListener("mouseenter", onDocumentEnter);
  window.clearTimeout(resizeTimer);
  window.clearTimeout(prefsTimer);
  window.clearTimeout(toastTimer);
  window.clearTimeout(clickTimer);
  window.clearInterval(autoTimer);
  window.clearTimeout(wheelNextTimer);
  window.clearTimeout(wheelPrevTimer);
  window.clearInterval(clockTimer);
});
</script>

<template>
  <div
    class="reader-page"
    :class="{ eink: theme.isEink }"
    :style="{ background: readerBackground }"
  >
    <div v-if="loading" class="status">加载中…</div>
    <div v-else-if="error" class="status error">
      <p>{{ error }}</p>
      <button @click="open(null, null)">重试</button>
      <button @click="goToc">打开目录</button>
    </div>

    <div
      v-else-if="snapshot && currentPage"
      ref="canvasWrapRef"
      class="canvas-wrap"
      :style="{ width: `${viewWidth}px`, height: `${viewHeight}px`, ...brightnessStyle }"
      @click="onClick"
      @dblclick="onDoubleClick"
      @pointerdown="onPointerDown"
      @pointermove="onPointerMove"
      @pointerup="onPointerUp"
      @pointercancel="onPointerCancel"
      @lostpointercapture="onLostPointerCapture"
      @wheel.prevent="onWheel"
    >
      <Transition v-if="slideMode" :name="transitionName">
        <ReaderCanvas
          v-show="!turnActive"
          :key="`${snapshot.chapterIndex}-${snapshot.pageIndex}`"
          :page="currentPage"
          :width="viewWidth"
          :height="viewHeight"
          :background="readerBackground"
          :background-image="readerImage"
          :background-alpha="readerBackgroundAlpha"
          :text-color="readerTextColor"
          :highlight-line="highlightLine"
          :font-family="prefs.fontFamily"
          :underline="readerUnderline"
          :info-bar="currentInfoBar"
        />
      </Transition>
      <ReaderCanvas
        v-else
        v-show="!turnActive"
        :key="`${snapshot.chapterIndex}-${snapshot.pageIndex}`"
        :page="currentPage"
        :width="viewWidth"
        :height="viewHeight"
        :background="readerBackground"
        :background-image="readerImage"
        :background-alpha="readerBackgroundAlpha"
        :text-color="readerTextColor"
        :highlight-line="highlightLine"
        :font-family="prefs.fontFamily"
        :underline="readerUnderline"
        :info-bar="currentInfoBar"
      />
      <SelectionHighlight
        v-if="selection"
        :page="currentPage"
        :selection="selection"
        :width="viewWidth"
        :height="viewHeight"
      />
      <template v-if="selection && selectionHandleStyle">
        <button
          type="button"
          class="selection-handle start"
          aria-label="调整选择起点"
          :style="selectionHandleStyle.start"
          @pointerdown="onSelectionHandleDown($event, 'start')"
          @pointermove="onSelectionHandleMove"
          @pointerup="onSelectionHandleUp"
          @pointercancel="onSelectionHandleUp"
          @click.stop
        />
        <button
          type="button"
          class="selection-handle end"
          aria-label="调整选择终点"
          :style="selectionHandleStyle.end"
          @pointerdown="onSelectionHandleDown($event, 'end')"
          @pointermove="onSelectionHandleMove"
          @pointerup="onSelectionHandleUp"
          @pointercancel="onSelectionHandleUp"
          @click.stop
        />
      </template>
      <div
        v-if="selectionMenuVisible && selection"
        class="selection-menu"
        :style="selectionMenuStyle"
        @pointerdown.stop
        @click.stop
      >
        <button type="button" @click.stop="onSelectionAction('copy')">复制</button>
        <button type="button" @click.stop="onSelectionAction('search')">搜索</button>
        <button type="button" @click.stop="onSelectionAction('bookmark')">书签</button>
        <button type="button" @click.stop="onSelectionAction('dict')">词典</button>
      </div>
    </div>

    <div v-else class="status">
      <p>章节加载失败</p>
      <button @click="goToc">打开目录</button>
    </div>

    <div v-if="turnActive" class="turn-layer">
      <PageTurnOverlay
        ref="turnOverlay"
        :mode="turnMode"
        :direction="turnDirection"
        :from-page="turnFromPage"
        :to-page="turnToPage"
        :width="viewWidth"
        :height="viewHeight"
        :background="readerBackground"
        :background-image="readerImage"
        :background-alpha="readerBackgroundAlpha"
        :text-color="readerTextColor"
        :font-family="prefs.fontFamily"
        :underline="readerUnderline"
        :brightness="brightness"
        :from-info-bar="turnFromInfoBar"
        :to-info-bar="turnToInfoBar"
        @done="onTurnDone"
      />
    </div>

    <div v-if="toast" class="toast">{{ toast }}</div>

    <transition name="menu">
      <div v-if="menuVisible && snapshot" class="menu-mask" data-id="vw_menu_bg" @click.self="menuVisible = false">
        <TitleBar data-id="title_bar" :title="snapshot.book.name" @back="goBack">
          <template #below>
            <div v-if="prefs.showTitleAddition" class="title-addition" data-id="title_bar_addition">
              <span class="addition-chapter" data-id="tv_chapter_name">{{ snapshot.chapterTitle }}</span>
              <span v-if="isOnline" class="addition-url" data-id="tv_chapter_url">{{ snapshot.book.bookUrl }}</span>
              <button
                v-if="isOnline"
                class="addition-source"
                data-id="tv_source_action"
                @click="onChangeSource"
              >
                {{ snapshot.book.originName || "换源" }}
              </button>
            </div>
          </template>
          <template #actions>
            <template v-if="isOnline">
              <button aria-label="换源" @click="onChangeSource">
                <Icon name="ic_exchange" :size="24" />
              </button>
              <button aria-label="刷新" @click="onRefreshToc">
                <Icon name="ic_refresh_black_24dp" :size="24" />
              </button>
              <button aria-label="离线缓存" @click="onCacheBook">
                <Icon name="ic_download_line" :size="24" />
              </button>
            </template>
            <button v-if="isTxt" aria-label="TXT 目录规则" @click="router.push('/toc-rules')">
              <Icon name="ic_exchange" :size="24" />
            </button>
            <button v-if="!isOnline" aria-label="设置编码" @click="showToast('编码设置暂未实现')">
              <Icon name="ic_translate" :size="24" />
            </button>
            <button aria-label="更多" @click="moreMenuVisible = true">
              <Icon name="ic_more_vert" :size="24" />
            </button>
          </template>
        </TitleBar>
        <PopupMenu
          v-model="moreMenuVisible"
          :items="moreMenuItems"
          @select="onMoreSelect"
        />

        <div v-if="prefs.showBrightness" class="brightness-bar" :class="brightnessSide" data-id="ll_brightness">
          <button class="brightness-icon" aria-label="自动亮度" data-id="iv_brightness_auto" @click="resetBrightness">
            <Icon name="ic_brightness_auto" :size="22" />
          </button>
          <input
            class="brightness-slider"
            data-id="seek_brightness"
            type="range"
            min="0.4"
            max="1.4"
            step="0.05"
            :value="brightness"
            @input="adjustBrightness(Number(($event.target as HTMLInputElement).value))"
          />
          <button class="brightness-icon" aria-label="切换位置" @click="toggleBrightnessSide">
            <Icon name="ic_swap_horiz" :size="22" />
          </button>
        </div>

        <div class="fab-row" data-id="ll_floating_button">
          <button class="fab" aria-label="搜索" data-id="fabSearch" @click="searchChapter">
            <Icon name="ic_search" :size="22" />
          </button>
          <button
            class="fab"
            data-id="fabAutoPage"
            :class="{ active: prefs.autoReadSeconds > 0 }"
            aria-label="自动翻页"
            @click="menuVisible = false; activeDialog = 'auto'"
          >
            <Icon name="ic_auto_page" :size="22" />
          </button>
          <button
            class="fab"
            data-id="fabReplaceRule"
            :class="{ active: activeDialog === 'effective' }"
            aria-label="替换净化"
            @click="activeDialog = 'effective'"
          >
            <Icon name="ic_find_replace" :size="22" />
          </button>
          <button class="fab" aria-label="夜间模式" data-id="fabNightTheme" @click="toggleNight">
            <Icon name="ic_brightness" :size="22" />
          </button>
        </div>

        <div class="read-bottom" data-id="bottom_menu">
          <div class="chapter-nav">
            <button data-id="tv_pre" @click="changeChapter(-1)">上一章</button>
            <input
              type="range"
              data-id="seek_read_page"
              min="0"
              :max="Math.max(snapshot.pages.length - 1, 0)"
              :value="snapshot.pageIndex"
              @input="jumpPage(Number(($event.target as HTMLInputElement).value))"
            />
            <button data-id="tv_next" @click="changeChapter(1)">下一章</button>
          </div>

          <div class="read-icon-row">
            <button data-id="ll_catalog" @click="openTocFromMenu">
              <Icon name="ic_toc" :size="22" data-id="iv_catalog" />
              <span data-id="tv_catalog">目录</span>
            </button>
            <button data-id="ll_read_aloud" @click="openAloudFromMenu">
              <Icon name="ic_read_aloud" :size="22" data-id="iv_read_aloud" />
              <span data-id="tv_read_aloud">朗读</span>
            </button>
            <button data-id="ll_font" @click="openStyleFromMenu">
              <Icon name="ic_interface_setting" :size="22" data-id="iv_font" />
              <span data-id="tv_font">界面</span>
            </button>
            <button data-id="ll_setting" @click="openMoreFromMenu">
              <Icon name="ic_settings" :size="22" data-id="iv_setting" />
              <span data-id="tv_setting">设置</span>
            </button>
          </div>

</div>

      </div>
    </transition>

    <ReadAloudPanel
      v-model="aloudVisible"
      :snapshot="snapshot"
      @page-end="nextPage"
      @position="onAloudPosition"
      @finished="onAloudFinished"
      @open-config="activeDialog = 'aloudConfig'"
      @show-menu="onAloudShowMenu"
      @backstage="onAloudBackstage"
    />
    <ReadStyleDialog
      :model-value="activeDialog === 'style'"
      @update:model-value="activeDialog = $event ? 'style' : ''"
      @apply="onPrefsApplied"
      @open-dialog="activeDialog = $event"
    />
    <BgTextDialog
      :model-value="activeDialog === 'bg'"
      @update:model-value="activeDialog = $event ? 'bg' : ''"
    />
    <PaddingDialog
      :model-value="activeDialog === 'padding'"
      @update:model-value="activeDialog = $event ? 'padding' : ''"
      @apply="onPrefsApplied"
    />
    <AutoReadDialog
      :model-value="activeDialog === 'auto'"
      @update:model-value="activeDialog = $event ? 'auto' : ''"
      @apply="updateAutoRead"
    />
    <ReadAloudConfigDialog
      :model-value="activeDialog === 'aloudConfig'"
      @update:model-value="activeDialog = $event ? 'aloudConfig' : ''"
    />
    <SpeakEngineDialog
      :model-value="activeDialog === 'engine'"
      @update:model-value="activeDialog = $event ? 'engine' : ''"
    />
    <ClickActionDialog
      :model-value="activeDialog === 'click'"
      @update:model-value="activeDialog = $event ? 'click' : ''"
    />
    <PageKeyDialog
      :model-value="activeDialog === 'pageKey'"
      @update:model-value="activeDialog = $event ? 'pageKey' : ''"
    />
    <MoreConfigDialog
      :model-value="activeDialog === 'more'"
      @update:model-value="activeDialog = $event ? 'more' : ''"
      @apply="onPrefsApplied"
    />
    <SimulatedReadingDialog
      :model-value="activeDialog === 'simulated'"
      :total-chapters="snapshot?.totalChapters ?? 0"
      :current-chapter="snapshot?.chapterIndex ?? 0"
      @update:model-value="activeDialog = $event ? 'simulated' : ''"
      @finish="simulateFinish"
      @jump="onSimulatedJump"
    />
    <EffectiveReplacesDialog
      :model-value="activeDialog === 'effective'"
      :rules="snapshot?.effectiveRules ?? []"
      @update:model-value="activeDialog = $event ? 'effective' : ''"
    />
    <ContentEditDialog
      :model-value="activeDialog === 'contentEdit'"
      :snapshot="snapshot"
      @update:model-value="activeDialog = $event ? 'contentEdit' : ''"
      @saved="onContentSaved"
    />
    <FontSelectDialog
      :model-value="activeDialog === 'fonts'"
      @update:model-value="activeDialog = $event ? 'fonts' : ''"
      @apply="onPrefsApplied"
    />
    <DictDialog v-model="dictVisible" :word="dictWord" />
  </div>
</template>

<style scoped>
.reader-page {
  position: relative;
  width: 100%;
  height: 100vh;
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
}
.turn-layer {
  position: absolute;
  inset: 0;
  z-index: 2;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: none;
}
.reader-page.eink,
.reader-page.eink * {
  animation: none !important;
  transition: none !important;
}
.status {
  color: #f5f5f5;
  display: flex;
  flex-direction: column;
  gap: 10px;
  align-items: center;
}
.status.error {
  color: #ffb4b4;
}
.canvas-wrap {
  position: relative;
  overflow: hidden;
  touch-action: pan-y;
}
.selection-handle {
  position: absolute;
  z-index: 4;
  width: 14px;
  height: 14px;
  padding: 0;
  border: 2px solid #fff;
  border-radius: 50%;
  background: var(--ac-accent);
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.35);
  touch-action: none;
  cursor: ew-resize;
}
.selection-menu {
  position: absolute;
  z-index: 5;
  display: flex;
  align-items: center;
  padding: 2px 0;
  border-radius: 3px;
  background: var(--ac-background);
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.28);
}
.selection-menu button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 10px 14px;
  font-size: 14px;
  cursor: pointer;
  white-space: nowrap;
}
.selection-menu button:hover {
  background: var(--ac-transparent-10);
}
.toast {
  position: fixed;
  left: 50%;
  bottom: 48px;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.78);
  color: #fff;
  padding: 8px 16px;
  border-radius: 999px;
  font-size: 13px;
  z-index: 30;
}
.menu-mask {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  flex-direction: column;
  z-index: 20;
}
.menu-mask :deep(.title-bar) {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
}
.brightness-bar {
  position: absolute;
  top: 80px;
  bottom: 240px;
  width: 44px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 8px 0;
  background: var(--ac-transparent-30);
  border-radius: 22px;
  z-index: 3;
}
.brightness-bar.left {
  left: 8px;
}
.brightness-bar.right {
  right: 8px;
}
.brightness-icon {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: #ffffff;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
}
.brightness-slider {
  flex: 1;
  width: 24px;
  accent-color: var(--ac-primary);
  writing-mode: vertical-lr;
  direction: rtl;
}
.read-bottom {
  margin-top: 0;
  background: var(--ac-background-menu);
  color: var(--ac-primary-text);
  padding-bottom: env(safe-area-inset-bottom);
  box-shadow: 0 -2px 10px rgba(0, 0, 0, 0.18);
}
.fab-row {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  justify-items: center;
  align-items: center;
  width: 100%;
  max-width: 600px;
  margin: auto auto 0;
  padding: 8px 12px 16px;
}
.fab {
  width: 40px;
  height: 40px;
  flex: none;
  border-radius: 50%;
  border: none;
  background: var(--ac-background);
  color: var(--ac-primary-text);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.24);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: 0;
}
.fab.active {
  color: var(--ac-primary);
  box-shadow: 0 0 0 2px var(--ac-primary);
}
.chapter-nav {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 5px 20px;
}
.chapter-nav button {
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  padding: 10px;
  font-size: 14px;
  cursor: pointer;
}
.chapter-nav input {
  flex: 1;
  accent-color: var(--ac-primary);
}
.read-icon-row {
  display: flex;
  justify-content: space-around;
  padding: 6px 12px;
}
.read-icon-row button {
  width: 60px;
  border: none;
  background: transparent;
  color: var(--ac-primary-text);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 3px;
  font-size: 12px;
  padding: 4px 0 7px;
  cursor: pointer;
}
.read-progress {
  margin: 0;
  padding: 4px 20px 10px;
  font-size: 11px;
  color: var(--ac-secondary-text);
}
.effective-sheet {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 4;
}
.effective-card {
  background: var(--ac-background);
  color: var(--ac-primary-text);
  border-radius: 4px;
  padding: 16px;
  width: min(320px, 86%);
  max-height: 60vh;
  overflow-y: auto;
}
.effective-card h3 {
  margin: 0 0 8px;
  font-size: 15px;
}
.effective-card ul {
  margin: 0;
  padding-left: 18px;
  font-size: 13px;
}
.effective-card button {
  margin-top: 12px;
  border: 1px solid var(--ac-divider);
  background: transparent;
  color: var(--ac-primary);
  border-radius: 2px;
  padding: 6px 14px;
  cursor: pointer;
}
.menu-enter-active,
.menu-leave-active {
  transition: opacity 0.2s ease;
}
.menu-enter-from,
.menu-leave-to {
  opacity: 0;
}
.page-slide-next-enter-active,
.page-slide-next-leave-active,
.page-slide-prev-enter-active,
.page-slide-prev-leave-active {
  transition: transform 0.24s ease;
  position: absolute;
  inset: 0;
}
.page-slide-next-enter-from {
  transform: translateX(100%);
}
.page-slide-next-leave-to {
  transform: translateX(-100%);
}
.page-slide-prev-enter-from {
  transform: translateX(-100%);
}
.page-slide-prev-leave-to {
  transform: translateX(100%);
}

.title-addition {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
  padding: 0 16px 6px;
  line-height: 1.25;
}
.addition-chapter,
.addition-url {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.addition-chapter {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  opacity: 0.9;
}
.addition-url {
  flex: none;
  max-width: 38%;
  font-size: 11px;
  opacity: 0.65;
}
.addition-source {
  flex: none;
  border: none;
  border-radius: 2px;
  padding: 1px 8px;
  font-size: 11px;
  line-height: 18px;
  background: rgba(255, 255, 255, 0.22);
  color: inherit;
  cursor: pointer;
}
</style>
