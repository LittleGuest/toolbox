<script setup>
import { ref, computed, h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import { ChevronRight } from "@vicons/carbon";
import { VueFlow, useVueFlow, MarkerType } from "@vue-flow/core";
import { MiniMap } from "@vue-flow/minimap";
import { Background } from "@vue-flow/background";
import { ControlButton, Controls } from "@vue-flow/controls";
import TableNode from "@/components/datafaker/TableNode.vue";
import ColumnNode from "@/components/datafaker/ColumnNode.vue";
import DatafakerNode from "@/components/datafaker/DatafakerNode.vue";
import ColumnDatafakerDrawer from "@/components/datafaker/ColumnDatafakerDrawer.vue";
import { datasourceDetailApi } from "@/db.js";

const message = useMessage();
const {
  onConnect,
  findNode,
  addEdges,
  addNodes,
  screenToFlowCoordinate,
  onNodesInitialized,
  onNodeDoubleClick,
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
  const nodeId = data.data.schema + "#" + data.data.tableName;
  const tableNode = {
    id: nodeId,
    type: "table",
    width: 200,
    height: 30,
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
  addNodes(tableNode);

  // 查找表节点，获取表节点信息
  const tableNode2 = findNode(tableNode.id);
  const columnNodes = data.data.children.map((item, index) => {
    return {
      id: `${item.database}#${item.schema}#${item.tableName}#${item.name}`,
      type: "column",
      width: 200,
      height: 30,
      position: {
        x: tableNode2.position.x - tableNode2.width / 2,
        y: tableNode2.position.y + tableNode2.height + 18 + index * 63,
      },
      data: item,
    };
  });
  console.log("columnNodes", columnNodes);
  addNodes(columnNodes);

  // TODO: 拖拽节点放下时
  adapterGenerator(data.data, tableNode2);
};

// 自动生成生成器节点
// 为每个字段匹配一个合适的生成器
// data：字段信息
// pNode：表节点
const adapterGenerator = async (data, pNode) => {
  if (!data.children) {
    return;
  }
  // 字段的生成器
  const res = await adapterColumnsApi(
    data.children.map((item) => {
      return {
        name: item.name,
        columnType: item.type,
      };
    })
  );
  // 自动生成生成器节点node,自动创建连线edge(为每个字段匹配一个合适的生成器)

  // 自动生成生成器节点
  const generatorNodes = Object.entries(res).map(([key, value], index) => {
    return {
      id: data.schema + "#" + data.tableName + "#" + key + "#" + value,
      type: "datafaker",
      width: 120,
      height: 30,
      position: {
        x: pNode.position.x + pNode.dimensions.width + 200,
        y: pNode.position.y + pNode.height + index * 80,
      },
      data: {
        id: data.schema + "#" + data.tableName + "#" + key + "#" + value,
        columnName: key,
        datafaker: value,
        datafakerName: datafakersObj.value[value] || "未命名",
      },
    };
  });
  // 添加生成器节点到画布
  addNodes(generatorNodes);

  // 自动创建连线
  const edges = generatorNodes.map((item) => {
    const field = data.children.find((child) => {
      if (child.name === item.data.columnName) {
        return child;
      }
    });
    return {
      id: `${item.id}#edge`,
      source: item.id,
      target: `${field.database}#${field.schema}#${field.tableName}#${field.name}`,
      animated: true,
      markerEnd: MarkerType.ArrowClosed,
    };
  });
  addEdges(edges);
  // TODO: 节点放下动画,生成器节点生成的动画,自动创建连线的动画
};

// 打开生成器弹窗
const showDatafakerDialog = ref(false);
// 生成器弹窗数据
const datafakerData = ref();
// 节点双击事件处理
onNodeDoubleClick((event) => {
  console.log("双击节点:", event);
  const node = event.node;
  const data = node.data;
  if (node.type === "datafaker") {
    datafakerData.value = data;
    showDatafakerDialog.value = true;
  }
});

// onConnect(addEdges);

// 生成器列表
const datafakersObj = ref();

// 生成器列表
const datafakersApi = async () => {
  return await invoke("datafaker_providers")
    .then((res) => {
      return res;
    })
    .catch((err) => {
      message.error(err);
    });
};
// 生成器列表
const adapterColumnsApi = async (columns) => {
  return await invoke("datafaker_adapter_columns", { columns })
    .then((res) => {
      return res;
    })
    .catch((err) => {
      message.error(err);
    });
};

// 初始化
const init = async () => {
  // 数据源详情
  const detail = await datasourceDetailApi(props.datasourceId);
  datasourceInfo.value = detail[0];
  if (datasourceInfo.value) {
    // 数据表
    datasourceTables.value = await databaseTableTreeApi(datasourceInfo.value);
  }

  // 生成器列表
  datafakersObj.value = await datafakersApi();
};

onMounted(async () => {
  await init();
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
      :nodes-draggable="false"
      :zoom-on-double-click="false"
      @drop="onDrop"
      @dragover="onDragOver"
      @dragleave="onDragLeave"
    >
      <template #node-table="props">
        <TableNode :id="props.id" :data="props.data" />
      </template>
      <template #node-column="props">
        <ColumnNode :id="props.id" :data="props.data" />
      </template>
      <template #node-datafaker="props">
        <DatafakerNode :data="props.data" />
      </template>

      <Background />
      <MiniMap />
    </VueFlow>

    <ColumnDatafakerDrawer
      v-if="showDatafakerDialog"
      :show="showDatafakerDialog"
      :data="datafakerData"
    />
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
