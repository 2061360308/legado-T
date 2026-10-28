import { defineStore } from "pinia";
import { dictRules, httpTtsEngines, replaceRules, txtTocRules } from "@/api/rules";
import type { DictRule, HttpTts, ReplaceRule, TxtTocRule } from "@/api/bindings";

export const useRulesStore = defineStore("rules", {
  state: () => ({
    replaceRuleList: [] as ReplaceRule[],
    dictRuleList: [] as DictRule[],
    httpTtsList: [] as HttpTts[],
    txtTocRuleList: [] as TxtTocRule[],
    loading: false,
    error: "" as string,
  }),
  actions: {
    async loadTxtTocRules() {
      this.txtTocRuleList = await txtTocRules();
    },
    async loadAll() {
      this.loading = true;
      this.error = "";
      try {
        const [replace, dict, tts, toc] = await Promise.all([
          replaceRules(),
          dictRules(),
          httpTtsEngines(),
          txtTocRules(),
        ]);
        this.replaceRuleList = replace;
        this.dictRuleList = dict;
        this.httpTtsList = tts;
        this.txtTocRuleList = toc;
      } catch (e) {
        this.error = String(e);
      } finally {
        this.loading = false;
      }
    },
  },
});