<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Paste, Close, Play, ArrowDown, ArrowUp } from "@vicons/carbon";

const message = useMessage();

const inputType = ref("decimal");
const input = ref("");
const binary = ref("");
const octal = ref("");
const decimal = ref("");
const hex = ref("");
const typeOptions = [
  {
    label: "二进制",
    value: "binary"
  },
  {
    label: "八进制",
    value: "octal"
  },
  {
    label: "十进制",
    value: "decimal"
  },
  {
    label: "十六进制",
    value: "hex"
  }
];

const api = async () => {
  return await invoke("number_base", {
    inputType: inputType.value,
    input: input.value,
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const change = async (value) => {
  const res = await api();
  binary.value = res.binary;
  octal.value = res.octal;
  decimal.value = res.decimal;
  hex.value = res.hex;
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};

const clear = () => {
  input.value = "";
  binary.value = "";
  octal.value = "";
  decimal.value = "";
  hex.value = "";
};

const convInput = ref("");
const convOutput = ref("");
const fromBase = ref(10);
const toBase = ref(16);

const DIGITS = "0123456789abcdefghijklmnopqrstuvwxyz";
const baseOptions = Array.from({ length: 35 }, (_, i) => ({
  label: `${i + 2} 进制`,
  value: i + 2,
}));

const isValidInBase = (s: string, base: number) => {
  const charset = DIGITS.slice(0, base);
  return new RegExp(`^[${charset}]+$`, "i").test(s);
};

const convertBase = () => {
  const s = convInput.value.trim();
  if (!s) {
    message.warning("请输入数值");
    return;
  }
  const from = fromBase.value;
  const to = toBase.value;
  if (!isValidInBase(s, from)) {
    message.error(`"${s}" 不是合法的 ${from} 进制数`);
    return;
  }
  let big = 0n;
  for (const ch of s.toLowerCase()) {
    big = big * BigInt(from) + BigInt(DIGITS.indexOf(ch));
  }
  convOutput.value = big.toString(to).toUpperCase();
};

const pasteConvInput = async () => {
  try {
    convInput.value = await readText();
  } catch {
  }
};

const clearConv = () => {
  convInput.value = "";
  convOutput.value = "";
};

const hexInput = ref("");
const hexOutput = ref("");
const hexNoSpace = ref(false);

const encodeHex = () => {
  const bytes = new TextEncoder().encode(hexInput.value);
  const sep = hexNoSpace.value ? "" : " ";
  hexOutput.value = Array.from(bytes, (b) => b.toString(16).padStart(2, "0")).join(sep);
};

const decodeHex = () => {
  const clean = hexOutput.value.replace(/\s+/g, "");
  if (clean.length % 2 !== 0 || !/^[0-9a-fA-F]*$/.test(clean)) {
    message.error("无效的十六进制输入");
    return;
  }
  const bytes = new Uint8Array(clean.length / 2);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.slice(i * 2, i * 2 + 2), 16);
  }
  try {
    hexInput.value = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    message.error("解码失败：不是有效的 UTF-8 字节序列");
  }
};

const binInput = ref("");
const binOutput = ref("");

const encodeBin = () => {
  const bytes = new TextEncoder().encode(binInput.value);
  binOutput.value = Array.from(bytes, (b) => b.toString(2).padStart(8, "0")).join(" ");
};

const decodeBin = () => {
  const clean = binOutput.value.replace(/\s+/g, "");
  if (clean.length % 8 !== 0 || !/^[01]*$/.test(clean)) {
    message.error("无效的二进制输入");
    return;
  }
  const bytes = new Uint8Array(clean.length / 8);
  for (let i = 0; i < bytes.length; i++) {
    bytes[i] = parseInt(clean.slice(i * 8, i * 8 + 8), 2);
  }
  try {
    binInput.value = new TextDecoder("utf-8", { fatal: true }).decode(bytes);
  } catch {
    message.error("解码失败：不是有效的 UTF-8 字节序列");
  }
};

const asciiInput = ref("");
const asciiOutput = ref("");
const asciiMode = ref<"text" | "code">("text");

const convertAscii = () => {
  if (asciiMode.value === "text") {
    asciiOutput.value = [...asciiInput.value]
      .map((ch) => {
        const code = ch.codePointAt(0) ?? 0;
        return `${ch} | ${code} | 0x${code.toString(16).toUpperCase()}`;
      })
      .join("\n");
    return;
  }
  const parts = asciiInput.value.split(/[\s,，]+/).filter(Boolean);
  if (parts.length === 0) {
    message.warning("请输入要转换的编码");
    return;
  }
  let result = "";
  for (const p of parts) {
    const m = p.trim().toLowerCase();
    if (/^0x[0-9a-f]+$/.test(m)) {
      const cp = parseInt(m, 16);
      if (cp > 0x10ffff) {
        message.error(`无效的码点：${p}`);
        return;
      }
      result += String.fromCodePoint(cp);
    } else if (/^\d+$/.test(m)) {
      const cp = parseInt(m, 10);
      if (cp > 0x10ffff) {
        message.error(`无效的码点：${p}`);
        return;
      }
      result += String.fromCodePoint(cp);
    } else {
      message.error(`无法识别的编码：${p}`);
      return;
    }
  }
  asciiOutput.value = result;
};

const pasteHexInput = async () => {
  try {
    hexInput.value = await readText();
  } catch {
  }
};
const pasteBinInput = async () => {
  try {
    binInput.value = await readText();
  } catch {
  }
};
const pasteAsciiInput = async () => {
  try {
    asciiInput.value = await readText();
  } catch {
  }
};

const copyText = (value: string) => {
  if (!value) {
    message.warning("内容为空");
    return;
  }
  writeText(value);
  message.success("复制成功");
};

const clearHex = () => {
  hexInput.value = "";
  hexOutput.value = "";
};
const clearBin = () => {
  binInput.value = "";
  binOutput.value = "";
};
const clearAscii = () => {
  asciiInput.value = "";
  asciiOutput.value = "";
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <n-tabs type="line" animated>
        <n-tab-pane name="common" tab="常见进制">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">输入类型</span>
              <n-select
                placeholder="请选择类型"
                size="small"
                style="width: 150px"
                :options="typeOptions"
                v-model:value="inputType"
              />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">输入</span>
              <n-input
                placeholder="请输入"
                clearable
                type="text"
                v-model:value="input"
                @update:value="change"
                @clear="clear"
                maxlength="19"
              />
            </div>
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button quaternary circle @click="paste">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button quaternary circle @click="copy(input)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
            </div>
          </div>

          <div class="tb-editor-grid">
            <div class="tb-editor">
              <span class="tb-editor-label">二进制</span>
              <n-input readonly v-model:value="binary" class="tb-mono" />
            </div>
            <div class="tb-editor">
              <span class="tb-editor-label">八进制</span>
              <n-input readonly v-model:value="octal" class="tb-mono" />
            </div>
            <div class="tb-editor">
              <span class="tb-editor-label">十进制</span>
              <n-input readonly v-model:value="decimal" class="tb-mono" />
            </div>
            <div class="tb-editor">
              <span class="tb-editor-label">十六进制</span>
              <n-input readonly v-model:value="hex" class="tb-mono" />
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="arbitrary" tab="任意进制">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">来源进制</span>
              <n-select v-model:value="fromBase" :options="baseOptions" style="width: 130px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">目标进制</span>
              <n-select v-model:value="toBase" :options="baseOptions" style="width: 130px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">输入</span>
              <n-input
                placeholder="请输入数值，如 255、ff、11111111"
                type="text"
                v-model:value="convInput"
                @keyup.enter="convertBase"
              />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="convertBase">
                    <template #icon><n-icon><Play /></n-icon></template>
                  </n-button>
                </template>
                转换
              </n-tooltip>
            </div>
          </div>

          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="convOutput"
              type="textarea"
              :rows="6"
              placeholder="转换结果"
            />
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="pasteConvInput">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输入
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copy(convOutput)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearConv">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="hex" tab="字符串 ↔ 十六进制">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="hexInput"
              type="textarea"
              :rows="6"
              placeholder="请输入文本，编码后输出十六进制"
            />
          </div>
          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="encodeHex">
                  <template #icon><n-icon><ArrowDown /></n-icon></template>
                </n-button>
              </template>
              编码
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="decodeHex">
                  <template #icon><n-icon><ArrowUp /></n-icon></template>
                </n-button>
              </template>
              解码
            </n-tooltip>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="hexOutput"
              type="textarea"
              :rows="6"
              placeholder="十六进制结果（解码时请将十六进制粘贴到此处）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-checkbox v-model:checked="hexNoSpace">连续无空格</n-checkbox>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="pasteHexInput">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输入
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyText(hexOutput)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearHex">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="bin" tab="字符串 ↔ 二进制">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="binInput"
              type="textarea"
              :rows="6"
              placeholder="请输入文本，编码后输出二进制"
            />
          </div>
          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="encodeBin">
                  <template #icon><n-icon><ArrowDown /></n-icon></template>
                </n-button>
              </template>
              编码
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="decodeBin">
                  <template #icon><n-icon><ArrowUp /></n-icon></template>
                </n-button>
              </template>
              解码
            </n-tooltip>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="binOutput"
              type="textarea"
              :rows="6"
              placeholder="二进制结果（解码时请将二进制粘贴到此处）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="pasteBinInput">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输入
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyText(binOutput)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearBin">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="ascii" tab="文本 ↔ ASCII 码">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="asciiInput"
              type="textarea"
              :rows="6"
              placeholder="文本模式下输入文本；码模式下输入十进制或十六进制码（逗号/空格分隔，支持 0x 前缀）"
            />
          </div>
          <div class="tb-config-row">
            <div class="tb-config-item">
              <n-radio-group v-model:value="asciiMode" size="small">
                <n-radio-button value="text">文本 → 码</n-radio-button>
                <n-radio-button value="code">码 → 文本</n-radio-button>
              </n-radio-group>
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="convertAscii">
                    <template #icon><n-icon><Play /></n-icon></template>
                  </n-button>
                </template>
                转换
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="asciiOutput"
              type="textarea"
              :rows="6"
              placeholder="转换结果"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="pasteAsciiInput">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输入
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyText(asciiOutput)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearAscii">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>
      </n-tabs>
    </section>
  </div>
</template>
