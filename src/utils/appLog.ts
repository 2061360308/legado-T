import { appLogAdd } from "@/core/application/logs";

let installed = false;

export function writeAppLog(
  level: "info" | "warn" | "error",
  source: string,
  message: string,
): void {
  void appLogAdd(level, source, message).catch(() => undefined);
}

export function logError(source: string, error: unknown): void {
  writeAppLog("error", source, error instanceof Error ? error.stack || error.message : String(error));
}

export function installGlobalErrorLogging(): void {
  if (installed) return;
  installed = true;
  window.addEventListener("error", (event) => {
    logError("window.error", event.error ?? event.message);
  });
  window.addEventListener("unhandledrejection", (event) => {
    logError("window.rejection", event.reason);
  });
}
