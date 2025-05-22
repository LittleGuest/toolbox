<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste, Close } from "@vicons/carbon";

const uppercase = ref(false);
const outputType = ref();
const hmacMode = ref(false);
const input = ref("");
const hash = ref({});

const api = async () => {
  hash.value = await invoke("hash", {
    uppercase: uppercase.value,
    outputType: outputType.value,
    hmacMode: hmacMode.value,
    input: input.value,
  });
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
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-form-item label="是否开启大写">
      <el-switch v-model="uppercase" @change="change" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N" />
    </el-form-item>

    <el-form-item label="输入">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" @click="paste" />
        <el-button type="primary" :icon="CopyDocument" @click="copy(input)" />
      </el-button-group>
      <el-input v-model="input" autofocus @input="change" :rows="10" type="textarea" />
    </el-form-item>

    <el-form-item label="MD5">
      <el-text class="mx-1">{{ hash.md5 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hash.md5)" />
    </el-form-item>

    <el-form-item label="SHA1">
      <el-text class="mx-1">{{ hash.sha1 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hash.sha1)" />
    </el-form-item>

    <el-form-item label="SHA256">
      <el-text class="mx-1">{{ hash.sha256 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hash.sha256)" />
    </el-form-item>

    <el-form-item label="SHA512">
      <el-text class="mx-1">{{ hash.sha512 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hash.sha512)" />
    </el-form-item>

    <el-form-item label="SHA3 256">
      <el-text class="mx-1">{{ hash.sha3_256 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hash.sha3_256)" />
    </el-form-item>

    <el-form-item label="SHA3 512">
      <el-text class="mx-1">{{ hash.sha3_512 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" @click="copy(hash.sha3_512)" />
    </el-form-item>
  </el-form>
</template>
