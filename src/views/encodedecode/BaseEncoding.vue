<script setup lang="ts">
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";
import Base64Text from "./Base64Text.vue";
import Base64Image from "./Base64Image.vue";

const message = useMessage();

const activeTab = ref("base64text");

// ---------------- Base32 ----------------
const BASE32_ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

const base32Input = ref("");
const base32Output = ref("");

const base32Encode = (s: string): string => {
  const bytes = new TextEncoder().encode(s);
  let bits = "";
  for (const b of bytes) bits += b.toString(2).padStart(8, "0");
  let out = "";
  for (let i = 0; i < bits.length; i += 5) {
    const chunk = bits.slice(i, i + 5).padEnd(5, "0");
    out += BASE32_ALPHABET[parseInt(chunk, 2)];
  }
  const pad = (8 - (out.length % 8)) % 8;
  return out + "=".repeat(pad);
};

const base32Decode = (s: string): string => {
  const cleaned = s.replace(/\s+/g, "").toUpperCase().replace(/=+$/, "");
  let bits = "";
  for (const ch of cleaned) {
    const idx = BASE32_ALPHABET.indexOf(ch);
    if (idx === -1) throw new Error("非法 Base32 字符");
    bits += idx.toString(2).padStart(5, "0");
  }
  const bytes: number[] = [];
  for (let i = 0; i + 8 <= bits.length; i += 8) {
    bytes.push(parseInt(bits.slice(i, i + 8), 2));
  }
  return new TextDecoder().decode(new Uint8Array(bytes));
};

const encodeBase32 = () => {
  if (!base32Input.value) return;
  try {
    base32Output.value = base32Encode(base32Input.value);
  } catch (e) {
    message.error(String(e));
  }
};

const decodeBase32 = () => {
  if (!base32Input.value) return;
  try {
    base32Output.value = base32Decode(base32Input.value);
  } catch (e) {
    message.error(String(e));
  }
};

// ---------------- Base58 ----------------
const BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

const base58Input = ref("");
const base58Output = ref("");

const base58Encode = (s: string): string => {
  const bytes = Array.from(new TextEncoder().encode(s));
  let zeros = 0;
  while (zeros < bytes.length && bytes[zeros] === 0) zeros++;
  let num = BigInt(0);
  for (const b of bytes) num = num * 256n + BigInt(b);
  let out = "";
  while (num > 0n) {
    const rem = num % 58n;
    out = BASE58_ALPHABET[Number(rem)] + out;
    num = num / 58n;
  }
  return "1".repeat(zeros) + out;
};

const base58Decode = (s: string): string => {
  let zeros = 0;
  while (zeros < s.length && s[zeros] === "1") zeros++;
  let num = BigInt(0);
  for (const ch of s) {
    const idx = BASE58_ALPHABET.indexOf(ch);
    if (idx === -1) throw new Error("非法 Base58 字符");
    num = num * 58n + BigInt(idx);
  }
  const bytes: number[] = [];
  while (num > 0n) {
    bytes.unshift(Number(num % 256n));
    num = num / 256n;
  }
  const result = new Uint8Array(zeros + bytes.length);
  result.set(bytes, zeros);
  return new TextDecoder().decode(result);
};

const encodeBase58 = () => {
  if (!base58Input.value) return;
  try {
    base58Output.value = base58Encode(base58Input.value);
  } catch (e) {
    message.error(String(e));
  }
};

const decodeBase58 = () => {
  if (!base58Input.value) return;
  try {
    base58Output.value = base58Decode(base58Input.value);
  } catch (e) {
    message.error(String(e));
  }
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

const clear32 = () => {
  base32Input.value = "";
  base32Output.value = "";
};

const clear58 = () => {
  base58Input.value = "";
  base58Output.value = "";
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <n-tabs v-model:value="activeTab" type="line" animated>
        <!-- Base64 文本 -->
        <n-tab-pane name="base64text" tab="Base64 文本"><Base64Text /></n-tab-pane>
        <n-tab-pane name="base64img" tab="Base64 图片"><Base64Image /></n-tab-pane>

        <!-- Base32 -->
        <n-tab-pane name="base32" tab="Base32">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="base32Input"
              type="textarea"
              :rows="10"
              placeholder="请输入要编码 / 解码的文本"
            />
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="paste(base32Input)">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输入
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="copy(base32Input)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输入
              </n-tooltip>
            </div>
          </div>

          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="encodeBase32">
                  <template #icon><n-icon><ArrowDown /></n-icon></template>
                </n-button>
              </template>
              编码
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="decodeBase32">
                  <template #icon><n-icon><ArrowUp /></n-icon></template>
                </n-button>
              </template>
              解码
            </n-tooltip>
          </div>

          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="base32Output"
              type="textarea"
              :rows="10"
              placeholder="编码 / 解码结果将显示在这里"
            />
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="paste(base32Output)">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="copy(base32Output)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="clear32">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <!-- Base58 -->
        <n-tab-pane name="base58" tab="Base58">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="base58Input"
              type="textarea"
              :rows="10"
              placeholder="请输入要编码 / 解码的文本"
            />
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="paste(base58Input)">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输入
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="copy(base58Input)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输入
              </n-tooltip>
            </div>
          </div>

          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="encodeBase58">
                  <template #icon><n-icon><ArrowDown /></n-icon></template>
                </n-button>
              </template>
              编码
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="decodeBase58">
                  <template #icon><n-icon><ArrowUp /></n-icon></template>
                </n-button>
              </template>
              解码
            </n-tooltip>
          </div>

          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="base58Output"
              type="textarea"
              :rows="10"
              placeholder="编码 / 解码结果将显示在这里"
            />
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="paste(base58Output)">
                    <template #icon><n-icon><Paste /></n-icon></template>
                  </n-button>
                </template>
                粘贴输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="copy(base58Output)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="clear58">
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
