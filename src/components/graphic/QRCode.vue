<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument } from "@element-plus/icons-vue";

const input = ref("");
const output = ref("");


async function api() {
  const res = await invoke("qrcode", { input: input.value });
  output.value = res.replace('<?xml version="1.0" encoding="UTF-8"?>', "");
}

const change = (value) => {
  api();
  console.log("ouput", output.value);
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
    <el-row>
      <el-col><el-form-item label="Input">
          <el-button-group class="ml-4">
            <el-button type="primary" :icon="Document" @click="paste(input)" />
            <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
          </el-button-group>
          <el-input v-model="input" @input="change" />
        </el-form-item>
      </el-col>
      <el-col>
        <!--
        <el-form-item label="Output">
          <el-button-group class="ml-4">
            <el-button type="primary" :icon="CopyDocument" @click="copy(output)" />
          </el-button-group>
          <el-input v-model="output" :rows="10" type="textarea" />
        </el-form-item>
        -->
        {{ output }}
      </el-col>
    </el-row>
  </el-form>
</template>
