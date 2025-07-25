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
    {{ data.database }}#{{ data.schema }}#{{ data.tableName }}
  </n-card>
</template>

<style lang="scss" scoped>
</style>
