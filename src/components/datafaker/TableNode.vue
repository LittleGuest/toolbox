<script setup lang="ts">
import { ref } from "vue";

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

const emit = defineEmits(["delete-table"]);

const showDropdown = ref(false);
const dropdownX = ref(0);
const dropdownY = ref(0);

const deleteTable = () => {
  showDropdown.value = false;
  emit("delete-table", props.id);
};

const openContextMenu = (event: MouseEvent) => {
  event.preventDefault();
  dropdownX.value = event.clientX;
  dropdownY.value = event.clientY;
  showDropdown.value = false;
  requestAnimationFrame(() => {
    showDropdown.value = true;
  });
};

const contextOptions = [{ label: "删除表", key: "delete" }];

const handleSelect = (key: string) => {
  if (key === "delete") {
    deleteTable();
  }
};
</script>

<template>
  <n-card class="table-node" @contextmenu="openContextMenu">
    <div class="table-node__header">
      <span class="table-node__name">{{ data.tableName }}</span>
    </div>
    <n-dropdown
      placement="bottom-start"
      trigger="manual"
      :x="dropdownX"
      :y="dropdownY"
      :options="contextOptions"
      :show="showDropdown"
      @select="handleSelect"
      @clickoutside="showDropdown = false"
    />
  </n-card>
</template>

<style lang="scss" scoped>
.table-node {
  background-color: #f5f5f5;
  height: 48px;
  box-sizing: border-box;

  :deep(.n-card__content) {
    height: 100%;
    padding: 0 16px;
  }

  &__header {
    display: flex;
    align-items: center;
    height: 100%;
  }

  &__name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}
</style>
