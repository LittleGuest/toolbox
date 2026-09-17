<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useMessage } from "naive-ui";
import { Export, PageNumber } from "@vicons/carbon";
import PdfInputTile from "@/components/pdf/PdfInputTile.vue";

const message = useMessage();

const input = ref("");
const position = ref("bottom-center");
const fontSize = ref(12);
const formatPattern = ref("{n}");
const startAt = ref(1);
const loading = ref(false);

const positionOptions = [
  { label: "顶部靠左", value: "top-left" },
  { label: "顶部居中", value: "top-center" },
  { label: "顶部靠右", value: "top-right" },
  { label: "中部靠左", value: "middle-left" },
  { label: "中部居中", value: "middle-center" },
  { label: "中部靠右", value: "middle-right" },
  { label: "底部靠左", value: "bottom-left" },
  { label: "底部居中", value: "bottom-center" },
  { label: "底部靠右", value: "bottom-right" },
];

// 位置 -> 3x3 网格中的下标
const positionIndex = computed(() => {
  const rowMap: Record<string, number> = { top: 0, middle: 3, bottom: 6 };
  const colMap: Record<string, number> = { left: 0, center: 1, right: 2 };
  const [row, col] = position.value.split("-");
  return (rowMap[row] ?? 0) + (colMap[col] ?? 1);
});

// 把占位符应用为可读的示例文案
const sampleText = computed(() => {
  const s = startAt.value || 1;
  return formatPattern.value
    .replace(/\{n\}/g, String(s))
    .replace(/\{cur\}/g, "1")
    .replace(/\{total\}/g, "10");
});

const addNumbers = async () => {
  if (!input.value) return message.warning("请选择 PDF");
  const outputPath = await save({
    defaultPath: "numbered.pdf",
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!outputPath) return; // 用户取消选择
  loading.value = true;
  try {
    await invoke("pdf_add_page_numbers", {
      input: input.value,
      outputPath: outputPath as string,
      position: position.value,
      font_size: Number(fontSize.value) || 12,
      format_pattern: formatPattern.value,
      start_at: Number(startAt.value) || 1,
    });
    message.success("页码添加完成");
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
          <span class="tb-card-title-ico"><n-icon><PageNumber /></n-icon></span>
          添加页码
        </div>
        <div class="tb-card-header-actions">
          <span class="tb-card-header-sub">为 PDF 每页插入页码或自定义文本</span>
        </div>
      </header>

      <div>
        <span class="tb-editor-label">源文件</span>
        <PdfInputTile v-model="input" />
      </div>

      <div style="margin-top: 18px">
        <span class="tb-editor-label">页码设置</span>

        <div class="pn-grid">
          <!-- 左侧：配置项 -->
          <div class="pn-config">
            <div class="pn-row">
              <span class="pn-label">位置</span>
              <n-select v-model:value="position" :options="positionOptions" style="flex: 1; min-width: 140px" />
            </div>
            <div class="pn-row">
              <span class="pn-label">字号</span>
              <n-input-number v-model:value="fontSize" :min="6" :max="72" style="width: 110px" />
              <span class="pn-label pn-sub">pt</span>
            </div>
            <div class="pn-row">
              <span class="pn-label">起始页</span>
              <n-input-number v-model:value="startAt" :min="1" style="flex: 1; min-width: 110px" />
            </div>
            <div class="pn-row">
              <span class="pn-label">格式</span>
              <n-input v-model:value="formatPattern" placeholder="如 {n}" class="tb-mono" style="flex: 1; min-width: 140px" />
            </div>
          </div>

          <!-- 右侧：可视化预览 -->
          <div class="pn-preview">
            <div class="pn-preview-title">预览</div>
            <div class="pn-preview-inner">
              <div class="pn-preview-page">
                <div
                  v-for="k in 9"
                  :key="k"
                  class="pn-slot"
                  :class="{ 'pn-slot--active': k - 1 === positionIndex }"
                >
                  <span
                    v-if="k - 1 === positionIndex"
                    class="pn-num"
                    :style="{ fontSize: Math.min(Math.max(fontSize, 6), 26) + 'px' }"
                  >{{ sampleText }}</span>
                </div>
              </div>
              <div class="pn-preview-tip">
                <span class="pn-dot" />可拖拽下方 9 个方位选择页码位置
              </div>
            </div>
          </div>
        </div>

        <div class="pn-slots">
          <button
            v-for="(opt, i) in positionOptions"
            :key="opt.value"
            class="pn-slot-btn"
            :class="{ 'pn-slot-btn--active': position === opt.value }"
            @click="position = opt.value"
          >
            <span class="pn-slot-ico" />
          </button>
        </div>
        <p class="tb-hint" style="margin: 8px 0 0">
          占位符：<b class="pn-code">{n}</b> 起始页递增 · <b class="pn-code">{cur}</b> 文档顺序号 · <b class="pn-code">{total}</b> 总页数
        </p>
      </div>

      <div class="tb-submit-row">
        <n-button type="primary" size="large" :loading="loading" class="tb-submit" @click="addNumbers">
          <template #icon><n-icon><Export /></n-icon></template>
          {{ loading ? "正在添加…" : "添加页码" }}
        </n-button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.pn-grid {
  display: grid;
  grid-template-columns: 1fr 220px;
  gap: 18px;
  align-items: stretch;
}
.pn-config {
  display: flex;
  flex-direction: column;
  gap: 12px;
}
.pn-row {
  display: flex;
  align-items: center;
  gap: 10px;
}
.pn-label {
  font-size: 13px;
  color: var(--tb-text-2);
  width: 56px;
  flex: 0 0 auto;
}
.pn-sub {
  width: auto;
}
.pn-preview {
  background: var(--tb-bg-app);
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.pn-preview-title {
  font-size: 12px;
  font-weight: 600;
  color: var(--tb-text-3);
  letter-spacing: 0.06em;
  text-transform: uppercase;
}
.pn-preview-inner {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
.pn-preview-page {
  aspect-ratio: 3 / 4;
  width: 100%;
  border: 1px solid var(--tb-border-strong);
  border-radius: 6px;
  background: #fff;
  display: grid;
  grid-template-rows: repeat(3, 1fr);
  grid-template-columns: repeat(3, 1fr);
}
.pn-slot {
  display: grid;
  place-items: center;
  min-height: 0;
}
.pn-num {
  color: var(--tb-text);
  line-height: 1;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.pn-preview-tip {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 11.5px;
  color: var(--tb-text-3);
}
.pn-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--tb-primary);
}
.pn-slots {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
  margin-top: 14px;
}
.pn-slot-btn {
  height: 34px;
  border: 1px solid var(--tb-border);
  border-radius: 8px;
  background: var(--tb-bg-app);
  cursor: pointer;
  display: grid;
  place-items: center;
  transition: border-color 0.15s ease, background 0.15s ease;
}
.pn-slot-btn:hover {
  border-color: var(--tb-border-strong);
}
.pn-slot-btn--active {
  border-color: var(--tb-primary);
  background: var(--tb-primary-weak);
}
.pn-slot-ico {
  width: 8px;
  height: 8px;
  border-radius: 2px;
  background: var(--tb-text-3);
}
.pn-slot-btn--active .pn-slot-ico {
  background: var(--tb-primary);
}
.pn-code {
  color: var(--tb-primary);
  font-family: var(--tb-font-mono);
}
@media (max-width: 760px) {
  .pn-grid {
    grid-template-columns: 1fr;
  }
}
</style>