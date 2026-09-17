<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Play, Reset, Copy, Close } from "@vicons/carbon";

const message = useMessage();

const length = ref(16);
const count = ref(5);
const customCharset = ref("");

const CHARSET_OPTIONS = [
  { label: "小写字母", value: "lower" },
  { label: "大写字母", value: "upper" },
  { label: "数字", value: "digit" },
  { label: "特殊符号", value: "symbol" },
];

const CHARSETS: Record<string, string> = {
  lower: "abcdefghijklmnopqrstuvwxyz",
  upper: "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
  digit: "0123456789",
  symbol: "!@#$%^&*()-_=+[]{};:,.<>?",
};

const SIMILAR_CHARS = new Set(["O", "0", "I", "l", "1"]);

const selectedSets = ref<string[]>(["lower", "upper", "digit", "symbol"]);
const excludeSimilar = ref(false);
const passwordMode = ref(false);
const output = ref("");

const dedupe = (s: string) => Array.from(new Set(s)).join("");

const mergedCharset = computed(() => {
  let s: string;
  if (customCharset.value.trim()) {
    s = customCharset.value;
  } else {
    s = selectedSets.value.map((k) => CHARSETS[k] ?? "").join("");
  }
  if (excludeSimilar.value) {
    s = Array.from(s).filter((c) => !SIMILAR_CHARS.has(c)).join("");
  }
  return dedupe(s);
});

const generateOne = (): string => {
  const charset = mergedCharset.value;
  const len = Math.max(1, length.value || 1);
  if (!charset) return "";
  const buf = new Uint32Array(Math.max(len, 8));
  crypto.getRandomValues(buf);
  let bi = 0;
  const next = (max: number): number => {
    if (bi >= buf.length) {
      crypto.getRandomValues(buf);
      bi = 0;
    }
    return buf[bi++] % max;
  };
  // 密码模式：每种已选字符集至少取一个字符，其余随机补齐，再 Fisher-Yates 打乱
  if (passwordMode.value && !customCharset.value.trim()) {
    const parts: string[] = [];
    for (const key of selectedSets.value) {
      const set = CHARSETS[key];
      if (set) parts.push(set[next(set.length)]);
    }
    while (parts.length < len) {
      parts.push(charset[next(charset.length)]);
    }
    const arr = parts.slice(0, len);
    for (let i = arr.length - 1; i > 0; i--) {
      const j = next(i + 1);
      [arr[i], arr[j]] = [arr[j], arr[i]];
    }
    return arr.join("");
  }
  let s = "";
  for (let i = 0; i < len; i++) {
    s += charset[next(charset.length)];
  }
  return s;
};

const buildLines = (): string => {
  const n = Math.max(1, count.value || 1);
  const lines: string[] = [];
  for (let i = 0; i < n; i++) {
    lines.push(generateOne());
  }
  return lines.join("\n");
};

const generate = () => {
  if (!mergedCharset.value) {
    message.warning("请选择至少一种字符集或填写自定义字符集");
    return;
  }
  output.value = buildLines();
};

// 配置变化时自动重新生成
watch(
  [length, count, selectedSets, customCharset, excludeSimilar, passwordMode],
  () => {
    if (!mergedCharset.value) {
      output.value = "";
      return;
    }
    output.value = buildLines();
  }
);

const copyAll = () => {
  if (!output.value) {
    message.warning("暂无内容可复制");
    return;
  }
  writeText(output.value);
  message.success("复制成功");
};

const clear = () => {
  output.value = "";
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">长度</span>
          <n-input-number v-model:value="length" :min="1" :max="10000" style="width: 160px" />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">数量</span>
          <n-input-number v-model:value="count" :min="1" :max="1000" style="width: 160px" />
        </div>
      </div>

      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">字符集</span>
          <n-checkbox-group v-model:value="selectedSets">
            <n-space :size="18">
              <n-checkbox
                v-for="opt in CHARSET_OPTIONS"
                :key="opt.value"
                :value="opt.value"
                :label="opt.label"
              />
            </n-space>
          </n-checkbox-group>
        </div>
      </div>

      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">自定义字符集</span>
          <n-input
            v-model:value="customCharset"
            placeholder="填写后覆盖上方字符集选择，如 abcXYZ0123"
            clearable
            style="width: 340px"
          />
        </div>
      </div>

      <div class="tb-config-row">
        <div class="tb-config-item">
          <n-checkbox v-model:checked="excludeSimilar">排除相似字符（O0Il1）</n-checkbox>
        </div>
        <div class="tb-config-item">
          <n-checkbox v-model:checked="passwordMode">密码模式（每类至少一个字符并打乱）</n-checkbox>
        </div>
        <div class="tb-config-item">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" @click="generate">
                <template #icon>
                  <n-icon><Play /></n-icon>
                </template>
              </n-button>
            </template>
            生成
          </n-tooltip>
        </div>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输出</span>
        <n-input
          v-model:value="output"
          type="textarea"
          :rows="10"
          placeholder="生成结果（每行一个）"
        />
        <div class="tb-toolbar" style="margin-top: 12px">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="generate">
                <template #icon>
                  <n-icon><Reset /></n-icon>
                </template>
              </n-button>
            </template>
            重新生成
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="copyAll">
                <template #icon>
                  <n-icon><Copy /></n-icon>
                </template>
              </n-button>
            </template>
            复制全部
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="clear">
                <template #icon>
                  <n-icon><Close /></n-icon>
                </template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
      </div>
    </section>
  </div>
</template>
