<script setup>
import { onMounted, onUnmounted, ref } from 'vue'

// window.EXCALIDRAW_ASSET_PATH = "https://esm.sh/@excalidraw/excalidraw@0.18.0/dist/prod/"

const excalidraw = ref(null);

let root;
onMounted(() => {
  // root = createRoot(excalidraw.value);
  // root.render(React.createElement(Excalidraw, {}));
})

onUnmounted(() => {
  if (root) {
    root.unmount();
  }
});
</script>

<template>
  <div ref="excalidraw" classe="excalidraw">Loading...</div>
</template>

<style>
/* @import 'https://esm.sh/@excalidraw/excalidraw@0.18.0/dist/dev/index.css'; */

.excalidraw {
  height: 100%;
  width: 100%;
}
</style>
