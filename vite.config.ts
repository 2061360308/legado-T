import { defineConfig, loadEnv } from "vite";
import vue from "@vitejs/plugin-vue";
import { fileURLToPath, URL } from "node:url";

const host = process.env.TAURI_DEV_HOST;

function pwaHtmlPlugin(target: string) {
  return {
    name: "local-reader-pwa-html",
    transformIndexHtml(html: string) {
      if (target !== "web") return html;
      return {
        html,
        tags: [
          {
            tag: "link",
            attrs: { rel: "manifest", href: "/manifest.webmanifest" },
            injectTo: "head" as const,
          },
          {
            tag: "link",
            attrs: { rel: "icon", href: "/icons/icon-192.png" },
            injectTo: "head" as const,
          },
          {
            tag: "link",
            attrs: { rel: "apple-touch-icon", href: "/icons/icon-192.png" },
            injectTo: "head" as const,
          },
          {
            tag: "meta",
            attrs: { name: "theme-color", content: "#039BE5" },
            injectTo: "head" as const,
          },
          {
            tag: "meta",
            attrs: { name: "mobile-web-app-capable", content: "yes" },
            injectTo: "head" as const,
          },
          {
            tag: "meta",
            attrs: { name: "apple-mobile-web-app-capable", content: "yes" },
            injectTo: "head" as const,
          },
          {
            tag: "meta",
            attrs: { name: "apple-mobile-web-app-status-bar-style", content: "default" },
            injectTo: "head" as const,
          },
        ],
      };
    },
  };
}

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), "");
  const target =
    mode === "web" || env.VITE_RUNTIME === "web"
      ? "web"
      : env.TAURI_ENV_PLATFORM === "android"
        ? "android"
        : "pc";
  const buildId =
    process.env.VITE_BUILD_ID?.trim() ||
    new Date().toISOString().replace(/\D/g, "").slice(0, 14);
  const apiTarget = env.VITE_API_PROXY_TARGET || "http://127.0.0.1:3000";
  const apiProxy = {
    "/api": {
      target: apiTarget,
      changeOrigin: true,
    },
  };

  return {
    plugins: [vue(), pwaHtmlPlugin(target)],
    clearScreen: false,
    define: {
      __BUILD_ID__: JSON.stringify(buildId),
    },
    resolve: {
      alias: {
        "@": fileURLToPath(new URL("./src", import.meta.url)),
        "@platform-impl": fileURLToPath(
          new URL(`./src/adapters/${target}/index.ts`, import.meta.url),
        ),
      },
    },
    server: {
      port: 1420,
      strictPort: true,
      host: host || false,
      hmr: host
        ? {
            protocol: "ws",
            host,
            port: 1421,
          }
        : undefined,
      watch: {
        ignored: ["**/src-tauri/**"],
      },
      proxy: apiProxy,
    },
    preview: {
      proxy: apiProxy,
    },
    envPrefix: ["VITE_", "TAURI_ENV_*"],
    build: {
      target: "es2021",
      minify: "esbuild",
      sourcemap: false,
    },
  };
});
