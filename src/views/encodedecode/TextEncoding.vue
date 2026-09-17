<script setup lang="ts">
import CharsetEncoder from "./CharsetEncoder.vue";
import MessyCodeRecover from "./MessyCodeRecover.vue";
import EscapeTools from "./EscapeTools.vue";
import Utf8Unicode from "./Utf8Unicode.vue";
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <n-tabs type="line" animated>
        <n-tab-pane name="charset" tab="字符编码"><CharsetEncoder /></n-tab-pane>
        <n-tab-pane name="messy" tab="乱码恢复"><MessyCodeRecover /></n-tab-pane>
        <n-tab-pane name="escape" tab="转义工具"><EscapeTools /></n-tab-pane>
        <n-tab-pane name="unicode" tab="Unicode 转义"><Utf8Unicode /></n-tab-pane>
      </n-tabs>
    </section>
  </div>
</template>
