<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument, Close } from "@element-plus/icons-vue";

const compress = ref(true);
const input = ref("");
const output = ref("");


const api = async () => {
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

const clear = () => {
  input.value = "";
  output.value = "";
};

</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-form-item label="Configuration">
      <strong>GZip Compress/Decompress</strong>
      <el-switch v-model="compress" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N" />
    </el-form-item>

    <el-form-item label="Cron expression to parse">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste(input)" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
        <el-button type="primary" :icon="Close" @click="clear" />
      </el-button-group>
      <el-input v-model="output" :rows="10" type="textarea" />
    </el-form-item>

    <el-form-item label="Next scheduled dates">
      <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
      <el-input v-model="output" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
