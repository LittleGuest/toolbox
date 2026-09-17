<script setup lang="ts">
const regexVisUrl = "/vendor/regex-vis/index.html";
</script>

<template>
  <div class="tb-page">
    <section class="tb-card regex-vis-card">
      <iframe class="regex-vis-frame" :src="regexVisUrl" title="Regex Vis" />
    </section>
  </div>
</template>

<style scoped>
.regex-vis-card {
  padding: 8px;
}

.regex-vis-frame {
  display: block;
  width: 100%;
  height: calc(100vh - 160px);
  min-height: 640px;
  border: 0;
  border-radius: var(--tb-radius-m);
  background: var(--tb-bg-elevated);
}
</style>
