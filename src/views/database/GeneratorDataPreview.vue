<script setup>
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, createDiscreteApi } from "naive-ui";
import { ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Copy, Paste, Close } from "@vicons/carbon";

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi(["message", "dialog", "notification", "loadingBar", "modal"]);

const columns = ref([
  {
    title: "数据预览",
    key: "preview",
    children: [
      {
        title: "Attack",
        key: "attack",
      },
      {
        title: "Defend",
        key: "defend"
      },
      {
        title: "Speed",
        key: "speed"
      }
    ]
  }
]);

const data = ref([]);
</script>

<template>
  <n-data-table :columns="columns" :data="data" :pagination="false" :bordered="false" />
</template>

<style lang="scss" scoped></style>
