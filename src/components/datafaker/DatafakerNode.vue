<script setup>
import { Handle, Position, useVueFlow } from "@vue-flow/core";

const props = defineProps({
  data: {
    type: Object,
    required: true,
  },
});
</script>

<template>
  <n-card>
    <Handle :id="data.id" type="source" :position="Position.Left" />
    {{ data.datafakerName }}
    <!-- <span>{{ data.columnType }} - {{ data.datafaker }}</span> -->
  </n-card>
</template>

<style lang="scss" scoped>
// .container {
//   width: 100%;
//   display: flex;
//   justify-content: space-around;
//   border: 1px solid #000;
//   border-radius: 3px;
// }
</style>
