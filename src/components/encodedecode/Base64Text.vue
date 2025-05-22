<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";

const input = ref("");
const output = ref("");

const encodeBase64TextApi = async () => {
  return await invoke("encode_base64_text", { input: input.value });
};

const decodeBase64TextApi = async () => {
  return await invoke("decode_base64_text", { input: output.value });
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
  <n-form label-placement="left">
    <n-form-item label="操作">
      <n-button-group>
        <n-button @click="pasteInput">
          <template #icon>
            <n-icon>
              <Paste />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="copy(input)">
          <template #icon>
            <n-icon>
              <Copy />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="clear">
          <template #icon>
            <n-icon>
              <Close />
            </n-icon>
          </template>
        </n-button>
      </n-button-group>
    </n-form-item>
    <n-form-item label="输入">
      <n-input placeholder="" v-model:value="input" :rows="10" type="textarea" />
    </n-form-item>
    <n-form-item label="编码/解码">
      <n-button @click="encode">
        <template #icon>
          <n-icon>
            <ArrowDown />
          </n-icon>
        </template>
      </n-button>
      <n-button @click="decode">
        <template #icon>
          <n-icon>
            <ArrowUp />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="操作">
      <n-button-group>
        <n-button @click="pasteOutput()">
          <template #icon>
            <n-icon>
              <Paste />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="copy(output)">
          <template #icon>
            <n-icon>
              <Copy />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="clear">
          <template #icon>
            <n-icon>
              <Close />
            </n-icon>
          </template>
        </n-button>
      </n-button-group>
    </n-form-item>
    <n-form-item label="输出">
      <n-input placeholder="" v-model:value="output" :rows="10" type="textarea" />
    </n-form-item>
  </n-form>
</template>
