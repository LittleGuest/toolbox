<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { useMessage } from "naive-ui";
import { Save, Reset } from "@vicons/carbon";

const message = useMessage();

const language = ref("zh-cn");
const smartDetection = ref(false);
const font = ref("system");
const compactMode = ref(false);

const languageOptions = [
  { label: "简体中文", value: "zh-cn" },
  { label: "English", value: "en-us" },
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
  localStorage.setItem("smartDetection", String(smartDetection.value));
  localStorage.setItem("font", font.value);
  localStorage.setItem("compactMode", String(compactMode.value));
  document.documentElement.style.fontFamily = fontFamily.value;
  document.documentElement.dataset.compactMode = String(compactMode.value);
};

const resetSettings = () => {
  language.value = "zh-cn";
  smartDetection.value = false;
  font.value = "system";
  compactMode.value = false;
  saveSettings();
  message.success("已恢复默认设置");
};

onMounted(() => {
  language.value = localStorage.getItem("language") || "zh-cn";
  smartDetection.value = localStorage.getItem("smartDetection") === "true";
  font.value = localStorage.getItem("font") || "system";
  compactMode.value = localStorage.getItem("compactMode") === "true";
  saveSettings();
});

watch([language, smartDetection, font, compactMode], saveSettings);
</script>

<template>
  <div class="tb-page">
    <div class="tb-card">
      <n-form label-placement="left" label-width="140">
        <n-form-item>
          <n-space>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="saveSettings">
                  <template #icon>
                    <n-icon><Save /></n-icon>
                  </template>
                </n-button>
              </template>
              保存设置
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="resetSettings">
                  <template #icon>
                    <n-icon><Reset /></n-icon>
                  </template>
                </n-button>
              </template>
              恢复默认
            </n-tooltip>
          </n-space>
        </n-form-item>
      </n-form>
    </div>
  </div>
</template>
