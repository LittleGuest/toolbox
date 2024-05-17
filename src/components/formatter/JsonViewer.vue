<script setup>
import { ref } from "vue";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument } from "@element-plus/icons-vue";
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';

const input = ref({});


const paste = async () => {
  const clip = await readText();
  input.value = JSON.parse(clip);
};

const copy = (value) => {
  writeText(JSON.stringify(value));
};
</script>

<template>
  <el-button-group class="ml-4">
    <el-button type="primary" :icon="Document" @click="paste(input)" />
    <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
  </el-button-group>

  <vue-json-pretty :data="input" />
</template>

