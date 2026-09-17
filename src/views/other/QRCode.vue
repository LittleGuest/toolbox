<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import QRCode from "qrcode";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { MagicWand, Paste, Copy, Download } from "@vicons/carbon";

const message = useMessage();

const text = ref("https://github.com/");
const size = ref(260);
const margin = ref(2);
const darkColor = ref("#000000");
const lightColor = ref("#ffffff");
const errorCorrectionLevel = ref<"L" | "M" | "Q" | "H">("M");
const dataUrl = ref("");
const canvasRef = ref<HTMLCanvasElement | null>(null);

const canGenerate = computed(() => text.value.trim().length > 0);

const generate = async () => {
  if (!canGenerate.value) {
    dataUrl.value = "";
    return;
  }

  await nextTick();
  const options = {
    width: size.value,
    margin: margin.value,
    errorCorrectionLevel: errorCorrectionLevel.value,
    color: {
      dark: darkColor.value,
      light: lightColor.value,
    },
  };

  if (canvasRef.value) {
    await QRCode.toCanvas(canvasRef.value, text.value, options);
  }
  dataUrl.value = await QRCode.toDataURL(text.value, options);
};

const copyDataUrl = async () => {
  if (!dataUrl.value) {
    message.warning("请先生成二维码");
    return;
  }
  await writeText(dataUrl.value);
  message.success("已复制二维码 Data URL");
};

const pasteText = async () => {
  text.value = await readText();
  await generate();
};

const download = () => {
  if (!dataUrl.value) {
    message.warning("请先生成二维码");
    return;
  }
  const link = document.createElement("a");
  link.href = dataUrl.value;
  link.download = `qrcode-${Date.now()}.png`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
};

watch([text, size, margin, darkColor, lightColor, errorCorrectionLevel], generate);

onMounted(generate);
</script>

<template>
  <div class="tb-page">
    <section class="tb-card qr-card">
      <div class="tb-card-header">
        <span class="tb-card-header-title">二维码生成</span>
        <div class="tb-card-header-actions">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" @click="generate">
                <template #icon>
                  <n-icon><MagicWand /></n-icon>
                </template>
              </n-button>
            </template>
            生成
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="pasteText">
                <template #icon>
                  <n-icon><Paste /></n-icon>
                </template>
              </n-button>
            </template>
            粘贴内容
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="copyDataUrl">
                <template #icon>
                  <n-icon><Copy /></n-icon>
                </template>
              </n-button>
            </template>
            复制 Data URL
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="download">
                <template #icon>
                  <n-icon><Download /></n-icon>
                </template>
              </n-button>
            </template>
            下载 PNG
          </n-tooltip>
        </div>
      </div>

      <div class="qr-body">
        <div class="qr-form">
          <n-form label-placement="left" label-width="105">
            <n-form-item label="内容">
              <n-input
                v-model:value="text"
                type="textarea"
                :autosize="{ minRows: 9, maxRows: 14 }"
                placeholder="输入文本、链接或其它需要编码的内容"
              />
            </n-form-item>
            <n-form-item label="尺寸">
              <n-slider v-model:value="size" :min="120" :max="720" :step="20" />
            </n-form-item>
            <n-form-item label="边距">
              <n-input-number v-model:value="margin" :min="0" :max="10" />
            </n-form-item>
            <n-form-item label="纠错级别">
              <n-radio-group v-model:value="errorCorrectionLevel">
                <n-radio-button value="L">L</n-radio-button>
                <n-radio-button value="M">M</n-radio-button>
                <n-radio-button value="Q">Q</n-radio-button>
                <n-radio-button value="H">H</n-radio-button>
              </n-radio-group>
            </n-form-item>
            <n-form-item label="前景色">
              <n-color-picker v-model:value="darkColor" :show-alpha="false" />
            </n-form-item>
            <n-form-item label="背景色">
              <n-color-picker v-model:value="lightColor" :show-alpha="false" />
            </n-form-item>
          </n-form>
        </div>
        <div class="qr-preview">
          <span class="qr-preview-title">预览</span>
          <div class="qr-preview-box">
            <canvas ref="canvasRef" />
          </div>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.qr-card {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.qr-body {
  flex: 1;
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 0.9fr);
  gap: var(--tb-gap);
}

.qr-form {
  min-width: 0;
  overflow-y: auto;
  padding-right: 4px;
}

.qr-form :deep(.n-form-item) {
  margin-bottom: 14px;
}

.qr-preview {
  display: flex;
  flex-direction: column;
  min-width: 0;
  min-height: 360px;
  border: 1px dashed var(--tb-border);
  border-radius: var(--tb-radius-m);
  padding: 14px;
  background: var(--tb-bg-app);
}

.qr-preview-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--tb-text-3);
  letter-spacing: 0.06em;
  text-transform: uppercase;
  margin-bottom: 8px;
}

.qr-preview-box {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.qr-preview-box canvas {
  max-width: 100%;
  max-height: 100%;
  border-radius: 8px;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.08);
}
</style>
