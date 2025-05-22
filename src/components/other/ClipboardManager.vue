<script setup>
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste, Close } from "@vicons/carbon";

const clipboarHistory = ref([]);

const copy = (value) => {
  writeText(value);
};

const paste = async () => {
  const clip = await readText();
  input.value = JSON.parse(clip);
};

</script>

<template>
  <n-list>
    <n-list-item v-for="history in clipboarHistory">
      <template #suffix>
        <n-button @click="copy(input)">
          <template #icon>
            <n-icon>
              <Copy />
            </n-icon>
          </template>
        </n-button>
      </template>
      <n-thing title="Thing" description="description">
        Biu<br>
        Biu<br>
        Biu<br>
      </n-thing>
    </n-list-item>
  </n-list>
</template>
