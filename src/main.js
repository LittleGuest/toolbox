import { createApp } from "vue";
import "./styles.css";
import App from "./App.vue";
import ElementPlus from "element-plus";
import router from "./router";

createApp(App).use(router).use(ElementPlus).mount("#app");
