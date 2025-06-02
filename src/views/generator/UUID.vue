<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste, Close } from "@vicons/carbon";

const hyphens = ref();
const uppercase = ref(false);
const uuidVersion = ref(4);
const number = ref(5);
const uuids = ref("");
const versionOptions = [
  {
    label: "v1",
    value: 1
  },
  {
    label: "v3",
    value: 3
  },
  {
    label: "v4",
    value: 4
  },
  {
    label: "v5",
    value: 5
  },
  {
    label: "v6",
    value: 6
  },
  {
    label: "v7",
    value: 7
  },
  {
    label: "v8",
    value: 8
  },
];

const api = async () => {
  return await invoke("uuid", {
    hyphens: hyphens.value,
    uppercase: uppercase.value,
    version: uuidVersion.value,
    number: number.value,
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const generate = async () => {
  const data = await api();
  uuids.value = data.join().replaceAll(",", "\n");
};

const copy = () => {
  writeText(uuids.value);
};
</script>

<template>
  <n-form label-placement="left">
    <n-form-item label="大写">
      <n-switch v-model:value="uppercase" checked="Y" unchecked="N" />
    </n-form-item>
    <n-form-item label="UUID版本">
      <n-select placeholder="请选择版本" :options="versionOptions" v-model:value="uuidVersion" />
    </n-form-item>
    <n-form-item label="生成数量">
      <span>Generate UUID(s) x</span>
      <n-input-number placeholder="请输入生成数量" v-model:value="number" min="5" max="999999" />
      <n-button @click="generate">生成</n-button>
    </n-form-item>

    <n-form-item label="操作">
      <n-button @click="copy">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="UUID(S)">
      <n-input placeholder="" v-model:value="uuids" :rows="10" type="textarea" />
    </n-form-item>
  </n-form>
</template>
