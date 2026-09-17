<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useMessage } from "naive-ui";
import { GroupObjects } from "@vicons/carbon";
import MultiFilePicker from "@/components/pdf/MultiFilePicker.vue";

const message = useMessage();

const files = ref<string[]>([]);
const loading = ref(false);

const merge = async () => {
  if (files.value.length < 2) return message.warning("请至少添加两个 PDF");
  const outputPath = await save({
    defaultPath: "merged.pdf",
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!outputPath) return; // 用户取消选择
  loading.value = true;
  try {
    await invoke("pdf_merge", { inputs: files.value, outputPath: outputPath as string });
    message.success("合并成功");
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
          <span class="tb-card-title-ico"><n-icon><GroupObjects /></n-icon></span>
          合并 PDF
        </div>
        <div class="tb-card-header-actions">
          <span class="tb-card-header-sub">按列表顺序合并至少 2 个文件</span>
        </div>
      </header>

      <MultiFilePicker
        :files="files"
        title="选择 PDF"
        :extensions="['pdf']"
        empty-hint="可拖拽多个 PDF，自动按列表顺序合并"
        @update="files = $event"
      />

      <div class="tb-submit-row">
        <n-button type="primary" size="large" :loading="loading" class="tb-submit" @click="merge">
          <template #icon><n-icon><GroupObjects /></n-icon></template>
          {{ loading ? "正在合并…" : "合并 PDF" }}
        </n-button>
      </div>
    </section>
  </div>
</template>