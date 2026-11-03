/// <reference lib="webworker" />
import { engineHandlers } from "./engineWasmCore";
import type {
  EngineOperationName,
  EngineRequest,
  EngineResponseMessage,
} from "./engineProtocol";

async function run(op: string, args: unknown[]): Promise<unknown> {
  if (!Object.prototype.hasOwnProperty.call(engineHandlers, op)) {
    throw new Error(`未知引擎操作：${op}`);
  }
  const handler = engineHandlers[op as EngineOperationName] as (
    ...args: unknown[]
  ) => Promise<unknown>;
  return handler(...args);
}

self.onmessage = async (event: MessageEvent<EngineRequest>) => {
  const { id, op, args } = event.data;
  try {
    const result = await run(op, args);
    const response: EngineResponseMessage = { id, result };
    self.postMessage(response);
  } catch (error) {
    const response: EngineResponseMessage = {
      id,
      error: error instanceof Error ? error.message : String(error),
    };
    self.postMessage(response);
  }
};
