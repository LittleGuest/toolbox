<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument, ArrowUpBold, ArrowDownBold, Close } from "@element-plus/icons-vue";

const ident = ref(4);
const ft = ref("json");
const tt = ref("json");
const input = ref();
const output = ref();

async function cffcApi(ft, tt, value) {
  return await invoke("cffc", { ident: ident.value, ft: ft, tt: tt, input: value });
}

const itt = async () => {
  output.value = await cffcApi(ft.value, tt.value, input.value);
};

const tti = async () => {
  input.value = await cffcApi(tt.value, ft.value, output.value);
};

const pasteInput = async () => {
  input.value = await readText();
};

const pasteOutput = async () => {
  output.value = await readText();
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
    <el-form-item label="缩进">
      <el-select v-model="ident" class="m-2" size="large">
        <el-option key="2" label="2" value="2" />
        <el-option key="4" label="4" value="4" />
      </el-select>
    </el-form-item>

    <el-form-item label="输入">
      <el-select v-model="ft" class="m-2" size="large">
        <el-option key="json" label="Json" value="json" />
        <el-option key="toml" label="Toml" value="toml" />
        <el-option key="yaml" label="Yaml" value="yaml" />
      </el-select>
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="pasteInput" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
        <el-button type="primary" :icon="Close" @click="clear" />
      </el-button-group>
      <el-input v-model="input" :rows="10" type="textarea" />
    </el-form-item>
    <el-form-item label="转换">
      <el-button :icon="ArrowDownBold" @click="itt" />
      <el-button :icon="ArrowUpBold" @click="tti" />
    </el-form-item>


    <el-form-item label="输出">
      <el-select v-model="tt" class="m-2" size="large">
        <el-option key="json" label="Json" value="json" />
        <el-option key="toml" label="Toml" value="toml" />
        <el-option key="yaml" label="Yaml" value="yaml" />
      </el-select>
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="pasteOutput" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(output)" />
        <el-button type="primary" :icon="Close" @click="clear" />
      </el-button-group>
      <el-input v-model="output" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
