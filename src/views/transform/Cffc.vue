<script setup lang="ts">
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
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">缩进</span>
          <n-select placeholder="请缩进字符" :options="indentOptions" v-model:value="indent" style="width: 120px" />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">输入文件类型</span>
          <n-select placeholder="请选择文件类型" :options="typeOptions" v-model:value="ft" style="width: 150px" />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">输出文件类型</span>
          <n-select placeholder="请选择文件类型" :options="typeOptions" v-model:value="tt" style="width: 150px" />
        </div>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输入</span>
        <n-input placeholder="" v-model:value="input" :rows="8" type="textarea" />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="pasteInput">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="copy(input)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="clear">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
      </div>

      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="itt">
              <template #icon><n-icon><ArrowDown /></n-icon></template>
            </n-button>
          </template>
          转换
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="tti">
              <template #icon><n-icon><ArrowUp /></n-icon></template>
            </n-button>
          </template>
          反向转换
        </n-tooltip>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输出</span>
        <n-input placeholder="" v-model:value="output" :rows="8" type="textarea" />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="pasteOutput()">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="copy(output)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary @click="clear">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
      </div>
    </section>
  </div>
</template>
