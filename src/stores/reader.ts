import { defineStore } from "pinia";

export const useReaderStore = defineStore("reader", {
  state: () => ({
    bookUrl: "" as string,
    chapterIndex: 0,
    pageIndex: 0,
    totalPages: 0,
    preLoading: false,
  }),
  actions: {
    open(bookUrl: string, chapterIndex = 0, pageIndex = 0) {
      this.bookUrl = bookUrl;
      this.chapterIndex = chapterIndex;
      this.pageIndex = pageIndex;
    },
  },
});