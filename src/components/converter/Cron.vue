<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const cronMode = ref(false);
const nextScheduledDates = ref(5);
const outputFormat = ref();
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
</script>

<template>
  <el-form label-position="right" label-width="180px">
    <el-form-item label="Cron Mode">
      <strong>Seconds incoluded (6-segment Cron)</strong>
      <el-switch v-model="cronMode" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N" />
    </el-form-item>

    <el-form-item label="Next scheduled dates">
      <el-input-number v-model="nextScheduledDates" :min="5" :max="365" controls-position="right" size="large"
        @change="change" />
    </el-form-item>

    <el-form-item label="Output format">
      <el-select v-model="outputFormat" class="m-2" size="large">
        <el-option key="1" label="yyyy-MM-dd HH:mm::ss" value="yyyy-MM-dd HH:mm::ss" />
        <el-option key="1" label="yyyy-MM-dd HH:mm::ss" value="yyyy-MM-dd HH:mm::ss" />
      </el-select>
    </el-form-item>

    <el-form-item label="Cron expression to parse">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste(input)" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
      </el-button-group>
      <el-input v-model="input" clearable />
    </el-form-item>

    <el-form-item label="Next scheduled dates">
      <el-button type="primary" :icon="CopyDocument" @click="copy(output)" />
      <el-input v-model="output" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
