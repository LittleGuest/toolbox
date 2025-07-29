<script setup>
import { onMounted, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Paste, Close } from "@vicons/carbon";

const message = useMessage();

onMounted(() => {
  api();
});

const input = ref("");
const datetime_utc8 = ref("");
const timestamp = ref();
const timestamp_mill = ref();

const api = async () => {
  const res = await invoke("timestamp").then((res) => {
    return res;
  }).catch((error) => message.error(error));

  input.value = res.timestamp;
  datetime_utc8.value = res.datetime_utc8;
  timestamp.value = res.timestamp;
  timestamp_mill.value = res.timestamp + res.timestamp_mill;
};

const change = (value) => { api(); };

const clear = () => {
  input.value = "";
  datetime_utc8.value = "";
  timestamp.value = "";
  timestamp_mill.value = "";
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <n-form label-placement="left">
    <n-form-item label="时间">
      <n-button @click="paste">
        <template #icon>
          <n-icon>
            <Paste />
          </n-icon>
        </template>
      </n-button>
      <n-input placeholder="" v-model:value="input" @update:value="change" @clear="clear" clearable />
    </n-form-item>

    <n-form-item label="UTC+8">
      <n-input placeholder="" readonly v-model:value="datetime_utc8" />
      <n-button @click="copy(datetime_utc8)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>

    <n-form-item label="时间戳（秒）">
      <n-input placeholder="" readonly v-model:value="timestamp" />
      <n-button @click="copy(timestamp)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>

    <n-form-item label="时间戳（毫秒）">
      <n-input placeholder="" readonly v-model:value="timestamp_mill" />
      <n-button @click="copy(timestamp_mill)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
  </n-form>
</template>
