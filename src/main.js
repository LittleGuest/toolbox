import { createApp, ref, onMounted } from "vue";
import App from "./Provider.vue";
import router from "./router";

const app = createApp(App);

// 主题状态管理
const theme = ref("light");

// 从本地存储加载主题设置
onMounted(() => {
  const savedTheme = localStorage.getItem("theme");
  if (savedTheme) {
    theme.value = savedTheme;
  }
});

// 切换主题
const toggleTheme = () => {
  theme.value = theme.value === "light" ? "dark" : "light";
  localStorage.setItem("theme", theme.value);
};

// 注册全局属性
app.config.globalProperties.theme = theme;
app.config.globalProperties.toggleTheme = toggleTheme;

app.use(router).mount("#app");
