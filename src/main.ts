import { createApp } from "vue";
import { createPinia } from "pinia";
import App from "./App.vue";
import router from "./router";
import "./styles/global.css";
import { useThemeStore } from "./stores/theme";

const app = createApp(App);

app.use(createPinia());
app.use(router);

const theme = useThemeStore();
theme.applyToDocument();
theme.watchSystem();

app.mount("#app");