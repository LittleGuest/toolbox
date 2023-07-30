<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const conversion = ref();
const encoding = ref();
const input = ref("");
const output = ref("");

const paste = (value) => {
  console.log("paste", value);
};

const copy = (value) => {
  console.log("copy", value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-form-item label="Conversion">
      <strong>select which conversion mode you want to use</strong>
      <el-select v-model="conversion" class="m-2" size="large">
        <el-option key="Encode" label="Encode" value="Encode" />
        <el-option key="Decode" label="Decode" value="Decode" />
      </el-select>
    </el-form-item>

    <el-form-item label="Input">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste(input)" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
      </el-button-group>
      <el-input v-model="input" :rows="10" type="textarea" />
    </el-form-item>

    <el-form-item label="Output">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="CopyDocument" @click="copy(output)" />
      </el-button-group>
      <el-input v-model="output" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
