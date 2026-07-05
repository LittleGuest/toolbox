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
  <n-card class="column-node">
    <!-- 提示 -->
    <n-tooltip
      placement="right"
      trigger="hover"
      style="background-color: white"
    >
      <template #trigger>
        <n-thing :description="data.name" />
      </template>
      <n-table>
        <thead>
          <tr>
            <th>列名</th>
            <th>类型</th>
            <th>描述</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td>{{ data.name }}</td>
            <td>{{ data.type }}</td>
            <td>{{ data.comment }}</td>
          </tr>
        </tbody>
      </n-table>
    </n-tooltip>
    <Handle
      :id="`${data.database}#${data.schema}#${data.tableName}#${data.name}`"
      type="target"
      :position="Position.Right"
    />
  </n-card>
</template>

<style lang="scss" scoped>
.column-node {
  height: 48px;
  box-sizing: border-box;

  :deep(.n-card__content) {
    height: 100%;
    padding: 0 16px;
    display: flex;
    align-items: center;
  }
}
</style>
