import { invoke } from "./backend";
import {
  COMMANDS,
  type DictRule,
  type DiffReport,
  type HttpTts,
  type ReplaceRule,
  type TxtTocRule,
} from "./bindings";

export function txtTocRules(): Promise<TxtTocRule[]> {
  return invoke<TxtTocRule[]>(COMMANDS.txtTocRuleList);
}

export function txtTocRuleSave(rule: TxtTocRule): Promise<TxtTocRule> {
  return invoke<TxtTocRule>(COMMANDS.txtTocRuleSave, { rule });
}

export function txtTocRuleDelete(id: number): Promise<void> {
  return invoke<void>(COMMANDS.txtTocRuleDelete, { id });
}

export function txtTocRuleReorder(ids: number[]): Promise<void> {
  return invoke<void>(COMMANDS.txtTocRuleReorder, { ids });
}

export function txtTocRuleRestoreDefault(): Promise<number> {
  return invoke<number>(COMMANDS.txtTocRuleRestoreDefault);
}

export function txtTocRuleImport(json: string): Promise<DiffReport> {
  return invoke<DiffReport>(COMMANDS.txtTocRuleImport, { json });
}

export function txtTocRuleExport(): Promise<string> {
  return invoke<string>(COMMANDS.txtTocRuleExport);
}

export async function replaceRules(): Promise<ReplaceRule[]> {
  return invoke<ReplaceRule[]>("replace_rule_list");
}

export async function replaceRuleSave(rule: ReplaceRule): Promise<ReplaceRule> {
  return invoke<ReplaceRule>("replace_rule_save", { rule });
}

export async function replaceRuleDelete(id: number): Promise<void> {
  return invoke<void>("replace_rule_delete", { id });
}

export async function replaceRuleImport(json: string): Promise<DiffReport> {
  return invoke<DiffReport>("replace_rule_import", { json });
}

export async function replaceRuleExport(): Promise<string> {
  return invoke<string>("replace_rule_export");
}

export async function dictRules(): Promise<DictRule[]> {
  return invoke<DictRule[]>("dict_rule_list");
}

export async function dictRuleSave(rule: DictRule): Promise<DictRule> {
  return invoke<DictRule>("dict_rule_save", { rule });
}

export async function dictRuleDelete(name: string): Promise<void> {
  return invoke<void>("dict_rule_delete", { name });
}

export async function dictRuleImport(json: string): Promise<DiffReport> {
  return invoke<DiffReport>("dict_rule_import", { json });
}

export async function dictRuleExport(): Promise<string> {
  return invoke<string>("dict_rule_export");
}

export async function httpTtsEngines(): Promise<HttpTts[]> {
  return invoke<HttpTts[]>("http_tts_list");
}

export async function httpTtsSave(engine: HttpTts): Promise<HttpTts> {
  return invoke<HttpTts>("http_tts_save", { engine });
}

export async function httpTtsDelete(id: number): Promise<void> {
  return invoke<void>("http_tts_delete", { id });
}

export async function httpTtsImport(json: string): Promise<DiffReport> {
  return invoke<DiffReport>("http_tts_import", { json });
}

export async function httpTtsExport(): Promise<string> {
  return invoke<string>("http_tts_export");
}
