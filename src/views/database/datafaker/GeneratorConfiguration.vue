<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, useMessage } from "naive-ui";
import { ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Copy, Paste, Close, Reset } from "@vicons/carbon";

const message = useMessage();

const configForm = ref();

</script>

<template>
  <n-form>
    <n-form-item label="生成器">
      <n-select placeholder="请选择生成器" label-field="name" value-field="name" v-model:value="configForm"
        :options="datasourceInfos" @update:value="handleSelectDatasource" />
    </n-form-item>
    <n-form-item>
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-button @click="handleValidateClick">
            <template #icon>
              <n-icon><Reset /></n-icon>
            </template>
          </n-button>
        </template>
        重置属性
      </n-tooltip>
    </n-form-item>
  </n-form>
</template>

<style lang="scss" scoped></style>
