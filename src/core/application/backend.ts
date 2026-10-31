import { platform } from "@/runtime";

export function invoke<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return platform.backend.invoke<T>(command, args);
}
