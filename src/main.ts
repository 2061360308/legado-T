import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import "./styles/global.css";
import { refreshPlatform } from "./platform";
import { useThemeStore } from "./stores/theme";

const app = createApp(App);

app.use(createPinia());
app.use(router);

const theme = useThemeStore();
theme.applyToDocument();
theme.watchSystem();

const WELCOME_KEY = "local-reader:welcome-shown";

async function bootstrap() {
  await refreshPlatform();
  theme.applyToDocument();
  if (!sessionStorage.getItem(WELCOME_KEY)) {
    sessionStorage.setItem(WELCOME_KEY, "1");
    await router.replace("/welcome");
  }
  app.mount("#app");
}

void bootstrap();

window.addEventListener("resize", () => {
  void refreshPlatform();
});
