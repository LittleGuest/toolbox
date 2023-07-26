<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const uppercase = ref();
const outputType = ref();
const hmacMode = ref();
const input = ref("");
const md5 = ref("");
const sha1 = ref("");
const sha256 = ref("");
const sha512 = ref("");

const paste = (value) => {
  console.log("paste", value);
};

const copy = (value) => {
  console.log("copy", value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px" style="max-width: 460px">
    <el-form-item label="Uppercase">
      <el-switch
        v-model="uppercase"
        class="ml-2"
        inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
        active-text="Y"
        inactive-text="N"
        active-value="1"
        inactive-value="0"
      />
    </el-form-item>

    <el-form-item label="Output Type">
      <el-select v-model="outputType" class="m-2" size="large">
        <el-option key="Hex" label="Hex" value="Hex" />
        <el-option key="Hex" label="Hex" value="Hex" />
      </el-select>
    </el-form-item>

    <el-form-item label="HMAC Mode">
      <el-switch
        v-model="hmacMode"
        class="ml-2"
        inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
        active-text="Y"
        inactive-text="N"
        active-value="1"
        inactive-value="0"
      />
    </el-form-item>

    <el-form-item label="Input">
      <el-button-group class="ml-4">
        <el-button type="primary" :icon="Document" v-on:click="paste(input)" />
        <el-button
          type="primary"
          :icon="CopyDocument"
          v-on:click="copy(input)"
        />
      </el-button-group>
      <el-input v-model="input" :rows="10" type="textarea" />
    </el-form-item>

    <el-form-item label="MD5">
      <el-text class="mx-1">{{ md5 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" v-on:click="copy(md5)" />
    </el-form-item>

    <el-form-item label="SHA1">
      <el-text class="mx-1">{{ sha1 }}</el-text>
      <el-button type="primary" :icon="CopyDocument" v-on:click="copy(sha1)" />
    </el-form-item>

    <el-form-item label="SHA256">
      <el-text class="mx-1">{{ sha256 }}</el-text>
      <el-button
        type="primary"
        :icon="CopyDocument"
        v-on:click="copy(sha256)"
      />
    </el-form-item>

    <el-form-item label="SHA512">
      <el-text class="mx-1">{{ sha512 }}</el-text>
      <el-button
        type="primary"
        :icon="CopyDocument"
        v-on:click="copy(sha512)"
      />
    </el-form-item>
  </el-form>
</template>
