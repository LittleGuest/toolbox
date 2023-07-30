<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument, ArrowUpBold, ArrowDownBold } from "@element-plus/icons-vue";

const input = ref("");
const output = ref("");

async function encodeBase64TextApi() {
  input.value = await invoke("encode_base64_text", { input: input.value });
}

async function decodeBase64TextApi() {
  output.value = await invoke("decode_base64_text", { input: output.value });
}

const encode = () => {
  output.value = encodeBase64TextApi();
};

const decode = () => {
  input.value = decodeBase64TextApi();
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <!--
    <el-form-item label="Configutaion">
      <strong>select which conversion mode you want to use</strong>
      <el-select v-model="conversion" class="m-2" size="large">
        <el-option key="Encode" label="Encode" value="Encode" />
        <el-option key="Decode" label="Decode" value="Decode" />
      </el-select>
    </el-form-item>

    <el-form-item label="Encoding">
      <strong>select which encoding do you want to use</strong>
      <el-select v-model="encoding" class="m-2" size="large">
        <el-option key="UTF-8" label="UTF-8" value="UTF-8" />
      </el-select>
    </el-form-item>
    -->

    <el-form-item label="输入">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
      </el-button-group>
      <el-input v-model="input" :rows="10" type="textarea" />
    </el-form-item>
    <el-form-item label="编码/解码">
      <el-button :icon="ArrowDownBold" @click="encode" />
      <el-button :icon="ArrowUpBold" @click="decode" />
    </el-form-item>


    <el-form-item label="输出">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(output)" />
      </el-button-group>
      <el-input v-model="output" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
