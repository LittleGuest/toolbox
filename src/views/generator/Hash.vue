<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Paste, Close } from "@vicons/carbon";
import Checksum from "./Checksum.vue";

const message = useMessage();

const uppercase = ref(false);
const outputType = ref();
const hmacMode = ref(false);
const input = ref("");
const hash = ref({});

let debounceTimer = null;
let requestId = 0;

const api = async (id) => {
  return await invoke("hash", {
    uppercase: uppercase.value,
    outputType: outputType.value,
    hmacMode: hmacMode.value,
    input: input.value,
  }).then((res) => {
    if (id === requestId) {
      hash.value = res;
    }
    return res;
  }).catch((error) => message.error(error));
};

const change = async (value) => {
  if (debounceTimer) {
    clearTimeout(debounceTimer);
  }
  requestId++;
  const currentRequestId = requestId;
  debounceTimer = setTimeout(() => {
    api(currentRequestId);
  }, 300);
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};

const clear = () => {
  input.value = "";
  hash.value = {};
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <n-tabs type="line" animated>
        <n-tab-pane name="text" tab="文本">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
        <n-input placeholder="请输入" clearable @update:value="change" @clear="clear" v-model:value="input" :rows="5"
          type="textarea" />
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

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">结果</span>
        <div class="hash-list">
          <div class="hash-row">
            <span class="hash-name">MD5</span>
            <n-input placeholder="" readonly v-model:value="hash.md5" class="hash-value" />
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="copy(hash.md5)">
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
          <div class="hash-row">
            <span class="hash-name">SHA1</span>
            <n-input placeholder="" readonly v-model:value="hash.sha1" class="hash-value" />
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="copy(hash.sha1)">
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
          <div class="hash-row">
            <span class="hash-name">SHA256</span>
            <n-input placeholder="" readonly v-model:value="hash.sha256" class="hash-value" />
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="copy(hash.sha256)">
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
          <div class="hash-row">
            <span class="hash-name">SHA512</span>
            <n-input placeholder="" readonly v-model:value="hash.sha512" class="hash-value" />
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="copy(hash.sha512)">
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
          <div class="hash-row">
            <span class="hash-name">SHA3 256</span>
            <n-input placeholder="" readonly v-model:value="hash.sha3_256" class="hash-value" />
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="copy(hash.sha3_256)">
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
          <div class="hash-row">
            <span class="hash-name">SHA3 512</span>
            <n-input placeholder="" readonly v-model:value="hash.sha3_512" class="hash-value" />
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="copy(hash.sha3_512)">
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
      </div>
        </n-tab-pane>
        <n-tab-pane name="file" tab="文件校验">
          <Checksum />
        </n-tab-pane>
      </n-tabs>
    </section>
  </div>
</template>

<style scoped>
.hash-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.hash-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.hash-name {
  flex-shrink: 0;
  width: 88px;
  font-family: var(--tb-font-mono);
  font-size: 13px;
  color: var(--tb-text-2);
}

.hash-value {
  flex: 1;
  min-width: 0;
}
</style>
