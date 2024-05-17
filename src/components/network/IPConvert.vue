<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument } from "@element-plus/icons-vue";

import { notification } from '../../tool.js';

const ipv4 = ref("");
const binaryv4 = ref("");
const octalv4 = ref("");
const decimalv4 = ref("");
const hexv4 = ref("");

const ipv6 = ref("");
const binaryv6 = ref("");
const octalv6 = ref("");
const decimalv6 = ref("");
const hexv6 = ref("");

const checkIp = async (t, v) => {
  return await invoke("check_ip", { t: t, ip: v });
}

const ipv4api = async () => {
  const res = await invoke("ip_to_number", { t: "v4", ip: ipv4.value });
  console.log(res);
  binaryv4.value = res.binary;
  octalv4.value = res.octal;
  decimalv4.value = res.decimal;
  hexv4.value = res.hex;
}

const ipv6api = async () => {
  const res = await invoke("ip_to_number", { t: "v6", ip: ipv6.value });
  console.log(res);
  binaryv6.value = res.binary;
  octalv6.value = res.octal;
  decimalv6.value = res.decimal;
  hexv6.value = res.hex;
}

const v4change = async (value) => {
  // const res = await checkIp("v4", ipv4.value);
  ipv4api();
};

const v6change = async (value) => {
  // const res = await checkIp("v6", ipv6.value);
  ipv6api();
};

const pastev4 = async () => {
  ipv4.value = await readText();
};

const pastev6 = async () => {
  ipv6.value = await readText();
};

const copy = (value) => {
  writeText(value);
  notification('复制成功');
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-tabs type="card">
      <el-tab-pane label="IPV4">
        <el-form-item label="输入">
          <el-button-group class="ml-4">
            <el-button type="primary" :icon="Document" @click="pastev4" />
            <el-button type="primary" :icon="CopyDocument" @click="copy(ipv4)" />
          </el-button-group>
          <el-input v-model="ipv4" @input="v4change" maxlength=19 />
        </el-form-item>

        <el-form-item label="二进制">
          <el-text class="mx-1">{{ binaryv4 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(binaryv4)" />
        </el-form-item>

        <el-form-item label="八进制">
          <el-text class="mx-1">{{ octalv4 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(octalv4)" />
        </el-form-item>

        <el-form-item label="十进制">
          <el-text class="mx-1">{{ decimalv4 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(decimalv4)" />
        </el-form-item>

        <el-form-item label="十六进制">
          <el-text class="mx-1">{{ hexv4 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(hexv4)" />
        </el-form-item>
      </el-tab-pane>

      <el-tab-pane label="IPV6">
        <el-form-item label="输入">
          <el-button-group class="ml-4">
            <el-button type="primary" :icon="Document" @click="pastev6" />
            <el-button type="primary" :icon="CopyDocument" @click="copy(ipv6)" />
          </el-button-group>
          <el-input v-model="ipv6" @input="v6change" maxlength=19 />
        </el-form-item>

        <el-form-item label="二进制">
          <el-text class="mx-1">{{ binaryv6 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(binaryv6)" />
        </el-form-item>

        <el-form-item label="八进制">
          <el-text class="mx-1">{{ octalv6 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(octalv6)" />
        </el-form-item>

        <el-form-item label="十进制">
          <el-text class="mx-1">{{ decimalv6 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(decimalv6)" />
        </el-form-item>

        <el-form-item label="十六进制">
          <el-text class="mx-1">{{ hexv6 }}</el-text>
          <el-button type="primary" :icon="CopyDocument" @click="copy(hexv6)" />
        </el-form-item>
      </el-tab-pane>
    </el-tabs>

  </el-form>
</template>
