<script setup lang="ts">
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import {
  LetterUu,
  LetterAa,
  TextCreation,
  TextIndent,
  TextAlignCenter,
  TextAlignRight,
  TextUnderline,
  TextAlignJustify,
  Repeat,
  ArrowsHorizontal,
  MagicWand,
  Copy,
  Paste,
  Close,
} from "@vicons/carbon";

const message = useMessage();

const input = ref("");
const output = ref("");

const apply = (fn: (s: string) => string) => {
  output.value = fn(input.value);
};

const toUpperCase = () => apply((s) => s.toUpperCase());

const toLowerCase = () => apply((s) => s.toLowerCase());

const toTitleCase = () =>
  apply(
    (s) =>
      s
        .replace(/\b\w/g, (c) => c.toUpperCase())
        .replace(/\B\w/g, (c) => c.toLowerCase())
  );

const toSentenceCase = () =>
  apply((s) =>
    s.replace(
      /(^|[.!?。！？])(\s*)(\S)/g,
      (_m, p1: string, p2: string, p3: string) => p1 + p2 + p3.toUpperCase()
    )
  );

const toCamelCase = () =>
  apply((s) => {
    const str = s.toLowerCase().replace(/[\s_-]+(\w)/g, (_m, c: string) => c.toUpperCase());
    return str.charAt(0).toLowerCase() + str.slice(1);
  });

const toPascalCase = () =>
  apply((s) => {
    const str = s.toLowerCase().replace(/[\s_-]+(\w)/g, (_m, c: string) => c.toUpperCase());
    return str.charAt(0).toUpperCase() + str.slice(1);
  });

const toSnakeCase = () => apply((s) => s.trim().replace(/[\s_-]+/g, "_"));

const toKebabCase = () => apply((s) => s.trim().replace(/[\s_-]+/g, "-"));

const swapCase = () =>
  apply((s) =>
    [...s].map((c) => (c.toUpperCase() === c ? c.toLowerCase() : c.toUpperCase())).join("")
  );

const reverseText = () => apply((s) => [...s].reverse().join(""));

const UPSIDE_DOWN_MAP: Record<string, string> = {
  a: "ɐ", b: "q", c: "ɔ", d: "p", e: "ǝ", f: "ɟ", g: "ƃ", h: "ɥ", i: "ᴉ", j: "ɾ",
  k: "ʞ", l: "l", m: "ɯ", n: "u", o: "o", p: "d", q: "b", r: "ɹ", s: "s", t: "ʇ",
  u: "n", v: "ʌ", w: "ʍ", x: "x", y: "ʎ", z: "z",
  "0": "0", "1": "Ɩ", "2": "ᄅ", "3": "Ɛ", "4": "ㄣ", "5": "ʎ", "6": "9", "7": "ㄥ", "8": "8", "9": "6",
  "?": "¿", "!": "¡", "(": ")", ")": "(", "[": "]", "]": "[", "{": "}", "}": "{",
  "<": ">", ">": "<", ",": "'", "'": ",",
};
const upsideDown = () =>
  apply((s) => [...s].map((c) => UPSIDE_DOWN_MAP[c] ?? c).join(""));

const pasteInput = async () => {
  try {
    input.value = await readText();
  } catch {
  }
};

const copy = (value: string) => {
  if (!value) {
    message.warning("内容为空");
    return;
  }
  writeText(value);
  message.success("复制成功");
};

const clear = () => {
  input.value = "";
  output.value = "";
};
</script>

<template>
  <div>
      <div class="tb-toolbar" style="margin-bottom: 14px">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="toUpperCase">
              <template #icon><n-icon><LetterUu /></n-icon></template>
            </n-button>
          </template>
          全部大写
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="toLowerCase">
              <template #icon><n-icon><LetterAa /></n-icon></template>
            </n-button>
          </template>
          全部小写
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="toTitleCase">
              <template #icon><n-icon><TextCreation /></n-icon></template>
            </n-button>
          </template>
          单词首字母大写
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="toSentenceCase">
              <template #icon><n-icon><TextIndent /></n-icon></template>
            </n-button>
          </template>
          句子首字母大写
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="toCamelCase">
              <template #icon><n-icon><TextAlignCenter /></n-icon></template>
            </n-button>
          </template>
          驼峰 camelCase
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="toPascalCase">
              <template #icon><n-icon><TextAlignRight /></n-icon></template>
            </n-button>
          </template>
          帕斯卡 PascalCase
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="toSnakeCase">
              <template #icon><n-icon><TextUnderline /></n-icon></template>
            </n-button>
          </template>
          蛇形 snake_case
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="toKebabCase">
              <template #icon><n-icon><TextAlignJustify /></n-icon></template>
            </n-button>
          </template>
          烤肉串 kebab-case
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="swapCase">
              <template #icon><n-icon><Repeat /></n-icon></template>
            </n-button>
          </template>
          反转大小写
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="reverseText">
              <template #icon><n-icon><ArrowsHorizontal /></n-icon></template>
            </n-button>
          </template>
          反转文本
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="upsideDown">
              <template #icon><n-icon><MagicWand /></n-icon></template>
            </n-button>
          </template>
          字符倒置
        </n-tooltip>
      </div>

      <div class="tb-editor-grid">
        <div class="tb-editor tb-mono">
          <span class="tb-editor-label">输入</span>
          <n-input
            v-model:value="input"
            type="textarea"
            :rows="10"
            placeholder="请输入要转换的文本"
          />
        </div>
        <div class="tb-editor tb-mono">
          <span class="tb-editor-label">输出</span>
          <n-input
            v-model:value="output"
            type="textarea"
            :rows="10"
            placeholder="转换结果"
          />
        </div>
      </div>

      <div class="tb-toolbar" style="margin-top: 14px">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="small" quaternary @click="pasteInput">
              <template #icon><n-icon><Paste /></n-icon></template>
            </n-button>
          </template>
          粘贴到输入
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="small" quaternary @click="copy(input)">
              <template #icon><n-icon><Copy /></n-icon></template>
            </n-button>
          </template>
          复制输入
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="small" quaternary @click="copy(output)">
              <template #icon><n-icon><Copy /></n-icon></template>
            </n-button>
          </template>
          复制输出
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button size="small" quaternary @click="clear">
              <template #icon><n-icon><Close /></n-icon></template>
            </n-button>
          </template>
          清除
        </n-tooltip>
      </div>
  </div>
</template>
