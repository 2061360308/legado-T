import { createRouter, createWebHashHistory, type RouteRecordRaw } from "vue-router";
import navigationGraph from "@/design/reports/navigation-graph.json";

const shellStart = (navigationGraph as { start?: string }).start ?? "activity_welcome";

const routes: RouteRecordRaw[] = [
  { path: "/", name: "bookshelf", component: () => import("@/views/bookshelf/BookshelfView.vue") },
  { path: "/import", name: "import", component: () => import("@/views/import/ImportView.vue") },
  {
    path: "/remote-books",
    name: "remote-books",
    component: () => import("@/views/remote/RemoteBooksView.vue"),
  },
  {
    path: "/arrange-books",
    name: "arrange-books",
    component: () => import("@/views/bookshelf/ArrangeBooksView.vue"),
  },
  {
    path: "/group-manage",
    name: "group-manage",
    component: () => import("@/views/bookshelf/GroupManageView.vue"),
  },
  {
    path: "/book-info",
    name: "book-info",
    component: () => import("@/views/bookshelf/BookInfoView.vue"),
  },
  {
    path: "/book-info-edit",
    name: "book-info-edit",
    component: () => import("@/views/bookshelf/BookInfoEditView.vue"),
  },
  { path: "/reader", name: "reader", component: () => import("@/views/reader/ReaderView.vue") },
  { path: "/audio", name: "audio", component: () => import("@/views/audio/AudioPlayView.vue") },
  {
    path: "/book-search",
    name: "book-search",
    component: () => import("@/views/search/BookSearchView.vue"),
  },
  {
    path: "/book-source",
    name: "book-source",
    component: () => import("@/views/source/BookSourceView.vue"),
  },
  {
    path: "/book-source-edit",
    name: "book-source-edit",
    component: () => import("@/views/source/BookSourceEditView.vue"),
  },
  {
    path: "/book-source-debug",
    name: "book-source-debug",
    component: () => import("@/views/source/BookSourceDebugView.vue"),
  },
  {
    path: "/source-login",
    name: "source-login",
    component: () => import("@/views/source/SourceLoginView.vue"),
  },
  {
    path: "/book-source-variables",
    name: "book-source-variables",
    component: () => import("@/views/source/BookSourceVariablesView.vue"),
  },
  {
    path: "/cache-books",
    name: "cache-books",
    component: () => import("@/views/source/CacheBookView.vue"),
  },
  { path: "/toc", name: "toc", component: () => import("@/views/toc/TocView.vue") },
  {
    path: "/toc-rules",
    name: "toc-rules",
    component: () => import("@/views/toc/TocRulesView.vue"),
  },
  { path: "/replace", name: "replace", component: () => import("@/views/replace/ReplaceView.vue") },
  {
    path: "/replace-edit",
    name: "replace-edit",
    component: () => import("@/views/replace/ReplaceEditView.vue"),
  },
  { path: "/dict", name: "dict", component: () => import("@/views/dict/DictView.vue") },
  {
    path: "/dict-rules",
    name: "dict-rules",
    component: () => import("@/views/dict/DictRulesView.vue"),
  },
  { path: "/config", name: "config", component: () => import("@/views/config/ConfigView.vue") },
  {
    path: "/theme",
    name: "theme",
    component: () => import("@/views/config/ThemeSettingsView.vue"),
  },
  {
    path: "/cover-config",
    name: "cover-config",
    component: () => import("@/views/config/CoverConfigView.vue"),
  },
  {
    path: "/welcome-config",
    name: "welcome-config",
    component: () => import("@/views/config/WelcomeConfigView.vue"),
  },
  {
    path: "/welcome",
    name: "welcome",
    component: () => import("@/views/welcome/WelcomeView.vue"),
  },
  {
    path: "/read-schemes",
    name: "read-schemes",
    component: () => import("@/views/config/ReadSchemesView.vue"),
  },
  {
    path: "/backup",
    name: "backup",
    component: () => import("@/views/config/BackupView.vue"),
  },
  {
    path: "/webdav",
    name: "webdav",
    redirect: "/backup",
  },
  { path: "/aloud", name: "aloud", component: () => import("@/views/aloud/AloudView.vue") },
  {
    path: "/other-settings",
    name: "other-settings",
    component: () => import("@/views/config/OtherSettingsView.vue"),
  },
  {
    path: "/bookmarks",
    name: "bookmarks",
    component: () => import("@/views/config/BookmarksView.vue"),
  },
  {
    path: "/read-records",
    name: "read-records",
    component: () => import("@/views/config/ReadRecordsView.vue"),
  },
  {
    path: "/files",
    name: "files",
    component: () => import("@/views/config/FileManageView.vue"),
  },
  {
    path: "/about",
    name: "about",
    component: () => import("@/views/config/AboutView.vue"),
  },
  { path: "/explore", name: "explore", component: () => import("@/views/misc/ExploreView.vue") },
  {
    path: "/explore-show",
    name: "explore-show",
    component: () => import("@/views/misc/ExploreShowView.vue"),
  },
  {
    path: "/rss-sources",
    name: "rss-sources",
    component: () => import("@/views/misc/RssSourcesView.vue"),
  },
  {
    path: "/rss-articles",
    name: "rss-articles",
    component: () => import("@/views/misc/RssArticlesView.vue"),
  },
  {
    path: "/legacy",
    name: "legacy",
    component: () => import("@/views/legacy/LegacyGalleryView.vue"),
  },
  {
    path: "/generated",
    name: "generated",
    component: () => import("@/views/legacy/GeneratedGalleryView.vue"),
  },
  {
    path: "/generated/:kind/:name",
    name: "generated-xml",
    component: () => import("@/views/legacy/GeneratedXmlPage.vue"),
  },
  { path: "/shell", redirect: `/shell/${shellStart}` },
  {
    path: "/shell/:name",
    name: "shell",
    component: () => import("@/views/legacy/AppShellView.vue"),
  },
  {
    path: "/legacy/menu/:name",
    name: "legacy-menu",
    component: () => import("@/views/legacy/LegacyMenuPage.vue"),
  },
  {
    path: "/legacy/pref/:name",
    name: "legacy-pref",
    component: () => import("@/views/legacy/LegacyPrefPage.vue"),
  },
  {
    path: "/legacy/:name",
    name: "legacy-layout",
    component: () => import("@/views/legacy/LegacyLayoutPage.vue"),
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;