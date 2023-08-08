<script setup>
import { ref } from "vue";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument } from "@element-plus/icons-vue";
import { Vue3JsonEditor } from 'vue3-json-editor'

const input = ref();

const change = (value) => {
  input.value = value;
}

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

  <Vue3JsonEditor v-model="input" :show-btns="false" :expandedOnStart="true" @json-change="change" />
</template>

