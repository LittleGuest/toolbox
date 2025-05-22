<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import xmlFormat from "xml-formatter";

import CodeMirror from "vue-codemirror6";

const indent = ref("    ");
const xml = ref("");

const formatSql = () => {
  xml.value = xmlFormat(xml.value, {
    indentation: indent.value,
    collapseContent: true,
    lineSeparator: "\n",
  });
};

const paste = async () => {
  const clip = await readText();
  xml.value = clip;
};

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <!-- <el-form label-position="right" label-width="100px"> -->
  <!--   <el-form-item label="缩进"> -->
  <!--     <el-select v-model="indent" class="m-2" size="large"> -->
  <!--       <el-option key="2" label="2" value="  " /> -->
  <!--       <el-option key="4" label="4" value="    " /> -->
  <!--     </el-select> -->
  <!--   </el-form-item> -->
  <!---->
  <!--   <el-row> -->
  <!--     <el-col> -->
  <!--       <el-form-item label="XML"> -->
  <!--         <el-button-group class="ml-4"> -->
  <!--           <el-button type="primary" :icon="Document" @click="paste(xml)" /> -->
  <!--           <el-button type="primary" :icon="CopyDocument" @click="copy(xml)" /> -->
  <!--           <el-button @click="formatSql">格式化</el-button> -->
  <!--         </el-button-group> -->
  <!--       </el-form-item> -->
  <!--       <code-mirror v-model="xml" /> -->
  <!--     </el-col> -->
  <!--   </el-row> -->
  <!-- </el-form> -->
</template>
