<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useMessage } from "naive-ui";
import { Upload, Close, FolderOpen, Add } from "@vicons/carbon";

const message = useMessage();

const targetFormats = [
  "png", "jpg", "jpeg", "webp", "bmp", "gif",
  "tiff", "tif", "tga", "ppm", "pgm", "pnm",
  "hdr", "exr", "avif", "qoi", "ff",
];

const files = ref<string[]>([]);
const outputFormat = ref("png");
const outputDir = ref("");
const outputs = ref<string[]>([]);
const loading = ref(false);

const fileNames = computed(() =>
  files.value.map((f) => f.split(/[\\/]/).pop() || f)
);

const pickFiles = async () => {
  try {
    const selected = await open({
      multiple: true,
      filters: [
        {
          name: "图片",
          extensions: ["png", "jpg", "jpeg", "webp", "bmp", "gif", "tiff", "tif", "tga", "ppm", "pgm", "pnm", "hdr", "exr", "avif", "qoi", "ff"],
        },
        { name: "所有文件", extensions: ["*"] },
      ],
    });
    if (selected) {
      files.value = (Array.isArray(selected) ? selected : [selected]).filter(
        (p): p is string => typeof p === "string"
      );
      if (!outputDir.value && files.value.length) {
        const idx = files.value[0].lastIndexOf("/");
        const idx2 = files.value[0].lastIndexOf("\\");
        const end = Math.max(idx, idx2);
        if (end > 0) {
          outputDir.value = files.value[0].slice(0, end);
        }
      }
    }
  } catch (error) {
    message.error(String(error));
  }
};

const pickDir = async () => {
  try {
    const selected = (await open({
      directory: true,
      multiple: false,
    })) as string | null;
    if (selected) outputDir.value = selected;
  } catch (error) {
    message.error(String(error));
  }
};

const convert = async () => {
  if (!files.value.length) {
    message.warning("请先选择图片文件");
    return;
  }
  if (!outputDir.value) {
    message.warning("请选择输出目录");
    return;
  }
  loading.value = true;
  try {
    outputs.value = await invoke("image_convert", {
      inputs: files.value,
      outputFormat: outputFormat.value,
      outputDir: outputDir.value,
    }) as string[];
    message.success(`转换完成，共 ${outputs.value.length} 个文件`);
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
};

const clear = () => {
  files.value = [];
  outputs.value = [];
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">目标格式</span>
          <n-select
            v-model:value="outputFormat"
            :options="targetFormats.map((f) => ({ label: f, value: f }))"
            style="width: 140px"
            placeholder="选择输出格式"
          />
        </div>
      </div>

      <div class="tb-editor">
        <span class="tb-editor-label">所选图片</span>
        <n-flex align="center" :size="8">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" @click="pickFiles">
                <template #icon><n-icon><Add /></n-icon></template>
              </n-button>
            </template>
            添加图片
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="clear">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
          <span class="tb-hint" v-if="!files.length">请选择一张或多张图片</span>
        </n-flex>
        <n-list v-if="files.length" bordered style="margin-top: 12px">
          <n-list-item v-for="(name, i) in fileNames" :key="i">
            <n-thing>
              <template #header>{{ name }}</template>
            </n-thing>
          </n-list-item>
        </n-list>
      </div>

      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">输出目录</span>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button quaternary @click="pickDir">
                <template #icon><n-icon><FolderOpen /></n-icon></template>
              </n-button>
            </template>
            选择输出目录
          </n-tooltip>
          <n-input v-model:value="outputDir" placeholder="输出目录路径" readonly />
        </div>
      </div>

      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" :loading="loading" @click="convert">
              <template #icon><n-icon><Upload /></n-icon></template>
            </n-button>
          </template>
          转换为 {{ outputFormat.toUpperCase() }}
        </n-tooltip>
      </div>

      <div v-if="outputs.length" class="tb-editor">
        <span class="tb-editor-label">转换结果</span>
        <n-data-table
          size="small"
          :bordered="true"
          :single-line="false"
          :columns="[
            { title: '输出文件', key: 'path' },
          ]"
          :data="outputs.map((p) => ({ path: p }))"
        >
        </n-data-table>
      </div>
    </section>
  </div>
</template>
