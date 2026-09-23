<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useMessage, useLoadingBar } from "naive-ui";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste, Folder, Calculator } from "@vicons/carbon";

const message = useMessage();
const loadingBar = useLoadingBar();

const checksum = ref("");
const validValue = ref("");
const checksumAlgorithm = ref("md5sum");
const checksumOptions = ref([
  {
    label: "md5sum",
    value: "md5sum",
  },
  {
    label: "sha1sum",
    value: "sha1sum",
  },
  {
    label: "sha224sum",
    value: "sha2_224sum",
  },
  {
    label: "sha256sum",
    value: "sha2_256sum",
  },
  {
    label: "sha384sum",
    value: "sha2_384sum",
  },
  {
    label: "sha512sum",
    value: "sha2_512sum",
  },
  {
    label: "sha3_256sum",
    value: "sha3_256sum",
  },
  {
    label: "sha3_384sum",
    value: "sha3_384sum",
  },
  {
    label: "sha3_512sum",
    value: "sha3_512sum",
  },
]);

const api = async (type, filePath) => {
  return await invoke("checksum", { type, filePath })
    .then((res) => {
      return res;
    })
    .catch((error) => message.error(error));
};

const filePath = ref("");
const handleUpload = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "All Files",
          extensions: ["*"],
        },
      ],
    });

    if (selected) {
      filePath.value = selected;
    }
  } catch (error) {
    console.error("文件选择错误:", error);
    loadingBar.error();
  }
};

const upload = async () => {
  checksum.value = "";
  loadingBar.start();

  checksum.value = await api(checksumAlgorithm.value, filePath.value);
  loadingBar.finish();
};

const checksumMatched = computed(() => {
  return checksum.value === validValue.value;
});

const copy = (value) => {
  if (!value) {
    return;
  }
  writeText(value);
  message.success("复制成功");
};
const paste = async () => {
  validValue.value = await readText();
};
</script>

<template>
  <div>
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">文件</span>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="large" type="success" @click="handleUpload">
                <template #icon>
                  <n-icon>
                    <Folder />
                  </n-icon>
                </template>
              </n-button>
            </template>
            选择
          </n-tooltip>
          <span class="file-path">{{ filePath }}</span>
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">校验算法</span>
          <n-select v-model:value="checksumAlgorithm" :options="checksumOptions" class="config-control" />
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="success" @click="upload">
                <template #icon>
                  <n-icon>
                    <Calculator />
                  </n-icon>
                </template>
              </n-button>
            </template>
            计算
          </n-tooltip>
        </div>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">校验值</span>
        <n-input
          placeholder=""
          type="textarea"
          :autosize="{
            minRows: 3,
          }"
          readonly
          v-model:value="checksum"
        />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="copy(checksum)">
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
        <span class="tb-editor-label">对比值</span>
        <n-input
          placeholder=""
          type="textarea"
          :autosize="{
            minRows: 3,
          }"
          readonly
          v-model:value="validValue"
        />
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
              <n-button @click="copy(validValue)">
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

      <div class="tb-action-row" v-if="validValue">
        <span :class="checksumMatched ? 'matched' : 'notMatched'">{{
          checksumMatched ? "一致" : "不一致"
        }}</span>
      </div>
  </div>
</template>

<style scoped>
.config-control {
  width: 160px;
}

.file-path {
  font-family: var(--tb-font-mono);
  font-size: 13px;
  color: var(--tb-text-2);
  word-break: break-all;
}

.matched {
  font-size: 24px;
  color: #18a058;
}

.notMatched {
  font-size: 24px;
  color: #d03050;
}
</style>
