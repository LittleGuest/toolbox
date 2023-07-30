<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const conversion = ref();
const expression = ref("");
const text = ref("");

const paste = (value) => {
  console.log("paste", value);
};

const copy = (value) => {
  console.log("copy", value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-form-item label="Configuration">
      <el-select v-model="conversion" class="m-2" size="large">
        <el-option key="Encode" label="Encode" value="Encode" />
        <el-option key="Decode" label="Decode" value="Decode" />
      </el-select>
    </el-form-item>

    <el-form-item label="Regular expression">
      <el-button type="primary" :icon="Document" @click="paste(expression)" />
      <el-input v-model="expression" clearable />
    </el-form-item>

    <el-form-item label="Text">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="CopyDocument" @click="copy(text)" />
      </el-button-group>
      <el-input v-model="text" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
