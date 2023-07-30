<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const formatterNumber = ref();
const inputType = ref();
const hmacMode = ref();
const input = ref("");
const hexadecimal = ref("");
const decimal = ref("");
const octal = ref("");
const binary = ref("");

async function api() {
  hash.value = await invoke("hash", { uppercase: uppercase.value, outputType: outputType.value, hmacMode: hmacMode.value, input: input.value });
}

const change = (value) => {
  api();
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
    <el-form-item label="Formatter number">
      <el-switch v-model="formatterNumber" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N"
        active-value="1" inactive-value="0" />
    </el-form-item>

    <el-form-item label="Input Type">
      <el-select v-model="inputType" class="m-2" size="large">
        <el-option key="Decimal" label="Decimal" value="Decimal" />
        <el-option key="Octal" label="Octal" value="Octal" />
        <el-option key="Hex" label="Hex" value="Hex" />
        <el-option key="Binary" label="Binary" value="Binary" />
      </el-select>
    </el-form-item>

    <el-form-item label="Input">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste(input)" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
      </el-button-group>
      <el-input v-model="input" :rows="10" type="textarea" />
    </el-form-item>

    <el-form-item label="Hexadecimal">
      <el-text class="mx-1">{{ hexadecimal }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hexadecimal)" />
    </el-form-item>

    <el-form-item label="Decimal">
      <el-text class="mx-1">{{ decimal }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(decimal)" />
    </el-form-item>

    <el-form-item label="Octal">
      <el-text class="mx-1">{{ octal }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(octal)" />
    </el-form-item>

    <el-form-item label="Binary">
      <el-text class="mx-1">{{ binary }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(binary)" />
    </el-form-item>
  </el-form>
</template>
