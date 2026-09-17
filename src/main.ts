import { createApp } from "vue";
import App from "./Provider.vue";
import router from "./router";
import "./styles/global.scss";

const app = createApp(App);

app.use(router).mount("#app");
