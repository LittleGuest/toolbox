<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";

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
  <n-card title="JWT 解码">
    <n-space vertical size="large">
      <n-input
        v-model:value="token"
        type="textarea"
        :autosize="{ minRows: 5, maxRows: 12 }"
        placeholder="粘贴 JWT Token"
      />
      <n-space>
        <n-button type="primary" :disabled="!canDecode" @click="decode">解码</n-button>
        <n-button @click="paste">粘贴并解码</n-button>
        <n-button :disabled="!token" @click="copy(token)">复制 Token</n-button>
      </n-space>
      <n-grid :cols="2" :x-gap="16" responsive="screen">
        <n-grid-item>
          <n-card title="Header" size="small">
            <template #header-extra>
              <n-button text :disabled="!header" @click="copy(header)">复制</n-button>
            </template>
            <n-code :code="header || '{}'" language="json" word-wrap />
          </n-card>
        </n-grid-item>
        <n-grid-item>
          <n-card title="Payload" size="small">
            <template #header-extra>
              <n-button text :disabled="!payload" @click="copy(payload)">复制</n-button>
            </template>
            <n-code :code="payload || '{}'" language="json" word-wrap />
          </n-card>
        </n-grid-item>
      </n-grid>
      <n-card title="完整解码结果" size="small">
        <template #header-extra>
          <n-button text :disabled="!decoded" @click="copy(decoded)">复制</n-button>
        </template>
        <n-code :code="decoded || '{}'" language="json" word-wrap />
      </n-card>
    </n-space>
  </n-card>
</template>
