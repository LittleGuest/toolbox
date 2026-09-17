<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useMessage } from "naive-ui";
import { Export, Image } from "@vicons/carbon";
import MultiFilePicker from "@/components/pdf/MultiFilePicker.vue";

const message = useMessage();

const IMG_EXTS = ["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif", "tga", "ppm", "pgm", "hdr", "exr", "avif", "qoi"];

const files = ref<string[]>([]);
const loading = ref(false);

const convert = async () => {
  if (!files.value.length) return message.warning("请先添加图片");
  const outputPath = await save({
    defaultPath: "images.pdf",
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!outputPath) return; // 用户取消选择
  loading.value = true;
  try {
    await invoke("images_to_pdf", { inputs: files.value, outputPath: outputPath as string });
    message.success("PDF 已生成");
  } catch (e) {
    message.error(String(e));
  } finally {
    loading.value = false;
  }
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <header class="tb-card-header">
        <div class="tb-card-header-title">
          <span class="tb-card-title-ico"><n-icon><Image /></n-icon></span>
          图片转 PDF
        </div>
        <div class="tb-card-header-actions">
          <span class="tb-card-header-sub">每张图片自动生成一页，按列表顺序排列</span>
        </div>
      </header>

      <MultiFilePicker
        :files="files"
        title="选择图片"
        :extensions="IMG_EXTS"
        :is-image="true"
        empty-hint="支持 PNG / JPG / WEBP / BMP / TIFF 等格式"
        @update="files = $event"
      />

      <div class="tb-submit-row">
        <n-button type="primary" size="large" :loading="loading" class="tb-submit" @click="convert">
          <template #icon><n-icon><Export /></n-icon></template>
          {{ loading ? "正在生成 PDF…" : "生成 PDF" }}
        </n-button>
      </div>
    </section>
  </div>
</template>