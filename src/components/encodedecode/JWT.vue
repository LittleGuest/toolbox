<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const token = ref("");
const header = ref("");
const payload = ref("");

const paste = (value) => {
  console.log("paste", value);
};

const copy = (value) => {
  console.log("copy", value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-form-item label="Jwt Token">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste(token)" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(token)" />
      </el-button-group>
      <el-input v-model="token" :rows="10" type="textarea" />
    </el-form-item>

    <el-form-item label="Header">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="CopyDocument" @click="copy(header)" />
      </el-button-group>
      <el-input v-model="header" :rows="10" type="textarea" />
    </el-form-item>

    <el-form-item label="Payload">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="CopyDocument" @click="copy(payload)" />
      </el-button-group>
      <el-input v-model="payload" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
