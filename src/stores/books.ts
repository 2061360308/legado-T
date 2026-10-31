import { defineStore } from "pinia";
import { bookList } from "@/core/application/books";
import type { Book } from "@/core/domain";

export const useBooksStore = defineStore("books", {
  state: () => ({
    list: [] as Book[],
    loading: false,
    error: "" as string,
  }),
  getters: {
    count: (s) => s.list.length,
  },
  actions: {
    async load() {
      this.loading = true;
      this.error = "";
      try {
        this.list = await bookList();
      } catch (e) {
        this.error = String(e);
      } finally {
        this.loading = false;
      }
    },
    setList(list: Book[]) {
      this.list = list;
    },
    setError(error: string) {
      this.error = error;
    },
  },
});