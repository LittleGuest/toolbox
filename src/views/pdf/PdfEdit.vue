<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import { useMessage } from "naive-ui";
import { ArrowDown, ArrowUp, Checkmark, PageNumber as PageIcon, Rotate, TrashCan } from "@vicons/carbon";
import PdfInputTile from "@/components/pdf/PdfInputTile.vue";

const message = useMessage();

const input = ref("");
const totalText = ref("");
const loading = ref(false);
const saved = ref(false);

const pages = ref<{ no: number; rotate: number; delete: boolean }[]>([]);

const applyTotal = () => {
  const n = parseInt(totalText.value);
  if (!n || n < 1) return message.warning("请输入有效页数");
  pages.value = Array.from({ length: n }, (_, i) => ({ no: i + 1, rotate: 0, delete: false }));
  saved.value = false;
};

const toggleDelete = (i: number) => {
  pages.value[i].delete = !pages.value[i].delete;
};
const rotate = (i: number) => {
  pages.value[i].rotate = (pages.value[i].rotate + 90) % 360;
};
const move = (i: number, dir: number) => {
  const j = i + dir;
  if (j < 0 || j >= pages.value.length) return;
  [pages.value[i], pages.value[j]] = [pages.value[j], pages.value[i]];
};

const stats = computed(() => ({
  del: pages.value.filter((p) => p.delete).length,
  rot: pages.value.filter((p) => p.rotate % 360 !== 0).length,
  keep: pages.value.filter((p) => !p.delete).length,
}));

const exportPdf = async () => {
  if (!input.value) return message.warning("请选择 PDF");
  if (!pages.value.length) return message.warning("请先设置总页数");
  const outputPath = await save({
    defaultPath: "edited.pdf",
    filters: [{ name: "PDF", extensions: ["pdf"] }],
  });
  if (!outputPath) return; // 用户取消选择
  const deletePages = pages.value.filter((p) => p.delete).map((p) => p.no);
  const rotateVec = pages.value
    .filter((p) => p.rotate % 360 !== 0)
    .map((p) => [p.no, (p.rotate % 360) as number]);
  const order = pages.value.filter((p) => !p.delete).map((p) => p.no);
  loading.value = true;
  try {
    await invoke("pdf_edit", {
      input: input.value,
      outputPath: outputPath as string,
      delete: deletePages,
      rotate: rotateVec,
      order,
    });
    message.success("编辑完成");
    saved.value = true;
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
          <span class="tb-card-title-ico"><n-icon><PageIcon /></n-icon></span>
          编辑 PDF
        </div>
        <div class="tb-card-header-actions">
          <span class="tb-card-header-sub">删除 / 旋转 / 调整页面顺序</span>
        </div>
      </header>

      <div>
        <span class="tb-editor-label">源文件</span>
        <PdfInputTile v-model="input" />
      </div>

      <div style="margin-top: 18px">
        <span class="tb-editor-label">页面数</span>
        <div class="pn-row">
          <n-input v-model:value="totalText" placeholder="输入总页数" style="width: 160px" class="tb-mono">
            <template #prefix><span style="color: var(--tb-text-3)">N =</span></template>
          </n-input>
          <n-button type="primary" secondary :disabled="!totalText.trim()" @click="applyTotal">
            {{ pages.length ? "重新载入" : "载入页面" }}
          </n-button>
          <span class="tb-hint">当前工具需手动指定总页数后逐页操作</span>
        </div>
      </div>

      <template v-if="pages.length">
        <div class="ed-stats">
          <span class="ed-stat"><b>{{ pages.length }}</b> 总页</span>
          <span class="ed-stat" :class="{ 'ed-stat--warn': stats.del }"><b>{{ stats.del }}</b> 删除</span>
          <span class="ed-stat" :class="{ 'ed-stat--accent': stats.rot }"><b>{{ stats.rot }}</b> 旋转</span>
          <span class="ed-stat ed-stat--ok"><b>{{ stats.keep }}</b> 保留</span>
        </div>

        <div style="margin-top: 14px">
          <span class="tb-editor-label">页面操作</span>
          <div class="ed-list">
            <div
              v-for="(pg, i) in pages"
              :key="pg.no"
              class="ed-item"
              :class="{ 'ed-item--del': pg.delete, 'ed-item--rot': pg.rotate % 360 !== 0 }"
            >
              <span class="ed-item-seq">{{ i + 1 }}</span>
              <div class="ed-item-main">
                <div class="ed-item-title">
                  <b>第 {{ pg.no }} 页</b>
                  <template v-if="pg.delete">
                    <n-tag type="error" size="tiny" :bordered="false">已删除</n-tag>
                  </template>
                  <n-tag v-else-if="pg.rotate % 360 !== 0" type="warning" size="tiny" :bordered="false">
                    旋转 {{ pg.rotate }}°
                  </n-tag>
                  <n-tag v-else type="default" size="tiny" :bordered="false">保留</n-tag>
                </div>
                <div class="ed-item-sub">原页 {{ pg.no }}</div>
              </div>
              <div class="ed-item-ops">
                <n-button size="small" quaternary :disabled="i === 0" @click="move(i, -1)">
                  <template #icon><n-icon><ArrowUp /></n-icon></template>
                </n-button>
                <n-button size="small" quaternary :disabled="i === pages.length - 1" @click="move(i, 1)">
                  <template #icon><n-icon><ArrowDown /></n-icon></template>
                </n-button>
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button
                      size="small"
                      :type="pg.rotate % 360 !== 0 ? 'warning' : 'default'"
                      :secondary="pg.rotate % 360 !== 0"
                      @click="rotate(i)"
                    >
                      <template #icon><n-icon><Rotate /></n-icon></template>
                    </n-button>
                  </template>
                  旋转 90°（当前 {{ pg.rotate }}°）
                </n-tooltip>
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button
                      size="small"
                      :type="pg.delete ? 'error' : 'default'"
                      :secondary="pg.delete"
                      @click="toggleDelete(i)"
                    >
                      <template #icon><n-icon><TrashCan /></n-icon></template>
                    </n-button>
                  </template>
                  {{ pg.delete ? "撤销删除" : "删除此页" }}
                </n-tooltip>
              </div>
            </div>
          </div>
        </div>
      </template>

      <div class="tb-submit-row">
        <n-button type="primary" size="large" :loading="loading" class="tb-submit" @click="exportPdf">
          <template #icon><n-icon><Checkmark /></n-icon></template>
          {{ saved ? "已保存 √" : loading ? "正在处理…" : "导出编辑后 PDF" }}
        </n-button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.pn-row {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}
.ed-stats {
  display: flex;
  gap: 10px;
  margin-top: 16px;
}
.ed-stat {
  font-size: 12px;
  color: var(--tb-text-3);
  padding: 4px 10px;
  border: 1px solid var(--tb-border);
  border-radius: 999px;
  background: var(--tb-bg-app);
}
.ed-stat b {
  margin-right: 2px;
  font-variant-numeric: tabular-nums;
}
.ed-stat--warn b {
  color: #d03050;
}
.ed-stat--accent b {
  color: var(--tb-accent);
}
.ed-stat--ok b {
  color: #18a058;
}
.ed-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.ed-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 10px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  background: var(--tb-bg-app);
  transition: border-color 0.15s ease, opacity 0.15s ease;
}
.ed-item--del {
  opacity: 0.55;
  border-color: var(--tb-border);
}
.ed-item--rot {
  border-color: rgba(200, 154, 76, 0.5);
}
.ed-item:hover {
  border-color: var(--tb-border-strong);
}
.ed-item-seq {
  width: 26px;
  height: 26px;
  display: grid;
  place-items: center;
  border-radius: 8px;
  background: var(--tb-primary-weak);
  color: var(--tb-primary);
  font-size: 13px;
  font-weight: 700;
  font-variant-numeric: tabular-nums;
  flex: 0 0 auto;
}
.ed-item--del .ed-item-seq {
  background: rgba(208, 48, 80, 0.12);
  color: #d03050;
}
.ed-item-main {
  flex: 1;
  min-width: 0;
}
.ed-item-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 13px;
  color: var(--tb-text);
}
.ed-item--del .ed-item-title b {
  text-decoration: line-through;
}
.ed-item-sub {
  font-size: 11.5px;
  color: var(--tb-text-3);
  margin-top: 2px;
}
.ed-item-ops {
  display: flex;
  align-items: center;
  gap: 2px;
  flex: 0 0 auto;
}
</style>