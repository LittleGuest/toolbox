<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument, ArrowUpBold, ArrowDownBold, Close } from "@element-plus/icons-vue";

const input = ref("");
const output = ref("");

async function encodeApi() {
  return await invoke("encode_url", { input: input.value });
}

async function decodeApi() {
  return await invoke("decode_url", { input: output.value });
}

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
  <el-form label-position="right" label-width="100px">
    <el-form-item label="输入">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="pasteInput" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
        <el-button type="primary" :icon="Close" @click="clear" />
      </el-button-group>
      <el-input v-model="input" :rows="10" type="textarea" />
    </el-form-item>
    <el-form-item label="编码/解码">
      <el-button :icon="ArrowDownBold" @click="encode" />
      <el-button :icon="ArrowUpBold" @click="decode" />
    </el-form-item>

    <el-form-item label="输出">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="pasteOutput" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(output)" />
        <el-button type="primary" :icon="Close" @click="clear" />
      </el-button-group>
      <el-input v-model="output" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
