import { platform } from "@/runtime";

export async function invoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  try {
    return await platform.backend.invoke<T>(command, args);
  } catch (error) {
    if (command !== "app_log_add") {
      void import("@/utils/appLog")
        .then(({ logError }) => logError(`invoke:${command}`, error))
        .catch(() => undefined);
    }
    throw error;
  }
}
