<script setup>
import { ref, onMounted, provide } from "vue";
import App from "./App.vue";
import { NConfigProvider, darkTheme } from "naive-ui";

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

// 提供主题状态给子组件
provide("theme", theme);
provide("toggleTheme", toggleTheme);
</script>

<template>
    <n-config-provider :theme="theme === 'dark' ? darkTheme : undefined">
        <n-message-provider placement="top-right">
            <n-loading-bar-provider>
                <App />
            </n-loading-bar-provider>
        </n-message-provider>
    </n-config-provider>
</template>
