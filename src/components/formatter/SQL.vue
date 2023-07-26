<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const language = ref("");
const indent = ref(2);
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
  <el-form label-position="right" label-width="100px" style="max-width: 460px">
    <el-form-item label="Language">
      <el-select v-model="language" class="m-2" size="large">
        <el-option key="MySQL" label="MySQL" value="MySQL" />
        <el-option key="Sqlite" label="Sqlite" value="Sqlite" />
        <el-option key="PostgreSQL" label="PostgreSQL" value="PostgreSQL" />
      </el-select>
    </el-form-item>

    <el-form-item label="Indentation">
      <el-select v-model="indent" class="m-2" size="large">
        <el-option key="2" label="2" value="2" />
        <el-option key="4" label="4" value="4" />
      </el-select>
    </el-form-item>

    <el-row>
      <el-col
        ><el-form-item label="Input">
          <el-button-group class="ml-4">
            <el-button
              type="primary"
              :icon="Document"
              v-on:click="paste(input)"
            />
            <el-button
              type="primary"
              :icon="CopyDocument"
              v-on:click="copy(input)"
            />
          </el-button-group>
          <el-input v-model="input" :rows="10" type="textarea" /> </el-form-item
      ></el-col>
      <el-col
        ><el-form-item label="Output">
          <el-button-group class="ml-4">
            <el-button
              type="primary"
              :icon="CopyDocument"
              v-on:click="copy(output)"
            />
          </el-button-group>
          <el-input
            v-model="output"
            :rows="10"
            type="textarea"
          /> </el-form-item
      ></el-col>
    </el-row>
  </el-form>
</template>
