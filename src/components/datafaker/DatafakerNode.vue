<script setup lang="ts">
import { Handle, Position, useVueFlow } from "@vue-flow/core";

const props = defineProps({
  data: {
    type: Object,
    required: true,
  },
});
</script>

<template>
  <n-card class="datafaker-node">
    <Handle :id="data.id" type="source" :position="Position.Left" />
    <span class="datafaker-node__name">{{ data.datafakerName }}</span>
    <!-- <span>{{ data.columnType }} - {{ data.datafaker }}</span> -->
  </n-card>
</template>

<style lang="scss" scoped>
.datafaker-node {
  height: 48px;
  box-sizing: border-box;

  :deep(.n-card__content) {
    height: 100%;
    padding: 0 16px;
    display: flex;
    align-items: center;
  }

  &__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}
</style>
