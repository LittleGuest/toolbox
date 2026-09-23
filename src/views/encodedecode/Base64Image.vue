<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open, save } from "@tauri-apps/plugin-dialog";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Paste, Close, Download, Upload } from "@vicons/carbon";

const message = useMessage();

const filePath = ref("");
const encodeResult = ref(null); // { mime, size, dataUrl }
const encodeLoading = ref(false);

const handleSelectFile = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "图片",
          extensions: ["png", "jpg", "jpeg", "gif", "webp", "bmp", "svg", "ico", "avif"],
        },
        { name: "所有文件", extensions: ["*"] },
      ],
    });
    if (selected) {
      filePath.value = selected;
      await encode();
    }
  } catch (error) {
    message.error(String(error));
  }
};

const encode = async () => {
  if (!filePath.value) {
    message.warning("请先选择图片文件");
    return;
  }
  encodeLoading.value = true;
  try {
    encodeResult.value = await invoke("encode_base64_image", {
      filePath: filePath.value,
    });
  } catch (error) {
    encodeResult.value = null;
    message.error(String(error));
  } finally {
    encodeLoading.value = false;
  }
};

const clearEncode = () => {
  filePath.value = "";
  encodeResult.value = null;
};

const decodeInput = ref("");
const decodeResult = ref(null); // { mime, size, dataUrl }
const decodeLoading = ref(false);

const decode = async () => {
  if (!decodeInput.value.trim()) {
    message.warning("请输入 Base64 或 Data URL");
    return;
  }
  decodeLoading.value = true;
  try {
    decodeResult.value = await invoke("decode_base64_image", {
      input: decodeInput.value,
    });
  } catch (error) {
    decodeResult.value = null;
    message.error(String(error));
  } finally {
    decodeLoading.value = false;
  }
};

const saveImage = async () => {
  if (!decodeResult.value) {
    message.warning("请先解码");
    return;
  }
  try {
    const ext = (decodeResult.value.mime || "image/png").split("/")[1].replace("x-", "");
    const path = await save({
      defaultPath: `image-${Date.now()}.${ext}`,
      filters: [
        {
          name: "图片",
          extensions: [ext === "jpeg" ? "jpg" : ext],
        },
      ],
    });
    if (path) {
      await invoke("save_base64_image", {
        input: decodeInput.value,
        filePath: path,
      });
      message.success("图片已保存");
    }
  } catch (error) {
    message.error(String(error));
  }
};

const pasteDecode = async () => {
  decodeInput.value = await readText();
  await decode();
};

const clearDecode = () => {
  decodeInput.value = "";
  decodeResult.value = null;
};

const copy = (value) => {
  if (!value) return;
  writeText(value);
  message.success("复制成功");
};

const formatSize = (bytes) => {
  const n = Number(bytes) || 0;
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(2)} KB`;
  return `${(n / 1024 / 1024).toFixed(2)} MB`;
};
</script>

<template>
  <div>
      <div class="tb-editor-grid">
        <div class="tb-editor">
          <span class="tb-editor-label">图片 → Base64</span>
          <div class="tb-toolbar">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="success" :loading="encodeLoading" @click="handleSelectFile">
                  <template #icon><n-icon><Upload /></n-icon></template>
                </n-button>
              </template>
              选择图片
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button quaternary @click="clearEncode">
                  <template #icon><n-icon><Close /></n-icon></template>
                </n-button>
              </template>
              清除
            </n-tooltip>
            <span class="file-name">{{ filePath ? filePath.split(/[\\/]/).pop() : "未选择" }}</span>
          </div>

          <template v-if="encodeResult">
            <div class="preview">
              <img :src="encodeResult.dataUrl" alt="预览" />
            </div>
            <div class="info-tags">
              <n-tag type="info">{{ encodeResult.mime }}</n-tag>
              <n-tag>{{ formatSize(encodeResult.size) }}</n-tag>
            </div>
            <n-input
              :value="encodeResult.dataUrl"
              type="textarea"
              :rows="8"
              readonly
              class="tb-mono"
              placeholder="正在生成…"
            />
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="copy(encodeResult.dataUrl)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制 Data URL
              </n-tooltip>
            </div>
          </template>
        </div>

        <div class="tb-editor tb-mono">
          <span class="tb-editor-label">Base64 → 图片</span>
          <n-input
            v-model:value="decodeInput"
            type="textarea"
            :autosize="{ minRows: 5, maxRows: 12 }"
            placeholder="粘贴 Base64 字符串或 Data URL（data:image/png;base64,...）"
          />
          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" :loading="decodeLoading" @click="decode">
                  <template #icon><n-icon><Upload /></n-icon></template>
                </n-button>
              </template>
              解码
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="pasteDecode">
                  <template #icon><n-icon><Paste /></n-icon></template>
                </n-button>
              </template>
              粘贴
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="clearDecode">
                  <template #icon><n-icon><Close /></n-icon></template>
                </n-button>
              </template>
              清除
            </n-tooltip>
          </div>

          <template v-if="decodeResult">
            <div class="preview">
              <img :src="decodeResult.dataUrl" alt="解码预览" />
            </div>
            <div class="info-tags">
              <n-tag type="info">{{ decodeResult.mime }}</n-tag>
              <n-tag>{{ formatSize(decodeResult.size) }}</n-tag>
            </div>
            <div class="tb-toolbar">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="saveImage">
                    <template #icon><n-icon><Download /></n-icon></template>
                  </n-button>
                </template>
                保存图片
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="copy(decodeResult.dataUrl)">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制 Data URL
              </n-tooltip>
            </div>
          </template>
        </div>
      </div>
  </div>
</template>

<style scoped>
.file-name {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 200px;
  color: var(--tb-text-3);
  font-size: 13px;
}

.info-tags {
  display: flex;
  gap: 8px;
  margin: 12px 0;
}

.preview {
  width: 100%;
  min-height: 120px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 1px dashed var(--tb-border);
  border-radius: var(--tb-radius-m);
  padding: 8px;
  background: repeating-conic-gradient(var(--tb-bg-app) 0 25%, var(--tb-bg-elevated) 0 50%) 0 0 / 16px 16px;
}

.preview img {
  max-width: 100%;
  max-height: 200px;
  object-fit: contain;
}
</style>
