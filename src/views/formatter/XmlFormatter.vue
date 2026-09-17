<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import CodeMirror from "vue-codemirror6";
import xmlFormat from "xml-formatter";
import { Copy, Paste, MagicWand } from "@vicons/carbon";

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
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">缩进</span>
          <n-select placeholder="请选择缩进字符" :options="indentOptions" v-model:value="indent"
            class="config-control" />
        </div>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">XML 输入</span>
        <code-mirror basic v-model="xml" class="code-mirror" />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="paste">
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
              <n-button @click="copy">
                <template #icon>
                  <n-icon>
                    <Copy />
                  </n-icon>
                </template>
              </n-button>
            </template>
            复制
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" @click="formatXml">
                <template #icon>
                  <n-icon>
                    <MagicWand />
                  </n-icon>
                </template>
              </n-button>
            </template>
            格式化
          </n-tooltip>
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.config-control {
  width: 140px;
}

.code-mirror {
  width: 100%;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  overflow: hidden;
  background: var(--tb-bg-app);
}
</style>
