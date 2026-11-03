export type RuleImportState = "new" | "update" | "existing";

export interface RuleImportItem {
  key: string;
  title: string;
  subtitle?: string;
  state: RuleImportState;
  selected: boolean;
}

export interface RuleImportRow<T> {
  value: T;
  item: RuleImportItem;
}

export function parseRuleList<T>(
  text: string,
  wrapperKeys: string[] = [
    "sources",
    "bookSources",
    "rssSources",
    "rules",
    "items",
    "replaceRules",
    "dictRules",
    "txtTocRules",
    "httpTts",
    "engines",
  ],
): T[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw new Error("JSON 解析失败，请检查规则格式");
  }
  let list: unknown[];
  if (Array.isArray(parsed)) {
    list = parsed;
  } else if (parsed && typeof parsed === "object") {
    const record = parsed as Record<string, unknown>;
    const wrapper = wrapperKeys.find((key) => Array.isArray(record[key]));
    list = wrapper ? (record[wrapper] as unknown[]) : [parsed];
  } else {
    throw new Error("规则内容不是 JSON 数组");
  }
  return list.filter(
    (item): item is T => Boolean(item && typeof item === "object"),
  );
}

export function stateByUpdateTime(
  oldTime?: number | null,
  newTime?: number | null,
): RuleImportState {
  if (oldTime === undefined || oldTime === null) return "new";
  return Number(newTime || 0) > Number(oldTime || 0) ? "update" : "existing";
}

export function stateByContent(
  existing: unknown,
  incoming: unknown,
): RuleImportState {
  if (!existing) return "new";
  return JSON.stringify(existing) === JSON.stringify(incoming)
    ? "existing"
    : "update";
}

export function selectedByState(
  state: RuleImportState,
  allowUpdate = true,
): boolean {
  return state === "new" || (allowUpdate && state === "update");
}
