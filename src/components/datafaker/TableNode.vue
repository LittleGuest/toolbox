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
  <n-card class="table-node">
    {{ data.tableName }}
  </n-card>
</template>

<style lang="scss" scoped>
.table-node {
  background-color: #f5f5f5;
}
</style>
