<script setup lang="ts">
import { computed, ref } from "vue";
import CryptoJS from "crypto-js";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowDown, ArrowUp, Copy, Paste, Close, View, ViewOff } from "@vicons/carbon";

const message = useMessage();

// ---------------- 配置 ----------------
const ALG_OPTIONS = [
  { label: "AES", value: "AES" },
  { label: "DES", value: "DES" },
  { label: "TripleDES", value: "TripleDES" },
  { label: "RC4", value: "RC4" },
  { label: "Rabbit", value: "Rabbit" },
];

const MODE_OPTIONS = [
  { label: "ECB", value: "ECB" },
  { label: "CBC", value: "CBC" },
];

const algorithm = ref("AES");
const mode = ref("CBC");
const key = ref("");
const iv = ref("");
const format = ref("Base64");
const showKey = ref(false);

const hasMode = computed(() =>
  algorithm.value === "AES" || algorithm.value === "DES" || algorithm.value === "TripleDES"
);

// ---------------- 输入输出 ----------------
const plain = ref("");
const cipher = ref("");
const output = ref("");

// ---------------- 加密 / 解密 ----------------
const buildOpts = () => {
  const opts: Record<string, unknown> = {
    mode: mode.value === "CBC" ? CryptoJS.mode.CBC : CryptoJS.mode.ECB,
    padding: CryptoJS.pad.Pkcs7,
  };
  if (mode.value === "CBC" && iv.value.trim()) {
    opts.iv = CryptoJS.enc.Utf8.parse(iv.value.trim());
  }
  return opts;
};

const formatResult = (result: any) =>
  format.value === "Base64"
    ? result.toString()
    : result.ciphertext.toString(CryptoJS.enc.Hex);

const encrypt = () => {
  if (!key.value) {
    message.warning("请输入密钥");
    return;
  }
  if (!plain.value) {
    message.warning("请输入明文");
    return;
  }
  try {
    const alg: any = (CryptoJS as any)[algorithm.value];
    let result: any;
    if (algorithm.value === "RC4" || algorithm.value === "Rabbit") {
      result = alg.encrypt(plain.value, key.value);
    } else {
      result = alg.encrypt(
        CryptoJS.enc.Utf8.parse(plain.value),
        CryptoJS.enc.Utf8.parse(key.value),
        buildOpts()
      );
    }
    output.value = formatResult(result);
    message.success("加密成功");
  } catch (e) {
    message.error(`加密失败：${e}`);
  }
};

const decrypt = () => {
  if (!key.value) {
    message.warning("请输入密钥");
    return;
  }
  const cipherText = cipher.value.trim();
  if (!cipherText) {
    message.warning("请输入密文");
    return;
  }
  try {
    const alg: any = (CryptoJS as any)[algorithm.value];
    const parsed = CryptoJS.lib.CipherParams.create({
      ciphertext:
        format.value === "Base64"
          ? CryptoJS.enc.Base64.parse(cipherText)
          : CryptoJS.enc.Hex.parse(cipherText),
    } as any);
    let plainText: string;
    if (algorithm.value === "RC4" || algorithm.value === "Rabbit") {
      plainText = alg.decrypt(parsed, key.value).toString(CryptoJS.enc.Utf8);
    } else {
      plainText = alg
        .decrypt(parsed, CryptoJS.enc.Utf8.parse(key.value), buildOpts())
        .toString(CryptoJS.enc.Utf8);
    }
    if (!plainText) {
      message.error("解密失败，请检查密钥、IV 或密文格式");
      return;
    }
    output.value = plainText;
    message.success("解密成功");
  } catch {
    message.error("解密失败，请检查密钥、IV 或密文格式");
  }
};

// ---------------- 操作 ----------------
const pastePlain = async () => {
  try {
    plain.value = await readText();
  } catch {}
};

const pasteCipher = async () => {
  try {
    cipher.value = await readText();
  } catch {}
};

const copy = async (value: string) => {
  if (!value) return;
  await writeText(value);
  message.success("复制成功");
};

const clear = () => {
  plain.value = "";
  cipher.value = "";
  output.value = "";
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <!-- 配置区 -->
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">算法</span>
          <n-select v-model:value="algorithm" :options="ALG_OPTIONS" style="width: 160px" />
        </div>
        <div class="tb-config-item" v-if="hasMode">
          <span class="tb-config-label">模式</span>
          <n-select v-model:value="mode" :options="MODE_OPTIONS" style="width: 120px" />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">密钥</span>
          <n-input
            v-model:value="key"
            :type="showKey ? 'text' : 'password'"
            placeholder="请输入密钥"
            style="width: 320px"
          >
            <template #suffix>
              <n-button text :focusable="false" @click="showKey = !showKey">
                <template #icon>
                  <n-icon><ViewOff v-if="showKey" /><View v-else /></n-icon>
                </template>
              </n-button>
            </template>
          </n-input>
        </div>
        <div class="tb-config-item" v-if="hasMode && mode === 'CBC'">
          <span class="tb-config-label">IV 向量</span>
          <n-input
            v-model:value="iv"
            placeholder="IV（CBC 模式可选，默认全零）"
            style="width: 320px"
          />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">输出格式</span>
          <n-radio-group v-model:value="format">
            <n-radio-button value="Base64">Base64</n-radio-button>
            <n-radio-button value="Hex">Hex</n-radio-button>
          </n-radio-group>
        </div>
      </div>

      <!-- 输入区 -->
      <div class="tb-editor-grid tb-mono">
        <div class="tb-editor">
          <span class="tb-editor-label">明文</span>
          <div class="tb-toolbar">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button size="small" @click="pastePlain">
                  <template #icon><n-icon><Paste /></n-icon></template>
                </n-button>
              </template>
              粘贴明文
            </n-tooltip>
          </div>
          <n-input
            v-model:value="plain"
            type="textarea"
            :rows="8"
            placeholder="请输入明文"
          />
        </div>
        <div class="tb-editor">
          <span class="tb-editor-label">密文</span>
          <div class="tb-toolbar">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button size="small" @click="pasteCipher">
                  <template #icon><n-icon><Paste /></n-icon></template>
                </n-button>
              </template>
              粘贴密文
            </n-tooltip>
          </div>
          <n-input
            v-model:value="cipher"
            type="textarea"
            :rows="8"
            placeholder="请输入密文"
          />
        </div>
      </div>

      <!-- 加密 / 解密 -->
      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="encrypt">
              <template #icon><n-icon><ArrowDown /></n-icon></template>
            </n-button>
          </template>
          加密
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="decrypt">
              <template #icon><n-icon><ArrowUp /></n-icon></template>
            </n-button>
          </template>
          解密
        </n-tooltip>
      </div>

      <!-- 输出区 -->
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">加密 / 解密结果</span>
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" @click="pastePlain">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴输入
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" :disabled="!output" @click="copy(output)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制输出
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" @click="clear">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
        <n-input
          v-model:value="output"
          type="textarea"
          :rows="8"
          placeholder="加密 / 解密结果"
        />
      </div>

      <!-- 说明 -->
      <n-text depth="3" class="tip-text">
        提示：ECB 模式不推荐用于生产环境（相同明文块产生相同密文，易被模式分析攻击）；CBC 模式建议使用随机 IV 并妥善保存。
        解密失败常见原因：密钥或 IV 不一致、密文格式与「输出格式」不匹配、密文被截断或篡改（padding 校验失败）。
      </n-text>
    </section>
  </div>
</template>

<style scoped>
.tip-text {
  display: block;
  margin-top: 14px;
  font-size: 12px;
  line-height: 1.8;
}
</style>
