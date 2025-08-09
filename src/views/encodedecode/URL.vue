<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";

const message = useMessage();

const input = ref("");
const output = ref("");

const encodeApi = async () => {
  return await invoke("encode_url", {
    input: input.value
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));

};

const decodeApi = async () => {
  return await invoke("decode_url", {
    input: output.value
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const encode = async () => {
  output.value = await encodeApi();
};

const decode = async () => {
  input.value = await decodeApi();
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
  <n-form label-placement="left" label-width="80">
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
