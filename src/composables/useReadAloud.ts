import { computed, ref } from "vue";
import { fetch } from "@tauri-apps/plugin-http";
import type { HttpTts } from "@/api/bindings";
import { httpTtsEngines } from "@/api/rules";
import { buildRequest } from "@/utils/ruleQuery";

export interface AloudHooks {
  getParagraphs: () => string[];
  getPageStart: () => number;
  getPageEnd: () => number;
  onPageEnd: () => void;
  onFinished: () => void;
  onPosition: (charPos: number, paragraphIndex: number) => void;
}

const PUNCT_ONLY = /^[\s\p{P}\p{S}]*$/u;

function stripIndent(text: string): string {
  return text.replace(/^[\s\u3000]+/, "").trim();
}

export function useReadAloud(hooks: AloudHooks) {
  const speaking = ref(false);
  const paused = ref(false);
  const index = ref(0);
  const charPos = ref(0);
  const error = ref("");
  const engines = ref<HttpTts[]>([]);
  const engineId = ref("");
  const rate = ref(1);
  const pitch = ref(1);
  const timerMinutes = ref(0);
  const remainSeconds = ref(0);
  const paragraphCount = computed(() => hooks.getParagraphs().length);

  let runToken = 0;
  let currentUtterance: SpeechSynthesisUtterance | null = null;
  let currentAudio: HTMLAudioElement | null = null;
  let currentAudioUrl = "";
  let currentAbort: AbortController | null = null;
  let resolveCurrent: (() => void) | null = null;
  let timerHandle: number | undefined;
  let consecutiveErrors = 0;
  let lastRequestedPageEnd = -1;

  const activeEngine = computed(() => {
    const id = engineId.value.trim();
    if (!id) return null;
    return engines.value.find((e) => String(e.id) === id) ?? null;
  });

  const engineLabel = computed(() => activeEngine.value?.name ?? "系统 TTS");

  async function loadEngines() {
    try {
      engines.value = await httpTtsEngines();
    } catch {
      engines.value = [];
    }
  }

  function offsets(): { start: number; length: number }[] {
    const paragraphs = hooks.getParagraphs();
    let offset = 0;
    return paragraphs.map((p) => {
      const length = p.length;
      const entry = { start: offset, length };
      offset += length + 1;
      return entry;
    });
  }

  function indexForChar(pos: number): number {
    const list = offsets();
    if (pos <= 0) return 0;
    for (let i = 0; i < list.length; i++) {
      if (pos < list[i].start + list[i].length + 1) return i;
    }
    return Math.max(0, list.length - 1);
  }

  function finishCurrent() {
    const resolve = resolveCurrent;
    resolveCurrent = null;
    resolve?.();
  }

  function cancelCurrent() {
    runToken += 1;
    currentAbort?.abort();
    currentAbort = null;
    if (window.speechSynthesis) window.speechSynthesis.cancel();
    currentUtterance = null;
    if (currentAudio) {
      currentAudio.pause();
      currentAudio.src = "";
      currentAudio = null;
    }
    if (currentAudioUrl) {
      URL.revokeObjectURL(currentAudioUrl);
      currentAudioUrl = "";
    }
    finishCurrent();
  }

  function speakSystem(text: string): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      if (!window.speechSynthesis || typeof SpeechSynthesisUtterance === "undefined") {
        reject(new Error("系统不支持语音合成"));
        return;
      }
      const utterance = new SpeechSynthesisUtterance(text);
      utterance.rate = Math.min(Math.max(rate.value, 0.5), 2);
      utterance.pitch = Math.min(Math.max(pitch.value, 0.5), 2);
      utterance.lang = "zh-CN";
      currentUtterance = utterance;
      resolveCurrent = resolve;
      utterance.onend = () => {
        currentUtterance = null;
        finishCurrent();
      };
      utterance.onerror = (event) => {
        currentUtterance = null;
        resolveCurrent = null;
        reject(new Error(event.error || "语音合成失败"));
      };
      window.speechSynthesis.speak(utterance);
    });
  }

  async function speakOnline(text: string, engine: HttpTts): Promise<void> {
    const request = buildRequest(engine.url, {
      speakText: text,
      speakSpeed: String(rate.value),
      speed: String(rate.value),
    });
    const controller = new AbortController();
    currentAbort = controller;
    const response = await fetch(request.url, {
      method: request.method,
      headers: request.headers,
      body: request.body,
      signal: controller.signal,
    });
    if (!response.ok) throw new Error(`在线合成失败 HTTP ${response.status}`);
    const buffer = await response.arrayBuffer();
    if (controller.signal.aborted) return;
    const blob = new Blob([buffer], { type: engine.contentType || "audio/mpeg" });
    currentAudioUrl = URL.createObjectURL(blob);
    const audio = new Audio(currentAudioUrl);
    currentAudio = audio;
    await new Promise<void>((resolve, reject) => {
      resolveCurrent = resolve;
      audio.onended = () => finishCurrent();
      audio.onerror = () => reject(new Error("音频播放失败"));
      audio.play().catch(reject);
    });
    currentAbort = null;
    if (currentAudio === audio) currentAudio = null;
    if (currentAudioUrl) {
      URL.revokeObjectURL(currentAudioUrl);
      currentAudioUrl = "";
    }
  }

  async function speakCurrent(): Promise<void> {
    const paragraphs = hooks.getParagraphs();
    const text = stripIndent(paragraphs[index.value] ?? "");
    if (!text) {
      index.value += 1;
      return;
    }
    const engine = activeEngine.value;
    if (engine) {
      await speakOnline(text, engine);
    } else {
      await speakSystem(text);
    }
  }

  async function loop(token: number) {
    while (speaking.value && token === runToken) {
      if (paused.value) {
        await new Promise((resolve) => setTimeout(resolve, 120));
        continue;
      }
      const paragraphs = hooks.getParagraphs();
      if (index.value >= paragraphs.length) {
        speaking.value = false;
        stopTimer(false);
        hooks.onFinished();
        return;
      }

      const list = offsets();
      const start = list[index.value]?.start ?? 0;
      const pageEnd = hooks.getPageEnd();
      if (pageEnd > 0 && start >= pageEnd && pageEnd !== lastRequestedPageEnd) {
        lastRequestedPageEnd = pageEnd;
        hooks.onPageEnd();
      }
      charPos.value = start;
      hooks.onPosition(start, index.value);

      const text = stripIndent(paragraphs[index.value]);
      if (!text || PUNCT_ONLY.test(text)) {
        index.value += 1;
        consecutiveErrors = 0;
        continue;
      }

      try {
        await speakCurrent();
        consecutiveErrors = 0;
        if (token !== runToken) return;
        index.value += 1;
        const next = offsets()[index.value];
        if (next) {
          charPos.value = next.start;
          hooks.onPosition(next.start, index.value);
        }
      } catch (e) {
        if (token !== runToken) return;
        if (currentAbort?.signal.aborted) return;
        consecutiveErrors += 1;
        error.value = String(e);
        if (consecutiveErrors >= 5) {
          speaking.value = false;
          stopTimer(false);
          error.value = "连续合成失败，已暂停朗读";
          return;
        }
        index.value += 1;
        if (activeEngine.value) {
          engineId.value = "";
          error.value = "在线合成失败，已回退系统 TTS";
        }
      }
    }
  }

  function start() {
    const paragraphs = hooks.getParagraphs();
    if (paragraphs.length === 0) {
      error.value = "当前章节没有可朗读内容";
      return;
    }
    if (speaking.value) return;
    error.value = "";
    consecutiveErrors = 0;
    lastRequestedPageEnd = -1;
    index.value = indexForChar(hooks.getPageStart());
    speaking.value = true;
    paused.value = false;
    runToken += 1;
    const token = runToken;
    if (timerMinutes.value > 0) {
      remainSeconds.value = timerMinutes.value * 60;
      startTimer();
    }
    loop(token);
  }

  function pause() {
    if (!speaking.value) return;
    paused.value = true;
    if (window.speechSynthesis) window.speechSynthesis.pause();
    currentAudio?.pause();
  }

  function resume() {
    if (!speaking.value) {
      start();
      return;
    }
    paused.value = false;
    if (window.speechSynthesis) window.speechSynthesis.resume();
    currentAudio?.play().catch(() => undefined);
  }

  function stop() {
    speaking.value = false;
    paused.value = false;
    runToken += 1;
    cancelCurrent();
    stopTimer(false);
  }

  function skip(delta: number) {
    const count = hooks.getParagraphs().length;
    const target = Math.min(Math.max(index.value + delta, 0), Math.max(count - 1, 0));
    if (target === index.value) return;
    index.value = target;
    const pos = offsets()[target]?.start ?? 0;
    charPos.value = pos;
    hooks.onPosition(pos, target);
    if (speaking.value) {
      cancelCurrent();
      runToken += 1;
      const token = runToken;
      setTimeout(() => loop(token), 0);
    }
  }

  function startTimer() {
    stopTimer(false);
    timerHandle = window.setInterval(() => {
      if (remainSeconds.value <= 0) {
        stopTimer(false);
        stop();
        return;
      }
      remainSeconds.value -= 1;
    }, 1000);
  }

  function stopTimer(reset: boolean) {
    if (timerHandle !== undefined) {
      window.clearInterval(timerHandle);
      timerHandle = undefined;
    }
    if (reset) remainSeconds.value = timerMinutes.value * 60;
  }

  function setTimer(minutes: number) {
    timerMinutes.value = minutes;
    remainSeconds.value = minutes * 60;
    if (speaking.value && minutes > 0) startTimer();
    if (minutes === 0) stopTimer(false);
  }

  function setEngine(id: string) {
    engineId.value = id;
    if (speaking.value) {
      cancelCurrent();
      runToken += 1;
      const token = runToken;
      setTimeout(() => loop(token), 0);
    }
  }

  function reloadAndContinue() {
    lastRequestedPageEnd = -1;
    if (!speaking.value) return;
    cancelCurrent();
    runToken += 1;
    index.value = indexForChar(hooks.getPageStart());
    const token = runToken;
    setTimeout(() => loop(token), 0);
  }

  return {
    speaking,
    paused,
    index,
    charPos,
    error,
    engines,
    engineId,
    engineLabel,
    rate,
    pitch,
    timerMinutes,
    remainSeconds,
    paragraphCount,
    loadEngines,
    start,
    pause,
    resume,
    stop,
    skip,
    setTimer,
    setEngine,
    reloadAndContinue,
  };
}
