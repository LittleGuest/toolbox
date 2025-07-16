<script setup>
import { Handle, Position, useVueFlow } from "@vue-flow/core";

const props = defineProps({
  id: {
    type: String,
    required: true,
  },
  data: {
    type: Object,
    required: true,
  },
});
</script>

<template>
  <n-card :title="id">
    <n-list>
      <n-list-item
        v-for="item in data.children"
        :key="item.name"
      >
      <!-- 提示 -->
        <n-tooltip placement="right" trigger="hover" style="background-color: white;">
          <template #trigger>
            <n-thing :description="item.name" />
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
                <td>{{ item.name }}</td>
                <td>{{ item.type }}</td>
                <td>{{ item.comment }}</td>
              </tr>
            </tbody>
          </n-table>
        </n-tooltip>
        <Handle
          :id="`target_${item.database}_${item.schema}_${item.tableName}_${item.name}`"
          type="target"
          :position="Position.Right"
        />
      </n-list-item>
    </n-list>
  </n-card>
</template>

<style lang="scss" scoped>
.tableInfo {
  display: flex;
  flex-direction: column;
}
</style>
