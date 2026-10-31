import { invoke } from "./backend";
import { COMMANDS, type ReadRecord } from "./bindings";

export function readRecordAddTime(bookUrl: string, seconds: number): Promise<void> {
  return invoke<void>(COMMANDS.readRecordAddTime, { bookUrl, seconds });
}

export function readRecordList(): Promise<ReadRecord[]> {
  return invoke<ReadRecord[]>(COMMANDS.readRecordList);
}

export function readRecordDelete(deviceId: string, bookName: string): Promise<void> {
  return invoke<void>(COMMANDS.readRecordDelete, { deviceId, bookName });
}

export function readRecordClear(): Promise<void> {
  return invoke<void>(COMMANDS.readRecordClear);
}
