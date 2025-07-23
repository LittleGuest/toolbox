<script setup>
import { ref, computed, h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import { ChevronRight } from "@vicons/carbon";
import { VueFlow, useVueFlow, MarkerType } from "@vue-flow/core";
import { MiniMap } from "@vue-flow/minimap";
import { Background } from "@vue-flow/background";
import { ControlButton, Controls } from "@vue-flow/controls";
import TableNode from "@/components/TableNode.vue";
import { datasourceDetailApi } from "@/db.js";

const message = useMessage();
const {
  onConnect,
  addEdges,
  addNodes,
  screenToFlowCoordinate,
  onNodesInitialized,
  updateNode,
} = useVueFlow();

// 定义属性
const props = defineProps({
  datasourceId: {
    type: Number,
  },
});

// 数据源详情
const datasourceInfo = ref({});
// 数据源表
const datasourceTables = ref([]);

// 数据库表API
const databaseTableTreeApi = async (info) => {
  return await invoke("database_table_tree", { datasourceInfo: info })
    .then((res) => {
      return res;
    })
    .catch((err) => {
      message.error(err);
    });
};

// 搜索表
const searchTable = ref("");
// 监听搜索表
watch(searchTable, async (val) => {
  if (val) {
    // 过滤表
    datasourceTables.value = datasourceTables.value.filter((item) =>
      item.name.includes(val)
    );
  } else {
    datasourceTables.value = await databaseTableTreeApi(datasourceInfo.value);
  }
});

// 节点
const nodes = ref([]);
// 边
const edges = ref([]);
// 拖拽节点类型
const draggedType = ref(null);
// 拖拽是否进行中
const isDragging = ref(false);
// 拖拽是否结束
const isDragOver = ref(false);

// 拖拽事件
const onDragStart = (event, type, data) => {
  if (event.dataTransfer) {
    // 传递数据
    event.dataTransfer.setData(
      "application/vueflow",
      JSON.stringify({
        type,
        data,
      })
    );
    event.dataTransfer.effectAllowed = "move";
  }
  draggedType.value = type;
  isDragging.value = true;
  document.addEventListener("drop", onDragEnd);
};

// 拖拽到画布上事件
const onDragOver = (event) => {
  event.preventDefault();
  if (draggedType.value) {
    isDragOver.value = true;
    if (event.dataTransfer) {
      event.dataTransfer.dropEffect = "move";
    }
  }
};

// 拖拽到画布外面的事件
function onDragLeave() {
  isDragOver.value = false;
}

// 拖拽结束事件
function onDragEnd() {
  isDragging.value = false;
  isDragOver.value = false;
  draggedType.value = null;
  document.removeEventListener("drop", onDragEnd);
}

// 拖拽放下事件
const onDrop = (event) => {
  event.preventDefault();

  let data = event.dataTransfer.getData("application/vueflow");
  data = JSON.parse(data);

  // 将屏幕坐标转换为画布坐标
  const position = screenToFlowCoordinate({
    x: event.clientX,
    y: event.clientY,
  });
  const nodeId = data.data.schema + "/" + data.data.tableName;
  const newNode = {
    id: nodeId,
    type: "table",
    position,
    data: data.data,
  };
  // 更新位置到鼠标中心
  const { off } = onNodesInitialized(() => {
    updateNode(nodeId, (node) => ({
      position: {
        x: node.position.x - node.dimensions.width / 2,
        y: node.position.y - node.dimensions.height / 2,
      },
    }));

    off();
  });
  addNodes(newNode);

  // TODO: 拖拽节点放下时
  // 自动生成生成器节点node,自动创建连线edge(为每个字段匹配一个合适的生成器)
  // TODO：节点放下动画,生成器节点生成的动画,自动创建连线的动画
};

onConnect(addEdges);

onMounted(async () => {
  // 数据源详情
  const detail = await datasourceDetailApi(props.datasourceId);
  datasourceInfo.value = detail[0];
  if (datasourceInfo.value) {
    datasourceTables.value = await databaseTableTreeApi(datasourceInfo.value);
  }
});
</script>

<template>
  <div class="flow-content">
    <!-- 左侧列表 -->
    <n-list class="sidebar" hoverable clickable :show-divider="false">
      <template #header>
        <n-input v-model:value="searchTable" placeholder="搜索..." />
      </template>
      <n-scrollbar style="max-height: 90vh">
        <n-list-item
          v-for="item in datasourceTables"
          :key="item.tableName"
          :draggable="true"
          @dragstart="onDragStart($event, 'input', item)"
        >
          <n-tooltip placement="right" trigger="hover">
            <template #trigger>
              <n-thing
                class="vue-flow__node-input"
                :description="item.tableName"
              />
            </template>
            {{ item.tableComment ? item.tableComment : item.tableName }}
          </n-tooltip>
        </n-list-item>
      </n-scrollbar>
    </n-list>

    <!-- 右侧画布 -->
    <VueFlow
      :nodes="nodes"
      :edges="edges"
      @drop="onDrop"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
    >
      <template #node-table="props">
        <TableNode :id="props.id" :data="props.data" />
      </template>

      <Background />
      <MiniMap />
    </VueFlow>
  </div>
</template>

<style lang="scss">
@import "@vue-flow/core/dist/style.css";
@import "@vue-flow/core/dist/theme-default.css";
@import "@vue-flow/controls/dist/style.css";
@import "@vue-flow/minimap/dist/style.css";
@import "@vue-flow/node-resizer/dist/style.css";

.flow-content {
  height: 100vh;
  display: flex;
  flex-direction: row;

  .sidebar {
    width: 300px;
    height: 100vh;
  }

  .nodes > * {
    margin-bottom: 10px;
    cursor: grab;
    font-weight: 500;
    -webkit-box-shadow: 5px 5px 10px 2px rgba(0, 0, 0, 0.25);
    box-shadow: 5px 5px 10px 2px #00000040;
  }
}
</style>
