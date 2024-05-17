<script setup>
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument } from "@element-plus/icons-vue";

onMounted(() => {
  api();
});

const input = ref("");
const datetime_utc8 = ref("");
const timestamp = ref();
const timestamp_mill = ref();

const api = async () => {
  const res = await invoke("timestamp");

  input.value = res.timestamp;
  datetime_utc8.value = res.datetime_utc8;
  timestamp.value = res.timestamp;
  timestamp_mill.value = res.timestamp + res.timestamp_mill;
}

const change = (value) => {
};

const clear = () => {
  input.value = "";
  datetime_utc8.value = "";
  timestamp.value = "";
  timestamp_mill.value = "";
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <el-form label-position="right" label-width="110px">
    <el-form-item label="时间">
      <el-button type="primary" :icon="Document" @click="paste" />
      <el-input v-model="input" @change="change" @clear="clear" clearable />
    </el-form-item>

    <el-form-item label="UTC+8">
      <el-text class="mx-1">{{ datetime_utc8 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(datetime_utc8)" />
    </el-form-item>

    <el-form-item label="时间戳（秒）">
      <el-text class="mx-1">{{ timestamp }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(timestamp)" />
    </el-form-item>

    <el-form-item label="时间戳（毫秒）">
      <el-text class="mx-1">{{ timestamp_mill }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(timestamp_mill)" />
    </el-form-item>
  </el-form>
</template>
