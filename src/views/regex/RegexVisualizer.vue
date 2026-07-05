<script setup lang="ts">
const regexVisUrl = "/vendor/regex-vis/index.html";
</script>

<template>
  <div class="regex-vis-page">
    <iframe class="regex-vis-frame" :src="regexVisUrl" title="Regex Vis" />
  </div>
</template>

<style scoped>
.regex-vis-page {
  width: 100%;
  height: calc(100vh - 36px);
  min-height: 720px;
  overflow: hidden;
  border-radius: 8px;
  background: #fff;
}

.regex-vis-frame {
  display: block;
  width: 100%;
  height: 100%;
  border: 0;
  background: #fff;
}
</style>
