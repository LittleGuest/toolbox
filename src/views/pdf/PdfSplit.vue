<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useMessage } from "naive-ui";
import { Checkmark, Cut } from "@vicons/carbon";
import PdfInputTile from "@/components/pdf/PdfInputTile.vue";

const message = useMessage();

const input = ref("");
const ranges = ref("1-3,5,7-9");
const outputs = ref<string[]>([]);
const loading = ref(false);

const shortName = (p: string) => p.split(/[\\/]/).pop() || p;

// 解析页码范围：支持 "3" / "1-5" / "1,3-4"，返回错误信息或成功段列表
const rangeError = computed(() => {
  if (!ranges.value.trim()) return "请填写页码范围";
  const tokens = ranges.value.split(",").map((t) => t.trim()).filter(Boolean);
  if (!tokens.length) return "页码范围不能为空";
  const invalid = tokens.filter((t) => !/^\d+(-\d+)?$/.test(t));
  if (invalid.length) return `无法识别的范围：${invalid.join("、")}`;
  return "";
});

const split = async () => {
  if (!input.value) return message.warning("请选择 PDF");
  if (rangeError.value) return message.warning(rangeError.value);
  const outputDir = (await open({ directory: true, multiple: false })) as string | null;
  if (!outputDir) return; // 用户取消选择
  loading.value = true;
  try {
    outputs.value = (await invoke("pdf_split", {
      input: input.value,
      outputDir,
      ranges: ranges.value,
    })) as string[];
    message.success(`已生成 ${outputs.value.length} 个文件`);
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
          <span class="tb-card-title-ico"><n-icon><Cut /></n-icon></span>
          拆分 PDF
        </div>
        <div class="tb-card-header-actions">
          <span class="tb-card-header-sub">按页码范围将 PDF 拆分为多个文件</span>
        </div>
      </header>

      <div>
        <span class="tb-editor-label">源文件</span>
        <PdfInputTile v-model="input" />
      </div>

      <div style="margin-top: 18px">
        <span class="tb-editor-label">页码范围</span>
        <n-input v-model:value="ranges" clearable placeholder="例如 1-3, 5, 7-9" class="tb-mono">
          <template #prefix><span style="color: var(--tb-text-3)">0</span></template>
        </n-input>
        <p class="tb-hint" style="margin-top: 6px">
          每个范围输出为一个 PDF：单页用逗号分隔，连续页用连字符。例如
          <b class="tb-hint-code">1-3, 5, 7-9</b>
        </p>
        <n-text v-if="rangeError" type="error" style="font-size: 12.5px">{{ rangeError }}</n-text>
        <n-text v-else-if="ranges.trim()" type="success" style="font-size: 12.5px">
          格式有效，将按 {{ ranges.split(",").length }} 个范围分别输出
        </n-text>
      </div>

      <div v-if="outputs.length" style="margin-top: 20px">
        <span class="tb-editor-label">已生成 {{ outputs.length }} 个文件</span>
        <div class="out-list">
          <div v-for="(p, i) in outputs" :key="i" class="out-item">
            <span class="out-item-ico"><n-icon><Checkmark /></n-icon></span>
            <span class="out-item-name">{{ shortName(p) }}</span>
          </div>
        </div>
      </div>

      <div class="tb-submit-row">
        <n-button type="primary" size="large" :loading="loading" class="tb-submit" @click="split">
          <template #icon><n-icon><Cut /></n-icon></template>
          {{ loading ? "正在拆分…" : "拆分 PDF" }}
        </n-button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.tb-file-tile__value.empty {
  color: var(--tb-text-3);
}
.tb-hint-code {
  color: var(--tb-primary);
  font-family: var(--tb-font-mono);
}
.out-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.out-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  background: var(--tb-bg-app);
}
.out-item-ico {
  display: grid;
  place-items: center;
  width: 22px;
  height: 22px;
  border-radius: 6px;
  background: rgba(24, 160, 88, 0.12);
  color: #18a058;
}
.out-item-name {
  font-size: 13px;
  color: var(--tb-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>