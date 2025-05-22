<script setup>
import { ref, computed } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { invoke } from "@tauri-apps/api/core";
import { createDiscreteApi } from "naive-ui";
import { Copy, Paste, Close, Download } from "@vicons/carbon";
import { Vue3JsonEditor } from "vue3-json-editor";

const { message, loadingBar } = createDiscreteApi(["message", "loadingBar",]);

const host = ref("http://127.0.0.1:8080");
const name = ref("");
const uri = ref("/v3/api-docs");
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


const fetchApiDataApi = async (url) => {
  return await invoke("fetch_api_config", {
    url,
  });
};

const fetchApiData = async () => {
  if (!name.value.startsWith("/")) {
    name.value = "/" + name.value;
  }
  if (!uri.value.startsWith("/")) {
    uri.value = "/" + uri.value;
  }

  const res = await fetchApiDataApi(host.value + name.value + uri.value);
  apiData.value = JSON.parse(res);
};

const download = () => {

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
      <n-input placeholder="" v-model:value="host" />
      <n-input placeholder="请输入服务名" v-model:value="name" />
      <n-input placeholder="" v-model:value="uri" />
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
