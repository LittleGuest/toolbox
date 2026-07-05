<script setup lang="ts">
import { computed, inject, onMounted, ref, watch, type Ref } from "vue";
import { useMessage } from "naive-ui";

const message = useMessage();
const injectedTheme = inject<Ref<string>>("theme");

const language = ref("zh-cn");
const theme = ref("light");
const smartDetection = ref(false);
const font = ref("system");
const compactMode = ref(false);

const languageOptions = [
  { label: "简体中文", value: "zh-cn" },
  { label: "English", value: "en-us" },
];

const themeOptions = [
  { label: "浅色", value: "light" },
  { label: "深色", value: "dark" },
];

const fontOptions = [
  { label: "系统默认", value: "system" },
  { label: "等宽字体", value: "monospace" },
  { label: "苹方/微软雅黑优先", value: "sans-cn" },
];

const fontFamily = computed(() => {
  if (font.value === "monospace") {
    return "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace";
  }
  if (font.value === "sans-cn") {
    return '"PingFang SC", "Microsoft YaHei", "Noto Sans CJK SC", sans-serif';
  }
  return "";
});

const saveSettings = () => {
  localStorage.setItem("language", language.value);
  localStorage.setItem("theme", theme.value);
  localStorage.setItem("smartDetection", String(smartDetection.value));
  localStorage.setItem("font", font.value);
  localStorage.setItem("compactMode", String(compactMode.value));
  if (injectedTheme) {
    injectedTheme.value = theme.value;
  }
  document.documentElement.style.fontFamily = fontFamily.value;
  document.documentElement.dataset.compactMode = String(compactMode.value);
};

const resetSettings = () => {
  language.value = "zh-cn";
  theme.value = "light";
  smartDetection.value = false;
  font.value = "system";
  compactMode.value = false;
  saveSettings();
  message.success("已恢复默认设置");
};

onMounted(() => {
  language.value = localStorage.getItem("language") || "zh-cn";
  theme.value = localStorage.getItem("theme") || injectedTheme?.value || "light";
  smartDetection.value = localStorage.getItem("smartDetection") === "true";
  font.value = localStorage.getItem("font") || "system";
  compactMode.value = localStorage.getItem("compactMode") === "true";
  saveSettings();
});

watch([language, theme, smartDetection, font, compactMode], saveSettings);
</script>

<template>
  <n-card title="设置">
    <n-form label-placement="left" label-width="140">
      <n-form-item label="主题">
        <n-select v-model:value="theme" :options="themeOptions" />
      </n-form-item>
      <!--
      <n-form-item label="语言">
        <n-select v-model:value="language" :options="languageOptions" />
      </n-form-item>
      <n-form-item label="字体">
        <n-select v-model:value="font" :options="fontOptions" />
      </n-form-item>
      <n-form-item label="紧凑模式">
        <n-switch v-model:value="compactMode" />
      </n-form-item>
      <n-form-item label="智能检测">
        <n-switch v-model:value="smartDetection" />
        <span class="hint">保存开关状态，后续工具可按该配置读取剪贴板并推荐入口。</span>
      </n-form-item>
      -->
      <n-form-item>
        <n-space>
          <n-button type="primary" @click="saveSettings">保存设置</n-button>
          <n-button @click="resetSettings">恢复默认</n-button>
        </n-space>
      </n-form-item>
    </n-form>
  </n-card>
</template>

<style scoped>
.hint {
  margin-left: 12px;
  color: var(--n-text-color-3);
}
</style>
