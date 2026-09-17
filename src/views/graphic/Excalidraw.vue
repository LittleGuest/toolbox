<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import React from "react";
import { createRoot, type Root } from "react-dom/client";
import { Excalidraw } from "@excalidraw/excalidraw";
import "@excalidraw/excalidraw/index.css";

const containerRef = ref<HTMLDivElement | null>(null);
let root: Root | null = null;

onMounted(() => {
  if (!containerRef.value) {
    return;
  }
  root = createRoot(containerRef.value);
  root.render(React.createElement(Excalidraw));
});

onUnmounted(() => {
  root?.unmount();
  root = null;
});
</script>

<template>
  <div class="excalidraw-page">
    <div ref="containerRef" class="excalidraw-container" />
  </div>
</template>

<style scoped>
.excalidraw-page {
  width: 100%;
  height: calc(100vh - 80px);
  min-height: 600px;
  display: flex;
  flex-direction: column;
  padding: 4px 12px 12px;
}

.excalidraw-container {
  width: 100%;
  flex: 1;
  min-height: 0;
  overflow: hidden;
  border-radius: 8px;
  background: var(--tb-bg-elevated);
}
</style>
