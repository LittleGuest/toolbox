<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
import QRCode from "qrcode";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";

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
  <n-card title="二维码生成">
    <n-grid :cols="2" :x-gap="24" responsive="screen">
      <n-grid-item>
        <n-form label-placement="left" label-width="110">
          <n-form-item label="内容">
            <n-input
              v-model:value="text"
              type="textarea"
              :autosize="{ minRows: 8, maxRows: 16 }"
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
          <n-space>
            <n-button type="primary" @click="generate">生成</n-button>
            <n-button @click="pasteText">粘贴内容</n-button>
            <n-button @click="copyDataUrl">复制 Data URL</n-button>
            <n-button @click="download">下载 PNG</n-button>
          </n-space>
        </n-form>
      </n-grid-item>
      <n-grid-item>
        <div class="qr-preview">
          <canvas ref="canvasRef" />
        </div>
      </n-grid-item>
    </n-grid>
  </n-card>
</template>

<style scoped>
.qr-preview {
  min-height: 360px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px dashed var(--n-border-color);
  border-radius: 8px;
}
</style>
