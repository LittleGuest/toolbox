<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";

const message = useMessage();

const input = ref("");
const output = ref("");

const encodeBase64TextApi = async () => {
  return await invoke("encode_base64_text", {
    input: input.value
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const decodeBase64TextApi = async () => {
  return await invoke("decode_base64_text", {
    input: output.value
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const encode = async () => {
  output.value = await encodeBase64TextApi();
};

const decode = async () => {
  input.value = await decodeBase64TextApi();
};

const pasteInput = async () => {
  input.value = await readText();
};

const pasteOutput = async () => {
  output.value = await readText();
};

const copy = (value) => {
  writeText(value);
};

const clear = () => {
  input.value = "";
  output.value = "";
};
</script>

<template>
  <div>
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输入</span>
        <n-input v-model:value="input" :rows="10" type="textarea" />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="pasteInput">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="copy(input)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="clear">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
      </div>

      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="encode">
              <template #icon><n-icon><ArrowDown /></n-icon></template>
            </n-button>
          </template>
          编码
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="decode">
              <template #icon><n-icon><ArrowUp /></n-icon></template>
            </n-button>
          </template>
          解码
        </n-tooltip>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输出</span>
        <n-input v-model:value="output" :rows="10" type="textarea" />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="pasteOutput">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="copy(output)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="clear">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
      </div>
  </div>
</template>
