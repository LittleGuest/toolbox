<script setup lang="ts">
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
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">JSON</span>
        <VueJsonPretty :data="input" showLength showLineNumber showIcon showSelectController editable
          class="json-viewer" />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="paste(input)">
                <template #icon>
                  <n-icon>
                    <Paste />
                  </n-icon>
                </template>
              </n-button>
            </template>
            粘贴
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="copy(input)">
                <template #icon>
                  <n-icon>
                    <Copy />
                  </n-icon>
                </template>
              </n-button>
            </template>
            复制
          </n-tooltip>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.json-viewer {
  width: 100%;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  padding: 12px;
  background: var(--tb-bg-app);
  font-family: var(--tb-font-mono);
}
</style>
