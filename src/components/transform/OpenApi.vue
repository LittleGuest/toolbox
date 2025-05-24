<script setup>
import { ref, computed } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { invoke } from "@tauri-apps/api/core";
import { createDiscreteApi } from "naive-ui";
import { Copy, Paste, Close, Download } from "@vicons/carbon";
import { Vue3JsonEditor } from "vue3-json-editor";

const { message, loadingBar } = createDiscreteApi(["message", "loadingBar",]);

const url = ref("http://127.0.0.1:8080/xxxxxxxx/v3/api-docs");
const typeOptions = [
  {
    label: "DOCX",
    value: "docx"
  },
  {
    label: "PDF",
    value: "pdf"
  },
  {
    label: "Markdown",
    value: "markdown"
  },
  {
    label: "HTML",
    value: "html"
  }
];
const outputType = ref();
const apiData = ref();

const fetchApiData = async () => {
  await invoke("fetch_api_data", {
    url: url.value
  }).then((res) => {
    apiData.value = JSON.parse(res);
  }).catch((error) => message.error(error));
};

const download = async () => {
  console.log(outputType.value, url.value);
  await invoke("download", {
    url: url.value,
    outputType: outputType.value
  }).then((res) => {
    message.info(res);
  }).catch((error) => message.error(error));
};

const paste = async () => {
  const clip = await readText();
  apiData.value = JSON.parse(clip);
};

const copy = () => {
  writeText(apiData.value);
};

const clear = () => {
  apiData.value = "";
};
</script>

<template>
  <n-form label-placement="left" label-width="auto">
    <n-form-item label="OpenAPI接口">
      <n-input placeholder="请输入OpenAPI接口" v-model:value="url" />
      <n-button @click="fetchApiData">获取</n-button>
    </n-form-item>
    <n-form-item label="输出类型">
      <n-select placeholder="请选择文件类型" :options="typeOptions" v-model:value="outputType" />
      <n-button @click="download">
        <template #icon>
          <n-icon>
            <Download />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
  </n-form>

  <n-button-group>
    <n-button @click="paste">
      <template #icon>
        <n-icon>
          <Paste />
        </n-icon>
      </template>
    </n-button>
    <n-button @click="copy">
      <template #icon>
        <n-icon>
          <Copy />
        </n-icon>
      </template>
    </n-button>
    <n-button @click="clear">
      <template #icon>
        <n-icon>
          <Close />
        </n-icon>
      </template>
    </n-button>
  </n-button-group>

  <Vue3JsonEditor v-model="apiData" :show-btns="false" :expandedOnStart="true" />
</template>
