<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";

import { notification } from "../../tool.js";

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
};

const ipv4api = async () => {
  const res = await invoke("ip_to_number", { t: "v4", ip: ipv4.value });
  console.log(res);
  binaryv4.value = res.binary;
  octalv4.value = res.octal;
  decimalv4.value = res.decimal;
  hexv4.value = res.hex;
};

const ipv6api = async () => {
  const res = await invoke("ip_to_number", { t: "v6", ip: ipv6.value });
  console.log(res);
  binaryv6.value = res.binary;
  octalv6.value = res.octal;
  decimalv6.value = res.decimal;
  hexv6.value = res.hex;
};

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
  notification("复制成功");
};
</script>

<template>
</template>
