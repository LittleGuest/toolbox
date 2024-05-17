<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const formatterNumber = ref();
const inputType = ref("Decimal");
const input = ref("");
const binary = ref("");
const octal = ref("");
const decimal = ref("");
const hex = ref("");

const api = async () => {
  const res = await invoke("number_base", { inputType: inputType.value, input: input.value });
  console.log(res);
  binary.value = res.binary;
  octal.value = res.octal;
  decimal.value = res.decimal;
  hex.value = res.hex;
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
    <!--
    <el-form-item label="Formatter number">
      <el-switch v-model="formatterNumber" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N"
        active-value="1" inactive-value="0" />
    </el-form-item>
    -->

    <el-form-item label="输入类型">
      <el-select v-model="inputType" class="m-2" size="large">
        <el-option key="Binary" label="二进制" value="Binary" />
        <el-option key="Octal" label="八进制" value="Octal" />
        <el-option key="Decimal" label="十进制" value="Decimal" />
        <el-option key="Hex" label="十六进制" value="Hex" />
      </el-select>
    </el-form-item>

    <el-form-item label="输入">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
      </el-button-group>
      <el-input v-model="input" @input="change" maxlength=19 />
    </el-form-item>

    <el-form-item label="二进制">
      <el-text class="mx-1">{{ binary }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(binary)" />
    </el-form-item>

    <el-form-item label="八进制">
      <el-text class="mx-1">{{ octal }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(octal)" />
    </el-form-item>

    <el-form-item label="十进制">
      <el-text class="mx-1">{{ decimal }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(decimal)" />
    </el-form-item>

    <el-form-item label="十六进制">
      <el-text class="mx-1">{{ hex }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hex)" />
    </el-form-item>
  </el-form>
</template>
