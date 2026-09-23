<script setup lang="ts">
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import CodeMirror from "vue-codemirror6";
import { format } from "sql-formatter";
import { Copy, Paste, MagicWand } from "@vicons/carbon";

const dialect = ref("mysql");
const indent = ref(2);
const upper = ref("upper");
const sql = ref("");
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
const dialectOptions = [
  {
    label: 'MySQl',
    value: 'mysql'
  },
  {
    label: 'SQLite',
    value: 'sqlite',
  },
  {
    label: 'PostgreSQL',
    value: 'postgresql',
  }
];

const formatSql = () => {
  sql.value = format(sql.value, {
    language: 'sql',
    tabWidth: indent.value,
    keywordCase: upper.value,
  });
};

const paste = async () => {
  const clip = await readText();
  sql.value = clip;
};

const copy = async () => {
  await writeText(sql.value);
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
        <div class="tb-config-item">
          <span class="tb-config-label">关键字大写</span>
          <n-switch v-model:value="upper" checked-value="upper" unchecked-value="lower" />
        </div>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">SQL 输入</span>
        <code-mirror basic v-model="sql" class="code-mirror" />
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
              <n-button type="primary" @click="formatSql">
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
