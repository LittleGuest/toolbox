<script setup lang="ts">
import { ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Compare, ArrowsHorizontal, Copy, Close } from "@vicons/carbon";

const message = useMessage();
const mode = ref<"text" | "json">("text");

const leftText = ref("");
const rightText = ref("");

// ---------- 文本差异（按行 LCS） ----------
interface LineCell {
  text: string;
  type: "same" | "delete" | "add" | "blank";
  no: number;
}

interface TextResult {
  left: LineCell[];
  right: LineCell[];
  add: number;
  del: number;
  same: number;
}

type DiffOp = { type: "same" | "delete" | "add"; text: string };

// 按行 LCS（O(n×m)，dp 用 Uint32Array 展平存储，限制 n*m <= 4,000,000）
function lcsOps(a: string[], b: string[]): DiffOp[] {
  const n = a.length;
  const m = b.length;
  const w = m + 1;
  const dp = new Uint32Array((n + 1) * w);
  for (let i = 1; i <= n; i++) {
    const ai = a[i - 1];
    for (let j = 1; j <= m; j++) {
      const cur = i * w + j;
      if (ai === b[j - 1]) dp[cur] = dp[(i - 1) * w + (j - 1)] + 1;
      else dp[cur] = Math.max(dp[(i - 1) * w + j], dp[i * w + (j - 1)]);
    }
  }
  const ops: DiffOp[] = [];
  let i = n;
  let j = m;
  while (i > 0 && j > 0) {
    if (a[i - 1] === b[j - 1]) {
      ops.push({ type: "same", text: a[i - 1] });
      i--;
      j--;
    } else if (dp[(i - 1) * w + j] >= dp[i * w + (j - 1)]) {
      ops.push({ type: "delete", text: a[i - 1] });
      i--;
    } else {
      ops.push({ type: "add", text: b[j - 1] });
      j--;
    }
  }
  while (i > 0) {
    ops.push({ type: "delete", text: a[i - 1] });
    i--;
  }
  while (j > 0) {
    ops.push({ type: "add", text: b[j - 1] });
    j--;
  }
  return ops.reverse();
}

const textResult = ref<TextResult | null>(null);

const runTextDiff = () => {
  const a = leftText.value.split("\n");
  const b = rightText.value.split("\n");
  if (a.length * b.length > 4_000_000) {
    message.warning("内容过大");
    textResult.value = null;
    return;
  }
  const ops = lcsOps(a, b);
  const left: LineCell[] = [];
  const right: LineCell[] = [];
  let oi = 0;
  let ni = 0;
  let add = 0;
  let del = 0;
  let same = 0;
  for (const op of ops) {
    if (op.type === "same") {
      left.push({ text: op.text, type: "same", no: ++oi });
      right.push({ text: op.text, type: "same", no: ++ni });
      same++;
    } else if (op.type === "delete") {
      left.push({ text: op.text, type: "delete", no: ++oi });
      right.push({ text: "", type: "blank", no: 0 });
      del++;
    } else {
      left.push({ text: "", type: "blank", no: 0 });
      right.push({ text: op.text, type: "add", no: ++ni });
      add++;
    }
  }
  textResult.value = { left, right, add, del, same };
};

// ---------- JSON 差异（按结构递归） ----------
type JsonDiffType = "add" | "delete" | "modify";

interface JsonDiffItem {
  path: string;
  type: JsonDiffType;
  oldText: string;
  newText: string;
}

const jsonCompared = ref(false);
const jsonItems = ref<JsonDiffItem[]>([]);
const jsonCounts = ref({ add: 0, del: 0, modify: 0 });

const fmt = (v: unknown): string => {
  if (v === undefined) return "undefined";
  if (typeof v === "function") return "function";
  if (typeof v === "symbol") return String(v);
  const s = JSON.stringify(v);
  return s === undefined ? String(v) : s;
};

const truncate = (s: string) => (s.length > 80 ? s.slice(0, 80) + "…" : s);

// 递归对比：对象按键（新增/删除/共有递归）、数组按索引（长度差为尾部新增/删除）、基本类型按值
const diffValue = (a: unknown, b: unknown, path: string, out: JsonDiffItem[]) => {
  const aObj = a !== null && typeof a === "object";
  const bObj = b !== null && typeof b === "object";
  if (aObj && bObj) {
    if (Array.isArray(a) && Array.isArray(b)) {
      const len = Math.min(a.length, b.length);
      for (let i = 0; i < len; i++) diffValue(a[i], b[i], `${path}[${i}]`, out);
      for (let i = len; i < a.length; i++) {
        out.push({ path: `${path}[${i}]`, type: "delete", oldText: fmt(a[i]), newText: "" });
      }
      for (let i = len; i < b.length; i++) {
        out.push({ path: `${path}[${i}]`, type: "add", oldText: "", newText: fmt(b[i]) });
      }
    } else if (Array.isArray(a) !== Array.isArray(b)) {
      out.push({ path: path || "(root)", type: "modify", oldText: fmt(a), newText: fmt(b) });
    } else {
      const ra = a as Record<string, unknown>;
      const rb = b as Record<string, unknown>;
      for (const k of Object.keys(ra)) {
        const p = path ? `${path}.${k}` : k;
        if (!(k in rb)) out.push({ path: p, type: "delete", oldText: fmt(ra[k]), newText: "" });
        else diffValue(ra[k], rb[k], p, out);
      }
      for (const k of Object.keys(rb)) {
        if (!(k in ra)) {
          const p = path ? `${path}.${k}` : k;
          out.push({ path: p, type: "add", oldText: "", newText: fmt(rb[k]) });
        }
      }
    }
  } else if (a === b) {
    return;
  } else {
    out.push({ path: path || "(root)", type: "modify", oldText: fmt(a), newText: fmt(b) });
  }
};

const runJsonDiff = () => {
  let a: unknown;
  let b: unknown;
  try {
    a = JSON.parse(leftText.value);
  } catch (e) {
    message.error(`JSON 解析失败：${(e as Error).message}`);
    return;
  }
  try {
    b = JSON.parse(rightText.value);
  } catch (e) {
    message.error(`JSON 解析失败：${(e as Error).message}`);
    return;
  }
  const items: JsonDiffItem[] = [];
  diffValue(a, b, "", items);
  for (const it of items) {
    it.oldText = truncate(it.oldText);
    it.newText = truncate(it.newText);
  }
  jsonItems.value = items;
  jsonCounts.value = {
    add: items.filter((i) => i.type === "add").length,
    del: items.filter((i) => i.type === "delete").length,
    modify: items.filter((i) => i.type === "modify").length,
  };
  jsonCompared.value = true;
};

const runDiff = () => {
  if (mode.value === "text") runTextDiff();
  else runJsonDiff();
};

const placeholderLeft = mode.value === "text" ? "请输入旧文本" : '{"name": "x"}';
const placeholderRight = mode.value === "text" ? "请输入新文本" : '{"name": "y"}';
const copyLeftLabel = mode.value === "text" ? "复制左侧" : "复制 A";
const copyRightLabel = mode.value === "text" ? "复制右侧" : "复制 B";

const swap = () => {
  const t = leftText.value;
  leftText.value = rightText.value;
  rightText.value = t;
  if (mode.value === "text") {
    if (textResult.value) runTextDiff();
  } else {
    if (jsonCompared.value) runJsonDiff();
  }
};

const copy = (value: string) => {
  if (!value) return;
  writeText(value);
  message.success("复制成功");
};

const clear = () => {
  leftText.value = "";
  rightText.value = "";
  textResult.value = null;
  jsonCompared.value = false;
  jsonItems.value = [];
  jsonCounts.value = { add: 0, del: 0, modify: 0 };
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
    <div class="input-row">
      <div class="input-col">
        <span class="tb-editor-label">原始</span>
        <n-input
          v-model:value="leftText"
          type="textarea"
          :rows="10"
          :placeholder="placeholderLeft"
          class="mono-input"
        />
      </div>
      <n-space :size="8" class="action-col" direction="vertical">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="runDiff">
              <template #icon>
                <n-icon>
                  <Compare />
                </n-icon>
              </template>
            </n-button>
          </template>
          对比
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="swap">
              <template #icon>
                <n-icon>
                  <ArrowsHorizontal />
                </n-icon>
              </template>
            </n-button>
          </template>
          交换
        </n-tooltip>
      </n-space>
      <div class="input-col">
        <span class="tb-editor-label">对比</span>
        <n-input
          v-model:value="rightText"
          type="textarea"
          :rows="10"
          :placeholder="placeholderRight"
          class="mono-input"
        />
      </div>
    </div>

    <div class="mode-bar">
      <n-radio-group v-model:value="mode" size="small">
        <n-radio-button value="text">文本差异</n-radio-button>
        <n-radio-button value="json">JSON 差异</n-radio-button>
      </n-radio-group>
    </div>

    <!-- 文本差异结果 -->
    <template v-if="mode === 'text' && textResult">
      <n-space :size="8" class="stat-bar">
        <n-tag :bordered="false" type="success" size="small">新增 {{ textResult.add }} 行</n-tag>
        <n-tag :bordered="false" type="error" size="small">删除 {{ textResult.del }} 行</n-tag>
        <n-tag :bordered="false" size="small">相同 {{ textResult.same }} 行</n-tag>
      </n-space>

      <div class="diff-row">
        <div class="diff-pane">
          <div
            v-for="(line, i) in textResult.left"
            :key="i"
            class="diff-line"
            :class="line.type"
          >
            <span class="line-no">{{ line.no || "" }}</span>
            <span class="prefix">{{ line.type === "delete" ? "-" : "" }}</span>
            <span class="line-text">{{ line.text || "" }}</span>
          </div>
        </div>
        <div class="diff-pane">
          <div
            v-for="(line, i) in textResult.right"
            :key="i"
            class="diff-line"
            :class="line.type"
          >
            <span class="line-no">{{ line.no || "" }}</span>
            <span class="prefix">{{ line.type === "add" ? "+" : "" }}</span>
            <span class="line-text">{{ line.text || "" }}</span>
          </div>
        </div>
      </div>
    </template>

    <!-- JSON 差异结果 -->
    <template v-if="mode === 'json' && jsonCompared">
      <n-alert
        v-if="jsonItems.length === 0"
        type="success"
        title="两个 JSON 完全相同"
        class="result-box"
      />
      <template v-else>
        <n-space :size="8" class="stat-bar">
          <n-tag :bordered="false" type="success" size="small">新增 {{ jsonCounts.add }}</n-tag>
          <n-tag :bordered="false" type="error" size="small">删除 {{ jsonCounts.del }}</n-tag>
          <n-tag :bordered="false" type="warning" size="small">修改 {{ jsonCounts.modify }}</n-tag>
        </n-space>

        <n-list bordered class="diff-list">
          <n-list-item v-for="(item, i) in jsonItems" :key="i">
            <div class="diff-item">
              <div class="item-head">
                <span class="item-path">{{ item.path }}</span>
                <n-tag
                  :bordered="false"
                  size="small"
                  :type="item.type === 'add' ? 'success' : item.type === 'delete' ? 'error' : 'warning'"
                >
                  {{ item.type === "add" ? "新增" : item.type === "delete" ? "删除" : "修改" }}
                </n-tag>
              </div>
              <div class="item-values">
                <span v-if="item.oldText" class="value old">{{ item.oldText }}</span>
                <span v-if="item.oldText && item.newText" class="arrow">→</span>
                <span v-if="item.newText" class="value new">{{ item.newText }}</span>
              </div>
            </div>
          </n-list-item>
        </n-list>
      </template>
    </template>

    <n-space :size="8" class="bottom-bar">
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button @click="copy(leftText)">
            <template #icon>
              <n-icon>
                <Copy />
              </n-icon>
            </template>
          </n-button>
        </template>
        {{ copyLeftLabel }}
      </n-tooltip>
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button @click="copy(rightText)">
            <template #icon>
              <n-icon>
                <Copy />
              </n-icon>
            </template>
          </n-button>
        </template>
        {{ copyRightLabel }}
      </n-tooltip>
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button @click="clear">
            <template #icon>
              <n-icon>
                <Close />
              </n-icon>
            </template>
          </n-button>
        </template>
        清除
      </n-tooltip>
    </n-space>
    </section>
  </div>
</template>

<style scoped>
.mode-bar {
  margin-top: 12px;
}

.input-row {
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.input-col {
  flex: 1;
  min-width: 0;
}

.action-col {
  padding-top: 60px;
}

.mono-input :deep(.n-input__input-el) {
  font-family: var(--n-font-family-mono, monospace);
}

.result-box {
  margin-top: 14px;
}

.stat-bar {
  margin-top: 14px;
}

.diff-row {
  display: flex;
  gap: 12px;
  margin-top: 10px;
}

.diff-pane {
  flex: 1;
  min-width: 0;
  max-height: 420px;
  overflow: auto;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  background: var(--tb-bg-app);
  font-family: var(--tb-font-mono);
  font-size: 12px;
  line-height: 1.7;
}

.diff-line {
  display: flex;
  padding: 0 6px;
}

.diff-line.delete {
  background: rgba(208, 60, 60, 0.15);
  color: #b0302a;
}

.diff-line.add {
  background: rgba(24, 160, 88, 0.13);
  color: #15803d;
}

.diff-line.same {
  color: var(--n-text-color-1, #333);
}

.line-no {
  flex-shrink: 0;
  width: 38px;
  text-align: right;
  margin-right: 8px;
  color: var(--n-text-color-3, #999);
  user-select: none;
}

.prefix {
  flex-shrink: 0;
  width: 12px;
  font-weight: 700;
}

.line-text {
  white-space: pre;
  word-break: break-all;
}

.diff-list {
  margin-top: 10px;
  max-height: 460px;
  overflow: auto;
}

.diff-item {
  display: flex;
  flex-direction: column;
  gap: 6px;
  width: 100%;
}

.item-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.item-path {
  font-family: var(--n-font-family-mono, monospace);
  font-size: 13px;
  font-weight: 600;
  color: var(--n-text-color-1, #333);
  word-break: break-all;
}

.item-values {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  flex-wrap: wrap;
  font-family: var(--n-font-family-mono, monospace);
  font-size: 12px;
  line-height: 1.6;
  word-break: break-all;
}

.value.old {
  color: #b0302a;
}

.value.new {
  color: #15803d;
}

.arrow {
  color: var(--n-text-color-3, #999);
  flex-shrink: 0;
}

.bottom-bar {
  margin-top: 12px;
}
</style>