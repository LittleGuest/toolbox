<script setup>
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste } from "@vicons/carbon";
import VueJsonPretty from 'vue-json-pretty';
import 'vue-json-pretty/lib/styles.css';

const input = ref({});

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

  <VueJsonPretty :data="input" showLength showLineNumber showIcon showSelectController editable />
</template>
