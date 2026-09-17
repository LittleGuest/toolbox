<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Shuffle, Copy } from "@vicons/carbon";

const message = useMessage();

type Source = "hex" | "rgb" | "hsv" | "cmyk";

const hex = ref("#3498db");
const rgb = ref("52, 152, 219");
const hsv = ref("210, 76, 86");
const cmyk = ref("65, 33, 0, 14");

// 联动抑制标记：回填其它格式时不触发各自的 watch
const suppress = ref(false);
const invalidField = ref<Source | null>(null);

interface RGB {
  r: number;
  g: number;
  b: number;
}
interface HSV {
  h: number;
  s: number;
  v: number;
}
interface CMYK {
  c: number;
  m: number;
  y: number;
  k: number;
}

const clamp = (v: number, min: number, max: number) => Math.min(max, Math.max(min, v));
const round1 = (v: number) => Math.round(v * 10) / 10;

// ---------------- 转换函数 ----------------
function hexToRgb(input: string): RGB | null {
  const m = /^#?([0-9a-fA-F]{3}|[0-9a-fA-F]{6})$/.exec(input.trim());
  if (!m) return null;
  let h = m[1];
  if (h.length === 3) h = h.split("").map((c) => c + c).join("");
  return {
    r: parseInt(h.slice(0, 2), 16),
    g: parseInt(h.slice(2, 4), 16),
    b: parseInt(h.slice(4, 6), 16),
  };
}

function rgbToHex({ r, g, b }: RGB): string {
  return "#" + ((r << 16) | (g << 8) | b).toString(16).padStart(6, "0");
}

function rgbToHsv({ r, g, b }: RGB): HSV {
  const rr = r / 255;
  const gg = g / 255;
  const bb = b / 255;
  const max = Math.max(rr, gg, bb);
  const min = Math.min(rr, gg, bb);
  const d = max - min;
  let h = 0;
  if (d !== 0) {
    if (max === rr) h = 60 * (((gg - bb) / d) % 6);
    else if (max === gg) h = 60 * ((bb - rr) / d + 2);
    else h = 60 * ((rr - gg) / d + 4);
  }
  if (h < 0) h += 360;
  const s = max === 0 ? 0 : (d / max) * 100;
  return { h, s, v: max * 100 };
}

function hsvToRgb({ h, s, v }: HSV): RGB {
  const hh = ((h % 360) + 360) % 360;
  const ss = clamp(s, 0, 100) / 100;
  const vv = clamp(v, 0, 100) / 100;
  const c = vv * ss;
  const x = c * (1 - Math.abs(((hh / 60) % 2) - 1));
  const m = vv - c;
  let r = 0;
  let g = 0;
  let b = 0;
  if (hh < 60) {
    r = c;
    g = x;
  } else if (hh < 120) {
    r = x;
    g = c;
  } else if (hh < 180) {
    g = c;
    b = x;
  } else if (hh < 240) {
    r = x;
    b = c;
  } else if (hh < 300) {
    r = c;
    b = x;
  } else {
    g = x;
    b = c;
  }
  return {
    r: Math.round((r + m) * 255),
    g: Math.round((g + m) * 255),
    b: Math.round((b + m) * 255),
  };
}

function rgbToCmyk({ r, g, b }: RGB): CMYK {
  const rr = r / 255;
  const gg = g / 255;
  const bb = b / 255;
  const k = 1 - Math.max(rr, gg, bb);
  if (k >= 1) return { c: 0, m: 0, y: 0, k: 100 };
  return {
    c: ((1 - rr - k) / (1 - k)) * 100,
    m: ((1 - gg - k) / (1 - k)) * 100,
    y: ((1 - bb - k) / (1 - k)) * 100,
    k: k * 100,
  };
}

function cmykToRgb({ c, m, y, k }: CMYK): RGB {
  const cc = clamp(c, 0, 100) / 100;
  const mm = clamp(m, 0, 100) / 100;
  const yy = clamp(y, 0, 100) / 100;
  const kk = clamp(k, 0, 100) / 100;
  return {
    r: Math.round(255 * (1 - cc) * (1 - kk)),
    g: Math.round(255 * (1 - mm) * (1 - kk)),
    b: Math.round(255 * (1 - yy) * (1 - kk)),
  };
}

// ---------------- 输入解析与校验 ----------------
const parseList = (input: string, count: number): number[] | null => {
  const parts = input.split(",").map((p) => p.trim()).filter((p) => p !== "");
  if (parts.length !== count) return null;
  const nums = parts.map(Number);
  if (nums.some((n) => Number.isNaN(n))) return null;
  return nums;
};

const parseRgb = (input: string): RGB | null => {
  const nums = parseList(input, 3);
  if (!nums) return null;
  const [r, g, b] = nums;
  if (![r, g, b].every((n) => n >= 0 && n <= 255)) return null;
  return { r: Math.round(r), g: Math.round(g), b: Math.round(b) };
};

const parseHsv = (input: string): HSV | null => {
  const nums = parseList(input, 3);
  if (!nums) return null;
  const [h, s, v] = nums;
  if (h < 0 || h > 360 || s < 0 || s > 100 || v < 0 || v > 100) return null;
  return { h, s, v };
};

const parseCmyk = (input: string): CMYK | null => {
  const nums = parseList(input, 4);
  if (!nums) return null;
  const [c, m, y, k] = nums;
  if (![c, m, y, k].every((n) => n >= 0 && n <= 100)) return null;
  return { c, m, y, k };
};

// ---------------- 联动 ----------------
const currentRgb = ref<RGB | null>(hexToRgb(hex.value));

const currentHex = computed(() => (currentRgb.value ? rgbToHex(currentRgb.value) : "#ffffff"));

const previewTextColor = computed(() => {
  const { r, g, b } = currentRgb.value ?? { r: 255, g: 255, b: 255 };
  const y = 0.299 * r + 0.587 * g + 0.114 * b;
  return y > 128 ? "#111111" : "#ffffff";
});

const colorInput = ref("#3498db");

// 以某个 RGB 为源，回填所有格式
function fillAll(rgb: RGB) {
  suppress.value = true;
  currentRgb.value = rgb;
  hex.value = rgbToHex(rgb);
  colorInput.value = hex.value;
  rgb.value = `${rgb.r}, ${rgb.g}, ${rgb.b}`;
  const hsvv = rgbToHsv(rgb);
  hsv.value = `${round1(hsvv.h)}, ${round1(hsvv.s)}, ${round1(hsvv.v)}`;
  const cmykk = rgbToCmyk(rgb);
  cmyk.value = `${round1(cmykk.c)}, ${round1(cmykk.m)}, ${round1(cmykk.y)}, ${round1(cmykk.k)}`;
  suppress.value = false;
  invalidField.value = null;
}

function recompute(source: Source) {
  let rgbv: RGB | null = null;
  if (source === "hex") rgbv = hexToRgb(hex.value);
  else if (source === "rgb") rgbv = parseRgb(rgb.value);
  else if (source === "hsv") {
    const v = parseHsv(hsv.value);
    if (v) rgbv = hsvToRgb(v);
  } else {
    const v = parseCmyk(cmyk.value);
    if (v) rgbv = cmykToRgb(v);
  }
  if (!rgbv) {
    invalidField.value = source;
    return;
  }
  fillAll(rgbv);
}

watch(hex, () => {
  if (!suppress.value) recompute("hex");
});
watch(rgb, () => {
  if (!suppress.value) recompute("rgb");
});
watch(hsv, () => {
  if (!suppress.value) recompute("hsv");
});
watch(cmyk, () => {
  if (!suppress.value) recompute("cmyk");
});

// 原生取色器：直接作为 hex 源重新计算
watch(colorInput, (v) => {
  if (suppress.value) return;
  suppress.value = true;
  hex.value = v;
  suppress.value = false;
  recompute("hex");
});

// 初始化时统一回填一次，保证四种格式显示一致
const initRgb = hexToRgb(hex.value);
if (initRgb) fillAll(initRgb);

const randomize = () => {
  fillAll({
    r: Math.floor(Math.random() * 256),
    g: Math.floor(Math.random() * 256),
    b: Math.floor(Math.random() * 256),
  });
};

const copyHex = () => {
  if (!hex.value) return;
  writeText(hex.value);
  message.success("复制成功");
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
    <!-- 预览 -->
    <div class="preview-block" :style="{ background: currentHex, color: previewTextColor }">
      <span class="preview-hex">{{ currentHex }}</span>
    </div>

    <n-space :size="8" class="preview-actions" align="center">
      <span class="picker-label">取色器</span>
      <input v-model="colorInput" type="color" class="color-picker" />
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button type="primary" @click="randomize">
            <template #icon>
              <n-icon>
                <Shuffle />
              </n-icon>
            </template>
          </n-button>
        </template>
        随机颜色
      </n-tooltip>
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button @click="copyHex">
            <template #icon>
              <n-icon>
                <Copy />
              </n-icon>
            </template>
          </n-button>
        </template>
        复制 HEX
      </n-tooltip>
    </n-space>

    <div class="fmt-form">
      <div class="fmt-row">
        <span class="tb-editor-label">HEX</span>
        <div class="fmt-field">
          <n-input v-model:value="hex" class="mono-input" placeholder="#rrggbb 或 #rgb" />
          <n-text v-if="invalidField === 'hex'" type="error" class="fmt-error">格式无效</n-text>
        </div>
      </div>
      <div class="fmt-row">
        <span class="tb-editor-label">RGB</span>
        <div class="fmt-field">
          <n-input v-model:value="rgb" class="mono-input" placeholder="r, g, b（0-255）" />
          <n-text v-if="invalidField === 'rgb'" type="error" class="fmt-error">格式无效</n-text>
        </div>
      </div>
      <div class="fmt-row">
        <span class="tb-editor-label">HSV</span>
        <div class="fmt-field">
          <n-input v-model:value="hsv" class="mono-input" placeholder="h, s, v（0-360, 0-100, 0-100）" />
          <n-text v-if="invalidField === 'hsv'" type="error" class="fmt-error">格式无效</n-text>
        </div>
      </div>
      <div class="fmt-row">
        <span class="tb-editor-label">CMYK</span>
        <div class="fmt-field">
          <n-input v-model:value="cmyk" class="mono-input" placeholder="c, m, y, k（0-100）" />
          <n-text v-if="invalidField === 'cmyk'" type="error" class="fmt-error">格式无效</n-text>
        </div>
      </div>
    </div>
    </section>
  </div>
</template>

<style scoped>
.preview-block {
  height: 110px;
  border-radius: 12px;
  border: 1px solid var(--tb-border);
  display: flex;
  align-items: center;
  justify-content: center;
  font-family: var(--tb-font-mono);
  font-size: 16px;
  font-weight: 600;
  transition: background-color 0.15s;
}

.preview-actions {
  margin-top: 12px;
}

.picker-label {
  font-size: 13px;
  color: var(--tb-text-2);
}

.color-picker {
  width: 36px;
  height: 28px;
  padding: 0;
  border: 1px solid var(--tb-border);
  border-radius: 6px;
  background: transparent;
  cursor: pointer;
}

.fmt-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-top: 14px;
}

.fmt-row {
  display: flex;
  align-items: flex-start;
  gap: 12px;
}

.fmt-row .tb-editor-label {
  flex-shrink: 0;
  width: 56px;
  padding-top: 8px;
  margin-bottom: 0;
}

.fmt-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.fmt-error {
  font-size: 12px;
}

.mono-input :deep(.n-input__input-el) {
  font-family: var(--tb-font-mono);
}
</style>
