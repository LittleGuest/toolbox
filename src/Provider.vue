<script setup lang="ts">
import { ref, onMounted, provide } from "vue";
import type { GlobalThemeOverrides } from "naive-ui";
import App from "./App.vue";
import { NConfigProvider, darkTheme, zhCN } from "naive-ui";

// 主题状态管理
const theme = ref("light");

// 从本地存储加载主题设置
onMounted(() => {
  const savedTheme = localStorage.getItem("theme");
  if (savedTheme) {
    theme.value = savedTheme;
  }
  applyTheme(theme.value);
});

// 切换主题
const toggleTheme = () => {
  theme.value = theme.value === "light" ? "dark" : "light";
  localStorage.setItem("theme", theme.value);
  applyTheme(theme.value);
};

const applyTheme = (t: string) => {
  document.documentElement.dataset.theme = t;
};

// 提供主题状态给子组件
provide("theme", theme);
provide("toggleTheme", toggleTheme);

// naive-ui 主题变量统一（浅深色保持一致的设计语言）
const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: "#4F6EF7",
    primaryColorHover: "#6a85ff",
    primaryColorPressed: "#3e5ad6",
    primaryColorSuppl: "#4F6EF7",
    borderRadius: "8px",
    borderRadiusSmall: "6px",
    fontSize: "14px",
    fontFamily:
      '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, "PingFang SC", "Microsoft YaHei", sans-serif',
    fontFamilyMono:
      'ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Courier New", monospace',
  },
  Card: {
    borderRadius: "14px",
  },
  Button: {
    borderRadiusSmall: "8px",
    borderRadiusMedium: "10px",
  },
  Input: {
    borderRadius: "10px",
  },
  DataTable: {
    borderRadius: "10px",
  },
};
</script>

<template>
  <n-config-provider
    :theme="theme === 'dark' ? darkTheme : undefined"
    :theme-overrides="themeOverrides"
    :locale="zhCN"
    :style="{
      height: '100%',
      background: 'var(--tb-bg-app)',
    }"
  >
    <n-message-provider placement="top-right">
      <n-loading-bar-provider>
        <n-dialog-provider>
          <App />
        </n-dialog-provider>
      </n-loading-bar-provider>
    </n-message-provider>
  </n-config-provider>
</template>