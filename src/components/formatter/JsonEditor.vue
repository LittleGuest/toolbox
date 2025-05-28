<script setup>
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste } from "@vicons/carbon";
import JsonEditorVue from 'json-editor-vue';

const input = ref();

const paste = async () => {
  const clip = await readText();
  input.value = JSON.parse(clip);
};

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <n-button-group>
    <n-button @click="paste(input)">
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
  </n-button-group>

  <JsonEditorVue v-model="input" />
</template>

<style scoped>
:v-deep .jsoneditor-menu {
  background-color: rgb(255, 251, 240) !important;
}
</style>
