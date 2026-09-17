<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, MagicWand } from "@vicons/carbon";

const message = useMessage();

const hyphens = ref();
const uppercase = ref(false);
const removeConnector = ref(false);
const uuidVersion = ref(4);
const number = ref(5);
const uuids = ref("");
const versionOptions = [
  {
    label: "v1",
    value: 1
  },
  {
    label: "v3",
    value: 3
  },
  {
    label: "v4",
    value: 4
  },
  {
    label: "v5",
    value: 5
  },
  {
    label: "v6",
    value: 6
  },
  {
    label: "v7",
    value: 7
  },
  {
    label: "v8",
    value: 8
  },
];

const api = async () => {
  return await invoke("uuid", {
    hyphens: hyphens.value,
    uppercase: uppercase.value,
    removeConnector: removeConnector.value,
    version: uuidVersion.value,
    number: number.value,
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const generate = async () => {
  const data = await api();
  uuids.value = data.join().replaceAll(",", "\n");
};

const copy = () => {
  writeText(uuids.value);
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">大写</span>
          <n-switch v-model:value="uppercase" checked="Y" unchecked="N" />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">去掉连接符</span>
          <n-switch v-model:value="removeConnector" checked="Y" unchecked="N" />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">UUID版本</span>
          <n-select placeholder="请选择版本" :options="versionOptions" v-model:value="uuidVersion"
            class="config-control" />
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">生成数量</span>
          <n-input-number placeholder="请输入生成数量" v-model:value="number" min="5" max="999999"
            class="config-number" />
        </div>
      </div>

      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="generate">
              <template #icon>
                <n-icon>
                  <MagicWand />
                </n-icon>
              </template>
            </n-button>
          </template>
          生成
        </n-tooltip>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">UUID 列表</span>
        <n-input placeholder="" v-model:value="uuids" :rows="10" type="textarea" />
        <div class="tb-toolbar">
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
        </div>
      </div>
    </section>
  </div>
</template>

<style scoped>
.config-control {
  width: 140px;
}

.config-number {
  width: 140px;
}
</style>
