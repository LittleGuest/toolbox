<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";

const message = useMessage();

const indent = ref(4);
const ft = ref("json");
const tt = ref("json");
const input = ref();
const output = ref();
const indentOptions = [
  {
    label: 2,
    value: 2
  },
  {
    label: 4,
    value: 4,
  }
];
const typeOptions = [
  {
    label: "JSON",
    value: "json"
  },
  {
    label: "YAML",
    value: "yaml"
  },
  {
    label: "TOML",
    value: "toml"
  }
];

const api = async (ft, tt, value) => {
  return await invoke("cffc", {
    indent: indent.value,
    ft: ft,
    tt: tt,
    input: value,
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const itt = async () => {
  output.value = await api(ft.value, tt.value, input.value);
};

const tti = async () => {
  input.value = await api(tt.value, ft.value, output.value);
};

const pasteInput = async () => {
  input.value = await readText();
};

const pasteOutput = async () => {
  output.value = await readText();
};

const copy = (value) => {
  writeText(value);
};

const clear = () => {
  input.value = "";
  output.value = "";
};
</script>

<template>
  <n-form label-placement="left">
    <n-form-item label="缩进">
      <n-select placeholder="请缩进字符" :options="indentOptions" v-model:value="indent" />
    </n-form-item>
    <n-form-item label="输入文件类型">
      <n-select placeholder="请选择文件类型" :options="typeOptions" v-model:value="ft" />
    </n-form-item>
    <n-form-item label="操作">
      <n-button-group>
        <n-button @click="pasteInput">
          <template #icon>
            <n-icon>
              <Paste />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="copy(input)">
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
    </n-form-item>
    <n-form-item label="输入">
      <n-input placeholder="" v-model:value="input" :rows="10" type="textarea" />
    </n-form-item>
    <n-form-item label="转换">
      <n-button @click="itt">
        <template #icon>
          <n-icon>
            <ArrowDown />
          </n-icon>
        </template>
      </n-button>
      <n-button @click="tti">
        <template #icon>
          <n-icon>
            <ArrowUp />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="输出文件类型">
      <n-select placeholder="请选择文件类型" :options="typeOptions" v-model:value="tt" />
    </n-form-item>
    <n-form-item label="操作">
      <n-button-group>
        <n-button @click="pasteOutput()">
          <template #icon>
            <n-icon>
              <Paste />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="copy(output)">
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
    </n-form-item>
    <n-form-item label="输出">
      <n-input placeholder="" v-model:value="output" :rows="10" type="textarea" />
    </n-form-item>
  </n-form>
</template>
