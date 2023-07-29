<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText } from '@tauri-apps/api/clipboard';
import { CopyDocument } from "@element-plus/icons-vue";

const hyphens = ref();
const uppercase = ref(false);
const uuidVersion = ref(4);
const number = ref(5);
const uuids = ref("");


async function api() {
  const value = await invoke("uuid", { hyphens: hyphens.value, uppercase: uppercase.value, version: uuidVersion.value, number: number.value, });
  uuids.value = value.join().replaceAll(",", "\n");
}

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <!--
    <el-form-item label="Hyphens">
      <el-switch v-model="hyphens" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N"
       />
    </el-form-item>
    -->

    <el-form-item label="Uppercase">
      <el-switch v-model="uppercase" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N" />
    </el-form-item>

    <el-form-item label="UUID version">
      <el-select v-model="uuidVersion" class="m-2" size="large">
        <el-option key="1" label="1" value="1" />
        <el-option key="3" label="3" value="3" />
        <el-option key="4" label="4" value="4" />
        <el-option key="5" label="5" value="5" />
      </el-select>
    </el-form-item>

    <el-form-item label="Generate">
      <el-text class="mx-1" type="primary">Generate UUID(s) x </el-text>
      <el-input-number v-model="number" :min="1" :max="999999" controls-position="right" />
      <el-button @click="api">生成</el-button>
    </el-form-item>

    <el-form-item label="UUID(s)">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="CopyDocument" @click="copy(uuids)" />
      </el-button-group>
      <el-input v-model="uuids" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
