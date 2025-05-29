<script setup>
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, createDiscreteApi } from "naive-ui";
import { ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Copy, Paste, Close } from "@vicons/carbon";

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi(["message", "dialog", "notification", "loadingBar", "modal"]);

const configForm = ref();

</script>

<template>
  <n-form>
    <n-form-item label="生成器">
      <n-select placeholder="请选择生成器" label-field="name" value-field="name" v-model:value="configForm"
        :options="datasourceInfos" @update:value="handleSelectDatasource" />
    </n-form-item>
    <n-form-item>
      <n-button @click="handleValidateClick">
        重置属性
      </n-button>
    </n-form-item>
  </n-form>
</template>

<style lang="scss" scoped></style>
