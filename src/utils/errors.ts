import { reactive } from "vue";
import { logError } from "./appLog";

export interface GlobalErrorState {
  visible: boolean;
  source: string;
  message: string;
}

export const globalError = reactive<GlobalErrorState>({
  visible: false,
  source: "",
  message: "",
});

export function showError(source: string, error: unknown): void {
  logError(source, error);
  globalError.source = source;
  globalError.message =
    error instanceof Error ? error.message : String(error || "未知错误");
  globalError.visible = true;
}
