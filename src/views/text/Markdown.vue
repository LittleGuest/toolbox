<script setup lang="ts">
import { ref } from "vue";
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const text = ref('');

</script>

<template>
  <div class="tb-page">
    <section class="tb-card md-card">
      <MdEditor v-model="text" class="md-editor" />
    </section>
  </div>
</template>

<style scoped>
.md-card {
  padding: 8px;
}

.md-editor {
  height: calc(100vh - 160px);
  min-height: 480px;
}
</style>
