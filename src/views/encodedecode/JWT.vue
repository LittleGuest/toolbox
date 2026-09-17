<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Play, Paste, Copy } from "@vicons/carbon";

const message = useMessage();

const token = ref("");
const header = ref("");
const payload = ref("");
const decoded = ref("");

const canDecode = computed(() => token.value.trim().length > 0);

const decode = async () => {
  if (!canDecode.value) {
    message.warning("请输入 JWT Token");
    return;
  }
  try {
    const value = await invoke<string>("decode_jwt", {
      input: token.value.trim(),
    });
    decoded.value = value;
    const parsed = JSON.parse(value);
    header.value = JSON.stringify(parsed.header ?? {}, null, 2);
    payload.value = JSON.stringify(parsed.payload ?? {}, null, 2);
  } catch (error) {
    message.error(`${error}`);
  }
};

const paste = async () => {
  token.value = await readText();
  await decode();
};

const copy = async (value: string) => {
  if (!value) {
    return;
  }
  await writeText(value);
  message.success("复制成功");
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">Token</span>
        <n-input
          v-model:value="token"
          type="textarea"
          :autosize="{ minRows: 5, maxRows: 12 }"
          placeholder="粘贴 JWT Token"
        />
        <div class="tb-action-row">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" :disabled="!canDecode" @click="decode">
                <template #icon><n-icon><Play /></n-icon></template>
              </n-button>
            </template>
            解码
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="paste">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴并解码
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button :disabled="!token" @click="copy(token)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制 Token
          </n-tooltip>
        </div>
      </div>

      <div class="tb-editor-grid">
        <div class="tb-editor">
          <span class="tb-editor-label">Header</span>
          <div class="tb-toolbar">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button text :disabled="!header" @click="copy(header)">
                  <template #icon><n-icon><Copy /></n-icon></template>
                </n-button>
              </template>
              复制
            </n-tooltip>
          </div>
          <n-code :code="header || '{}'" language="json" word-wrap />
        </div>
        <div class="tb-editor">
          <span class="tb-editor-label">Payload</span>
          <div class="tb-toolbar">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button text :disabled="!payload" @click="copy(payload)">
                  <template #icon><n-icon><Copy /></n-icon></template>
                </n-button>
              </template>
              复制
            </n-tooltip>
          </div>
          <n-code :code="payload || '{}'" language="json" word-wrap />
        </div>
      </div>

      <div class="tb-editor">
        <span class="tb-editor-label">完整解码结果</span>
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button text :disabled="!decoded" @click="copy(decoded)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制
          </n-tooltip>
        </div>
        <n-code :code="decoded || '{}'" language="json" word-wrap />
      </div>
    </section>
  </div>
</template>
