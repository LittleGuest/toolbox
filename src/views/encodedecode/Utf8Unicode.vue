<script setup lang="ts">
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";

const message = useMessage();

const unicodeInput = ref("");
const unicodeOutput = ref("");

const unicodeEscape = (s: string): string => {
  let out = "";
  for (const ch of Array.from(s)) {
    const c1 = ch.charCodeAt(0);
    if (c1 >= 0xd800 && c1 <= 0xdbff) {
      const c2 = ch.charCodeAt(1);
      out += "\\u" + c1.toString(16).padStart(4, "0") + "\\u" + c2.toString(16).padStart(4, "0");
    } else {
      out += "\\u" + c1.toString(16).padStart(4, "0");
    }
  }
  return out;
};

const unicodeUnescape = (s: string): string => {
  let out = "";
  let i = 0;
  while (i < s.length) {
    if (s[i] === "\\" && s[i + 1] === "u" && /^[0-9a-fA-F]{4}$/.test(s.slice(i + 2, i + 6))) {
      const code = parseInt(s.slice(i + 2, i + 6), 16);
      if (code >= 0xd800 && code <= 0xdbff) {
        const next = s.slice(i + 6).match(/^\\u[0-9a-fA-F]{4}/);
        if (next) {
          const code2 = parseInt(next[0].slice(2), 16);
          if (code2 >= 0xdc00 && code2 <= 0xdfff) {
            out += String.fromCharCode(code, code2);
            i += 12;
            continue;
          }
        }
      }
      out += String.fromCharCode(code);
      i += 6;
    } else {
      out += s[i];
      i++;
    }
  }
  return out;
};

const encodeUnicode = () => {
  if (!unicodeInput.value) return;
  unicodeOutput.value = unicodeEscape(unicodeInput.value);
};

const decodeUnicode = () => {
  if (!unicodeInput.value) return;
  unicodeOutput.value = unicodeUnescape(unicodeInput.value);
};

// ---------------- 辅助操作 ----------------
const paste = async (target: { value: string }) => {
  try {
    target.value = await readText();
  } catch {}
};

const copy = (value: string) => {
  if (!value) return;
  writeText(value);
  message.success("复制成功");
};

const clearUnicode = () => {
  unicodeInput.value = "";
  unicodeOutput.value = "";
};
</script>

<template>
  <div>
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输入</span>
        <n-input
          v-model:value="unicodeInput"
          type="textarea"
          :rows="10"
          placeholder="请输入要转换的文本或 \\u 转义序列"
        />
      </div>

      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="encodeUnicode">
              <template #icon><n-icon><ArrowDown /></n-icon></template>
            </n-button>
          </template>
          编码
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="decodeUnicode">
              <template #icon><n-icon><ArrowUp /></n-icon></template>
            </n-button>
          </template>
          解码
        </n-tooltip>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输出</span>
        <n-input
          v-model:value="unicodeOutput"
          type="textarea"
          :rows="10"
          placeholder="转换结果将显示在这里"
        />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="paste(unicodeInput)">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴输入
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="paste(unicodeOutput)">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴输出
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="copy(unicodeOutput)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制输出
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="clearUnicode">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
      </div>
  </div>
</template>
