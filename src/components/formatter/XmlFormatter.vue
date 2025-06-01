<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import CodeMirror from "vue-codemirror6";
import xmlFormat from "xml-formatter";
import { Copy, Paste } from "@vicons/carbon";

const indent = ref('    ');
const indentOptions = [
  {
    label: 2,
    value: '  '
  },
  {
    label: 4,
    value: '    ',
  }
];
const xml = ref("");

const formatXml = () => {
  xml.value = xmlFormat(xml.value, {
    indentation: indent.value,
    collapseContent: true,
    lineSeparator: "\n",
  });
};

const paste = async () => {
  const clip = await readText();
  xml.value = clip;
};

const copy = async () => {
  await writeText(xml.value);
};
</script>

<template>
  <n-form label-placement="left">
    <!-- <n-form-item label="方言"> -->
    <!--   <n-select placeholder="请选择方言" :options="dialectOptions" v-model:value="dialect" /> -->
    <!-- </n-form-item> -->
    <n-form-item label="缩进">
      <n-select placeholder="请选择缩进字符" :options="indentOptions" v-model:value="indent" />
    </n-form-item>
    <!-- <n-form-item label="关键字大写"> -->
    <!--   <n-switch v-model:value="upper" checked-value="upper" unchecked-value="lower" /> -->
    <!-- </n-form-item> -->
    <n-form-item label="">
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
        <n-button @click="formatXml">格式化</n-button>
      </n-button-group>
    </n-form-item>

    <code-mirror basic v-model="xml" />
  </n-form>
</template>
