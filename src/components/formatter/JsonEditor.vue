<script setup>
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste } from "@vicons/carbon";
import { Vue3JsonEditor } from "vue3-json-editor";

const input = ref();

const change = (value) => {
  input.value = value;
};

const paste = async () => {
  const clip = await readText();
  input.value = JSON.parse(clip);
};

const copy = (value) => {
  writeText(JSON.stringify(value));
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

  <Vue3JsonEditor v-model="input" :show-btns="false" :expandedOnStart="true" @json-change="change" />
</template>
