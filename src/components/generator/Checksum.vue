<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
// import { genFileId } from "element-plus";

const uppercase = ref(false);
const hashingAlgorithm = ref("md5");
const upload = ref();
const input = ref("");
const output = ref("");
const outputComparer = ref("");

const api = async () => {
  //hash.value = await invoke("hash", { uppercase: uppercase.value, outputType: outputType.value, hmacMode: hmacMode.value, input: input.value });
};

const change = (value) => {
  api();
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};

const handleExceed = (files) => {
  upload.value.clearFiles();
  const file = files[0];
  // file.uid = genFileId();
  upload.value.handleStart(file);
};

const submitUpload = () => { };
</script>

<template>
  <!-- <el-form label-position="right" label-width="100px"> -->
  <!--   <el-form-item label="大写"> -->
  <!--     <el-switch -->
  <!--       v-model="uppercase" -->
  <!--       @change="change" -->
  <!--       class="ml-2" -->
  <!--       inline-prompt -->
  <!--       style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" -->
  <!--       active-text="Y" -->
  <!--       inactive-text="N" -->
  <!--     /> -->
  <!--   </el-form-item> -->
  <!---->
  <!--   <el-form-item label="Hash算法"> -->
  <!--     <el-select v-model="hashingAlgorithm" class="m-2" size="large"> -->
  <!--       <el-option key="md5" label="md5" value="md5" /> -->
  <!--       <el-option key="sha1sum" label="sha1sum" value="sha1sum" /> -->
  <!--       <el-option key="sha224sum" label="sha224sum" value="sha224sum" /> -->
  <!--       <el-option key="sha256sum" label="sha256sum" value="sha256sum" /> -->
  <!--       <el-option key="sha384sum" label="sha384sum" value="sha384sum" /> -->
  <!--       <el-option key="sha512sum" label="sha512sum" value="sha512sum" /> -->
  <!--     </el-select> -->
  <!--   </el-form-item> -->
  <!---->
  <!--   <el-upload -->
  <!--     ref="upload" -->
  <!--     class="upload-demo" -->
  <!--     action="" -->
  <!--     :limit="1" -->
  <!--     :on-exceed="handleExceed" -->
  <!--     :auto-upload="false" -->
  <!--   > -->
  <!--     <template #trigger> -->
  <!--       <el-button type="primary">select file</el-button> -->
  <!--     </template> -->
  <!--     <el-button class="ml-3" type="success" @click="submitUpload" -->
  <!--       >计算</el-button -->
  <!--     > -->
  <!--   </el-upload> -->
  <!---->
  <!--   <el-form-item label="Output"> -->
  <!--     <el-text class="mx-1">{{ output }}</el-text> -->
  <!--     <el-button type="primary" :icon="CopyDocument" @click="copy(output)" /> -->
  <!--   </el-form-item> -->
  <!---->
  <!--   <el-form-item label="Output Comparer"> -->
  <!--     <el-button-group class="ml-4"> -->
  <!--       <el-button type="primary" :icon="Document" @click="paste" /> -->
  <!--       <el-button -->
  <!--         type="primary" -->
  <!--         :icon="CopyDocument" -->
  <!--         @click="copy(outputComparer)" -->
  <!--       /> -->
  <!--     </el-button-group> -->
  <!--     <el-input v-model="outputComparer" clearable /> -->
  <!--   </el-form-item> -->
  <!-- </el-form> -->
</template>
