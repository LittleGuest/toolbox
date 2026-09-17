<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { Play, Copy, Erase, Information, Code } from "@vicons/carbon";

const message = useMessage();

type CronType = "linux" | "spring" | "quartz";

interface CronResult {
  valid: boolean;
  error?: string | null;
  expression: string;
  type: string;
  description?: string | null;
  nextTimes: string[];
}

// ---------------- 类型 ----------------
const cronType = ref<CronType>("linux");

const TYPE_LIST: { key: CronType; title: string; sub: string }[] = [
  { key: "linux", title: "Linux", sub: "5 字段 · 无秒" },
  { key: "spring", title: "Java Spring", sub: "6 字段 · 秒级" },
  { key: "quartz", title: "Java Quartz", sub: "6-7 字段 · 可含年" },
];

const TYPE_META: Record<CronType, { placeholder: string; hint: string }> = {
  linux: {
    placeholder: "如 */5 * * * *（分 时 日 月 周）",
    hint: "5 个字段；周 0-7（0 和 7 均为周日）；支持 @ 宏",
  },
  spring: {
    placeholder: "如 0 */5 * * * *（秒 分 时 日 月 周）",
    hint: "6 个字段；周 0-7（0 和 7 均为周日）；支持 ? L W # 与 @ 宏",
  },
  quartz: {
    placeholder: "如 0 0 12 ? * MON（秒 分 时 日 月 周 [年]）",
    hint: "6-7 个字段；周 1-7（1 为周日）；支持 ? L W # 与年份字段",
  },
};

// ---------------- 解析区 ----------------
const loading = ref(false);
const result = ref<CronResult | null>(null);
const activeTab = ref("parse");
const expr = ref("*/5 * * * *");

const PRESETS: Record<CronType, { label: string; value: string }[]> = {
  linux: [
    { label: "每分钟", value: "* * * * *" },
    { label: "每 5 分钟", value: "*/5 * * * *" },
    { label: "每小时", value: "0 * * * *" },
    { label: "每天 08:00", value: "0 8 * * *" },
    { label: "每周一 09:00", value: "0 9 * * 1" },
    { label: "工作日 09:00", value: "0 9 * * 1-5" },
    { label: "每月 1 日 00:00", value: "0 0 1 * *" },
    { label: "每年 1 月 1 日", value: "0 0 1 1 *" },
    { label: "@daily", value: "@daily" },
    { label: "@hourly", value: "@hourly" },
  ],
  spring: [
    { label: "每秒", value: "* * * * * *" },
    { label: "每 10 秒", value: "*/10 * * * * *" },
    { label: "每分钟", value: "0 * * * * *" },
    { label: "每 5 分钟", value: "0 */5 * * * *" },
    { label: "每小时整点", value: "0 0 * * * *" },
    { label: "每天 08:00", value: "0 0 8 * * *" },
    { label: "每周一 09:00", value: "0 0 9 * * 1" },
    { label: "工作日 09:00", value: "0 0 9 * * 1-5" },
    { label: "每月 1 日", value: "0 0 0 1 * ?" },
    { label: "每月最后一天", value: "0 0 0 L * ?" },
    { label: "@daily", value: "@daily" },
  ],
  quartz: [
    { label: "每秒", value: "* * * * * *" },
    { label: "每 10 秒", value: "*/10 * * * * *" },
    { label: "每分钟", value: "0 * * * * *" },
    { label: "每 5 分钟", value: "0 */5 * * * *" },
    { label: "每小时整点", value: "0 0 * * * *" },
    { label: "每天 08:00", value: "0 0 8 * * *" },
    { label: "每周一 09:00", value: "0 0 9 ? * 2" },
    { label: "每月 1 日", value: "0 0 0 1 * ?" },
    { label: "每月最后一天", value: "0 0 0 L * ?" },
    { label: "每月第三个周五", value: "0 0 9 ? * 6#3" },
    { label: "每年 2 月 29 日", value: "0 0 0 29 2 ?" },
  ],
};

const parse = async (value?: string) => {
  const expression = (value ?? expr.value).trim();
  if (!expression) {
    message.warning("请输入 Cron 表达式");
    return;
  }
  loading.value = true;
  result.value = null;
  try {
    result.value = await invoke<CronResult>("cron_parse", {
      expression,
      count: 10,
      cronType: cronType.value,
    });
  } catch (error) {
    result.value = { valid: false, error: String(error), nextTimes: [] };
  } finally {
    loading.value = false;
  }
};

const applyPreset = (value: string) => {
  expr.value = value;
  parse(value);
};

// ---------------- 生成区 ----------------
interface FieldGen {
  mode: "any" | "interval" | "specific" | "special";
  interval: number;
  values: number[];
  special: string;
  specialDow: number;
  wDay: number;
  offset: number;
  nth: number;
  text: string;
}

const makeField = (overrides: Partial<FieldGen> = {}): FieldGen => ({
  mode: "any",
  interval: 5,
  values: [],
  special: "L",
  specialDow: 1,
  wDay: 15,
  offset: 3,
  nth: 3,
  text: "",
  ...overrides,
});

const gen = reactive({
  sec: makeField({ interval: 10 }),
  min: makeField({ interval: 5 }),
  hour: makeField({ interval: 2 }),
  dom: makeField(),
  month: makeField({ interval: 2 }),
  dow: makeField(),
  year: makeField(),
});

type FieldKey = keyof typeof gen;

interface FieldCard {
  key: FieldKey;
  label: string;
  kind: "num" | "dom" | "dow" | "year";
  unit?: string;
  max?: number;
}

const rangeOptions = (min: number, max: number) =>
  Array.from({ length: max - min + 1 }, (_, i) => ({ label: String(min + i), value: min + i }));

const MONTH_LABELS = ["一月", "二月", "三月", "四月", "五月", "六月", "七月", "八月", "九月", "十月", "十一月", "十二月"];
const monthOptions = rangeOptions(1, 12).map((o) => ({ ...o, label: `${MONTH_LABELS[o.value - 1]}(${o.value})` }));

const DOW_NAMES = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
const dowOptions = computed(() => {
  if (cronType.value === "quartz") {
    return DOW_NAMES.map((n, i) => ({ label: `${n}(${i + 1})`, value: i + 1 }));
  }
  return [
    ...DOW_NAMES.map((n, i) => ({ label: `${n}(${i})`, value: i })),
    { label: "周日(7)", value: 7 },
  ];
});

const MODE_OPTIONS = [
  { label: "任意 (*)", value: "any" },
  { label: "间隔 (*/N)", value: "interval" },
  { label: "指定", value: "specific" },
];
const domModeOptions = computed(() =>
  cronType.value === "linux"
    ? MODE_OPTIONS
    : [...MODE_OPTIONS, { label: "特殊 (L/W)", value: "special" }]
);
const dowModeOptions = computed(() =>
  cronType.value === "linux"
    ? [MODE_OPTIONS[0], MODE_OPTIONS[2]]
    : [MODE_OPTIONS[0], MODE_OPTIONS[2], { label: "特殊 (L/#)", value: "special" }]
);
const YEAR_MODE_OPTIONS = [MODE_OPTIONS[0], { label: "指定年份", value: "specific" }];

const DOM_SPECIAL_OPTIONS = [
  { label: "L · 每月最后一天", value: "L" },
  { label: "LW · 每月最后一个工作日", value: "LW" },
  { label: "NW · 离 N 号最近的工作日", value: "W" },
  { label: "L-N · 每月倒数第 N 天", value: "L-n" },
];
const DOW_SPECIAL_OPTIONS = [
  { label: "NL · 每月最后一个周 N", value: "L" },
  { label: "N#M · 每月第 M 个周 N", value: "#" },
];

// 按 Cron 字段顺序生成配置卡片（秒/分/时/日/月/周/[年]）
const FIELD_CARDS = computed<FieldCard[]>(() => {
  const cards: FieldCard[] = [];
  if (cronType.value !== "linux") {
    cards.push({ key: "sec", label: "秒", kind: "num", unit: "秒", max: 59 });
  }
  cards.push({ key: "min", label: "分", kind: "num", unit: "分钟", max: 59 });
  cards.push({ key: "hour", label: "时", kind: "num", unit: "小时", max: 23 });
  cards.push({ key: "dom", label: "日", kind: "dom" });
  cards.push({ key: "month", label: "月", kind: "num", unit: "个月", max: 12 });
  cards.push({ key: "dow", label: "周", kind: "dow" });
  if (cronType.value === "quartz") {
    cards.push({ key: "year", label: "年", kind: "year" });
  }
  return cards;
});

const numOpts = (c: FieldCard) =>
  c.key === "month" ? monthOptions : rangeOptions(0, c.max ?? 59);

const modeOptionsFor = (c: FieldCard) => {
  if (c.kind === "dom") return domModeOptions.value;
  if (c.kind === "dow") return dowModeOptions.value;
  return MODE_OPTIONS;
};

const composeNumField = (f: FieldGen) => {
  if (f.mode === "any") return "*";
  if (f.mode === "interval") return `*/${f.interval}`;
  if (f.mode === "specific") {
    return f.values.length ? [...f.values].sort((a, b) => a - b).join(",") : "*";
  }
  return "*";
};

const composeDom = () => {
  const f = gen.dom;
  if (f.mode === "special") {
    if (f.special === "L") return "L";
    if (f.special === "LW") return "LW";
    if (f.special === "W") return `${f.wDay}W`;
    if (f.special === "L-n") return `L-${f.offset}`;
  }
  return composeNumField(f);
};

const composeDow = () => {
  const f = gen.dow;
  if (f.mode === "special") {
    if (f.special === "L") return `${f.specialDow}L`;
    if (f.special === "#") return `${f.specialDow}#${f.nth}`;
  }
  return composeNumField(f);
};

const generatedExpr = computed(() => {
  const parts: string[] = [];
  if (cronType.value !== "linux") parts.push(composeNumField(gen.sec));
  parts.push(composeNumField(gen.min));
  parts.push(composeNumField(gen.hour));
  parts.push(composeDom());
  parts.push(composeNumField(gen.month));
  parts.push(composeDow());
  if (cronType.value === "quartz" && gen.year.mode === "specific" && gen.year.text.trim()) {
    parts.push(gen.year.text.trim());
  }
  return parts.join(" ");
});

// 切换类型时重置示例并清理不兼容的选项
watch(cronType, (t) => {
  expr.value = t === "linux" ? "*/5 * * * *" : t === "spring" ? "0 */5 * * * *" : "0 */5 * * * * ?";
  if (t === "linux") {
    if (gen.dom.mode === "special") gen.dom.mode = "any";
    if (gen.dow.mode === "special") gen.dow.mode = "any";
  }
  gen.dow.values = gen.dow.values.filter((v) => v >= (t === "quartz" ? 1 : 0) && v <= 7);
});

const copy = (value: string) => {
  if (!value) return;
  writeText(value);
  message.success("复制成功");
};

const clear = () => {
  result.value = null;
};
</script>

<template>
  <div class="tb-page">
    <!-- 方言选择：卡片式分段 -->
    <div class="kind-switch" role="radiogroup" aria-label="Cron 方言">
      <button
        v-for="t in TYPE_LIST"
        :key="t.key"
        role="radio"
        :aria-checked="cronType === t.key"
        class="kind-card"
        :class="{ active: cronType === t.key }"
        @click="cronType = t.key"
      >
        <span class="kind-dot" aria-hidden="true"></span>
        <span class="kind-title">{{ t.title }}</span>
        <span class="kind-sub">{{ t.sub }}</span>
      </button>
    </div>
    <p class="kind-hint">
      <n-icon :size="15" class="kind-hint-icon"><Information /></n-icon>
      <span>{{ TYPE_META[cronType].hint }}</span>
    </p>

    <div class="tb-card cron-body">
      <n-tabs v-model:value="activeTab" type="segment" animated class="cron-tabs">
        <!-- 解析 -->
        <n-tab-pane name="parse" tab="解析">
          <div class="parse-row">
            <n-input
              v-model:value="expr"
              class="expr-input"
              :placeholder="TYPE_META[cronType].placeholder"
              clearable
              @keyup.enter="parse()"
            />
            <n-button type="primary" :loading="loading" class="parse-btn" @click="parse()">
              <template #icon><n-icon><Play /></n-icon></template>
              解析
            </n-button>
          </div>

          <h3 class="block-label">常用预设</h3>
          <div class="preset-grid">
            <button
              v-for="p in PRESETS[cronType]"
              :key="p.value"
              type="button"
              class="preset-chip"
              :class="{ active: expr === p.value }"
              @click="applyPreset(p.value)"
            >
              <span class="preset-name">{{ p.label }}</span>
              <code class="preset-expr">{{ p.value }}</code>
            </button>
          </div>
        </n-tab-pane>

        <!-- 生成 -->
        <n-tab-pane name="generate" tab="生成">
          <div class="gen-grid">
            <div v-for="c in FIELD_CARDS" :key="c.key" class="field-card">
              <div class="field-head">
                <span class="field-name">{{ c.label }}</span>
                <n-select
                  v-model:value="gen[c.key].mode"
                  size="small"
                  :options="c.kind === 'year' ? YEAR_MODE_OPTIONS : modeOptionsFor(c)"
                  class="mode-select"
                />
              </div>

              <div class="field-body">
                <!-- 数值字段（秒/分/时/月） -->
                <template v-if="c.kind === 'num'">
                  <template v-if="gen[c.key].mode === 'interval'">
                    <n-input-number v-model:value="gen[c.key].interval" :min="1" :max="c.max ?? 59" size="small" class="num-input" />
                    <n-text depth="3" class="field-hint">每 {{ gen[c.key].interval }} {{ c.unit }}</n-text>
                  </template>
                  <n-select
                    v-else-if="gen[c.key].mode === 'specific'"
                    v-model:value="gen[c.key].values"
                    multiple
                    filterable
                    :options="numOpts(c)"
                    size="small"
                    class="body-select"
                    placeholder="选择或搜索"
                  />
                  <span v-else class="any-symbol">*</span>
                </template>

                <!-- 日字段 -->
                <template v-else-if="c.kind === 'dom'">
                  <template v-if="gen.dom.mode === 'interval'">
                    <n-input-number v-model:value="gen.dom.interval" :min="1" :max="31" size="small" class="num-input" />
                    <n-text depth="3" class="field-hint">每 {{ gen.dom.interval }} 天</n-text>
                  </template>
                  <n-select
                    v-else-if="gen.dom.mode === 'specific'"
                    v-model:value="gen.dom.values"
                    multiple
                    filterable
                    :options="rangeOptions(1, 31)"
                    size="small"
                    class="body-select"
                    placeholder="选择 1-31"
                  />
                  <template v-else-if="gen.dom.mode === 'special'">
                    <n-select v-model:value="gen.dom.special" :options="DOM_SPECIAL_OPTIONS" size="small" class="body-select" />
                    <n-input-number
                      v-if="gen.dom.special === 'W'"
                      v-model:value="gen.dom.wDay"
                      :min="1"
                      :max="31"
                      size="small"
                      class="num-input"
                    />
                    <n-input-number
                      v-else-if="gen.dom.special === 'L-n'"
                      v-model:value="gen.dom.offset"
                      :min="1"
                      :max="30"
                      size="small"
                      class="num-input"
                    />
                  </template>
                  <span v-else class="any-symbol">*</span>
                </template>

                <!-- 周字段 -->
                <template v-else-if="c.kind === 'dow'">
                  <n-select
                    v-if="gen.dow.mode === 'specific'"
                    v-model:value="gen.dow.values"
                    multiple
                    filterable
                    :options="dowOptions"
                    size="small"
                    class="body-select"
                    placeholder="选择星期"
                  />
                  <template v-else-if="gen.dow.mode === 'special'">
                    <n-select v-model:value="gen.dow.special" :options="DOW_SPECIAL_OPTIONS" size="small" class="body-select" />
                    <n-select v-model:value="gen.dow.specialDow" :options="dowOptions" size="small" style="width: 120px" />
                    <n-input-number
                      v-if="gen.dow.special === '#'"
                      v-model:value="gen.dow.nth"
                      :min="1"
                      :max="5"
                      size="small"
                      class="num-input"
                    />
                  </template>
                  <span v-else class="any-symbol">*</span>
                </template>

                <!-- 年字段（Quartz） -->
                <template v-else>
                  <n-input
                    v-if="gen.year.mode === 'specific'"
                    v-model:value="gen.year.text"
                    size="small"
                    class="body-select"
                    placeholder="如 2024-2026 / 2024,2026"
                  />
                  <span v-else class="any-symbol">*</span>
                </template>
              </div>
            </div>
          </div>

          <!-- 表达式预览 -->
          <div class="expr-bar">
            <div class="expr-info">
              <n-icon :size="16" class="expr-ico"><Code /></n-icon>
              <code class="expr-code">{{ generatedExpr }}</code>
            </div>
            <n-space :size="8">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" :loading="loading" @click="parse(generatedExpr)">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                解析预览
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button :disabled="!generatedExpr" @click="copy(generatedExpr)">
                    <template #icon>
                      <n-icon><Copy /></n-icon>
                    </template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
            </n-space>
          </div>
        </n-tab-pane>
      </n-tabs>

      <!-- 结果 -->
      <div v-if="result" class="result-panel">
        <n-alert
          v-if="!result.valid"
          type="error"
          title="表达式无效"
          :description="result.error"
          closable
          @close="clear"
        />
        <div v-else class="result-body">
          <div class="result-banner">
            <div class="result-main">
              <n-tag :bordered="false" type="success" size="small">
                {{ result.type === "linux" ? "Linux" : result.type === "spring" ? "Java Spring" : "Java Quartz" }}
              </n-tag>
              <h3 class="result-expr">{{ result.expression }}</h3>
              <p class="result-desc">{{ result.description }}</p>
            </div>
            <n-space :size="8">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="copy(result.expression)">
                    <template #icon>
                      <n-icon><Copy /></n-icon>
                    </template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="clear">
                    <template #icon>
                      <n-icon><Erase /></n-icon>
                    </template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </n-space>
          </div>

          <h3 class="block-label">接下来 10 次执行时间</h3>
          <div class="time-grid">
            <div v-for="(t, i) in result.nextTimes" :key="i" class="time-chip" :class="{ first: i === 0 }">
              <span v-if="i === 0" class="time-badge">下次</span>
              <code>{{ t }}</code>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* ---------- 方言切换（卡片式分段） ---------- */
.kind-switch {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
}

.kind-card {
  position: relative;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 3px;
  padding: 16px 18px 16px 22px;
  background: var(--tb-bg-elevated);
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  box-shadow: var(--tb-shadow-card);
  cursor: pointer;
  text-align: left;
  font-family: inherit;
  transition: border-color 0.2s ease, transform 0.15s ease, box-shadow 0.2s ease;
}

.kind-card::before {
  content: "";
  position: absolute;
  left: 0;
  top: 12px;
  bottom: 12px;
  width: 3px;
  border-radius: 0 3px 3px 0;
  background: transparent;
  transition: background 0.2s ease;
}

.kind-card:hover {
  border-color: var(--tb-border-strong);
  transform: translateY(-1px);
}

.kind-card.active {
  border-color: var(--tb-primary);
  box-shadow: 0 4px 18px var(--tb-primary-weak);
}

.kind-card.active::before {
  background: var(--tb-primary);
}

.kind-title {
  font-size: 15px;
  font-weight: 650;
  color: var(--tb-text);
}

.kind-sub {
  font-size: 12px;
  color: var(--tb-text-3);
}

.kind-hint {
  display: flex;
  align-items: center;
  gap: 6px;
  margin: 4px 2px 0;
  font-size: 12px;
  color: var(--tb-text-3);
}

.kind-hint-icon {
  color: var(--tb-primary);
  opacity: 0.8;
}

/* ---------- 主卡 ---------- */
.cron-body {
  padding: 22px;
}

.cron-tabs {
  margin-top: -6px;
}

.block-label {
  margin: 20px 0 12px;
  font-size: 13px;
  font-weight: 600;
  color: var(--tb-text-2);
}

/* ---------- 解析 ---------- */
.parse-row {
  display: flex;
  gap: 10px;
}

.expr-input {
  flex: 1;
}

.expr-input :deep(.n-input__input-el) {
  font-family: var(--tb-font-mono);
}

.parse-btn {
  padding: 0 20px;
}

.preset-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(210px, 1fr));
  gap: 8px;
}

.preset-chip {
  display: flex;
  flex-direction: column;
  gap: 4px;
  text-align: left;
  padding: 10px 12px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-s);
  background: transparent;
  color: var(--tb-text);
  cursor: pointer;
  font-family: inherit;
  transition: border-color 0.2s, background-color 0.2s;
}

.preset-chip:hover {
  border-color: var(--tb-primary);
  background: var(--tb-primary-weak);
}

.preset-chip.active {
  border-color: var(--tb-primary);
  background: var(--tb-primary-weak);
}

.preset-name {
  font-size: 13px;
  font-weight: 600;
}

.preset-expr {
  font-size: 12px;
  font-family: var(--tb-font-mono);
  color: var(--tb-text-3);
}

/* ---------- 生成 ---------- */
.gen-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 12px;
}

.field-card {
  padding: 12px 14px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-s);
  background: var(--tb-bg-app);
  transition: border-color 0.2s ease;
}

.field-card:focus-within {
  border-color: var(--tb-primary);
}

.field-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 10px;
}

.field-name {
  font-size: 13px;
  font-weight: 600;
  color: var(--tb-text);
}

.mode-select {
  width: 110px;
}

.field-body {
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
  min-height: 30px;
}

.body-select {
  min-width: 160px;
  flex: 1;
}

.num-input {
  width: 90px;
}

.field-hint {
  font-size: 12px;
}

.any-symbol {
  padding: 2px 12px;
  border: 1px dashed var(--tb-border-strong);
  border-radius: var(--tb-radius-s);
  font-family: var(--tb-font-mono);
  font-size: 16px;
  color: var(--tb-text-3);
}

/* ---------- 表达式预览 ---------- */
.expr-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
  margin-top: 16px;
  padding: 12px 14px;
  border: 1px solid var(--tb-border-strong);
  border-radius: var(--tb-radius-s);
  background: var(--tb-bg-app);
}

.expr-info {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
  min-width: 0;
}

.expr-ico {
  color: var(--tb-primary);
  flex-shrink: 0;
}

.expr-code {
  font-family: var(--tb-font-mono);
  font-size: 14px;
  font-weight: 600;
  color: var(--tb-primary);
  word-break: break-all;
}

/* ---------- 结果 ---------- */
.result-panel {
  margin-top: 16px;
}

.result-banner {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  flex-wrap: wrap;
  padding: 16px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  background: var(--tb-bg-app);
}

.result-expr {
  margin: 8px 0 4px;
  font-family: var(--tb-font-mono);
  font-size: 16px;
  color: var(--tb-text);
  word-break: break-all;
}

.result-desc {
  margin: 0;
  font-size: 13px;
  color: var(--tb-text-2);
}

.time-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 8px;
}

.time-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-s);
  font-family: var(--tb-font-mono);
  font-size: 13px;
  color: var(--tb-text);
}

.time-chip.first {
  border-color: var(--tb-primary);
  color: var(--tb-primary);
  font-weight: 600;
}

.time-badge {
  padding: 1px 8px;
  border-radius: 999px;
  background: var(--tb-primary);
  color: #fff;
  font-size: 11px;
  font-weight: 600;
  flex-shrink: 0;
}

@media (max-width: 760px) {
  .kind-switch {
    grid-template-columns: 1fr;
  }
}
</style>
