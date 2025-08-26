<script setup>
import { onMounted, ref, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Paste } from "@vicons/carbon";

const message = useMessage();

// 当前时间
const current = ref("");
const input = ref("");
const format = ref("");

// 定时器引用
let updateTimer = null;
// 获取并格式化当前时间
const updateCurrentTime = () => {
  const now = new Date();
  // 获取当前时间的秒级时间戳
  current.value = Math.floor(now.getTime() / 1000).toString();
};

const api = async (time) => {
  const res = await invoke("timestamp", { time }).then((res) => {
    return res;
  }).catch((error) => message.error(error));

  format.value = res.format;
};

const change = (value) => { api(value); };
watch(input, (newValue, oldValue) => {
  if (newValue !== oldValue) {
    api(newValue);
  }
});

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

onMounted(() => {
  // 初始化当前时间
  updateCurrentTime();
  // 设置定时器，每秒更新一次当前时间
  updateTimer = setInterval(updateCurrentTime, 1000);
});

// 组件卸载时清除定时器
onUnmounted(() => {
  if (updateTimer) {
    clearInterval(updateTimer);
  }
});
</script>

<template>
  <n-form label-placement="left" label-width="120">
    <n-form-item label="当前时间">
      <n-input placeholder="" v-model:value="current" readonly />
      <n-button @click="copy(current)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>

    <n-form-item label="时间戳">
      <n-input placeholder="" v-model:value="input" @update:value="change" @clear="clear" clearable />
      <n-button @click="paste">
        <template #icon>
          <n-icon>
            <Paste />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="时间">
      <n-input placeholder="" readonly v-model:value="format" />
      <n-button @click="copy(format)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <!--

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
    -->
  </n-form>
</template>
