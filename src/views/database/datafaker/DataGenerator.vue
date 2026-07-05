<script setup lang="ts">
import { nextTick, onUnmounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useMessage } from "naive-ui";
import type { LogInst } from "naive-ui";
import { useRouter } from "vue-router";
import { VueFlow, useVueFlow, MarkerType } from "@vue-flow/core";
import { MiniMap } from "@vue-flow/minimap";
import { Background } from "@vue-flow/background";
import TableNode from "@/components/datafaker/TableNode.vue";
import ColumnNode from "@/components/datafaker/ColumnNode.vue";
import DatafakerNode from "@/components/datafaker/DatafakerNode.vue";
import ColumnDatafakerDrawer from "@/components/datafaker/ColumnDatafakerDrawer.vue";
import { datasourceDetailApi } from "@/store/db";

const message = useMessage();
const router = useRouter();
const {
  addEdges,
  addNodes,
  screenToFlowCoordinate,
  onNodeDoubleClick,
  onNodeDrag,
  onNodeDragStop,
  onNodeDragStart,
  updateNode,
  fitView,
  setNodes,
  setEdges,
  toObject,
} = useVueFlow();
import {
  loadDatafakerConfigApi,
  saveDatafakerConfigApi,
} from "@/store/datafakerConfig";

// 定义属性
const props = defineProps({
  datasourceId: {
    type: Number,
  },
});

// 数据源详情
const datasourceInfo = ref({});
// 数据源表
const allDatasourceTables = ref([]);
const datasourceTables = ref([]);
const rowCount = ref(10);
const running = ref(false);
const runLogs = ref<string[]>([]);
const runLogVisible = ref(false);
const runLogUnlisten = ref<null | (() => void)>(null);
const runLogRef = ref<LogInst | null>(null);
const TABLE_NODE_WIDTH = 200;
const TABLE_NODE_HEIGHT = 48;
const COLUMN_NODE_WIDTH = 200;
const GENERATOR_NODE_WIDTH = 120;
const FIELD_NODE_HEIGHT = 48;
const FIELD_NODE_GAP = 0;
const GENERATOR_OFFSET_X = 340;
const draggingTablePositions = new Map<string, { x: number; y: number }>();

type RunLogPayload = {
  runId: string;
  message: string;
};

const scrollRunLogToBottom = async () => {
  await nextTick();
  runLogRef.value?.scrollTo({ position: "bottom", silent: true });
};

const showRunLog = () => {
  runLogVisible.value = true;
  scrollRunLogToBottom();
};

const closeRunLog = () => {
  runLogVisible.value = false;
};

const goBack = () => {
  router.back();
};

const fieldNodeY = (tableY: number, index: number) =>
  tableY +
  TABLE_NODE_HEIGHT +
  FIELD_NODE_GAP +
  index * (FIELD_NODE_HEIGHT + FIELD_NODE_GAP);

const sameTableNode = (node, tableData) => {
  const data = node.data || {};
  return data.schema === tableData.schema && data.tableName === tableData.tableName;
};

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

const filterDatasourceTables = (keyword: string) => {
  const value = keyword.trim().toLowerCase();
  if (!value) {
    datasourceTables.value = allDatasourceTables.value;
    return;
  }
  datasourceTables.value = allDatasourceTables.value.filter((table) => {
    const tableText = `${table.tableName || ""} ${table.tableComment || ""}`.toLowerCase();
    return tableText.includes(value);
  });
};

// 搜索表
const searchTable = ref("");
// 监听搜索表
watch(searchTable, async (val) => {
  filterDatasourceTables(val);
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
const onDrop = async (event) => {
  event.preventDefault();

  let data = event.dataTransfer.getData("application/vueflow");
  data = JSON.parse(data);

  // 将屏幕坐标转换为画布坐标
  const position = screenToFlowCoordinate({
    x: event.clientX,
    y: event.clientY,
  });
  const nodeId = data.data.schema + "#" + data.data.tableName;
  const tablePosition = {
    x: position.x - TABLE_NODE_WIDTH / 2,
    y: position.y - TABLE_NODE_HEIGHT / 2,
  };
  const tableNode = {
    id: nodeId,
    type: "table",
    width: TABLE_NODE_WIDTH,
    height: TABLE_NODE_HEIGHT,
    position: tablePosition,
    draggable: true,
    data: data.data,
  };
  addNodes(tableNode);

  const columnNodes = data.data.children.map((item, index) => {
    return {
      id: `${item.database}#${item.schema}#${item.tableName}#${item.name}`,
      type: "column",
      width: COLUMN_NODE_WIDTH,
      height: FIELD_NODE_HEIGHT,
      draggable: false,
      position: {
        x: tablePosition.x,
        y: fieldNodeY(tablePosition.y, index),
      },
      data: item,
    };
  });
  addNodes(columnNodes);

  await adapterGenerator(data.data, tableNode);
  onDragEnd();
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
  const columnDatafaker = await adapterColumnsApi(
    data.children.map((item) => {
      return {
        name: item.name,
        columnType: item.type,
      };
    })
  );
  // 自动生成生成器节点（为每个字段匹配一个合适的生成器）
  const generatorNodes = Object.entries(columnDatafaker).map(
    ([key, value], index) => {
      const columnData = data.children.find((item) => item.name === key);
      return {
        id: data.schema + "#" + data.tableName + "#" + key + "#" + value,
        type: "datafaker",
        width: GENERATOR_NODE_WIDTH,
        height: FIELD_NODE_HEIGHT,
        draggable: false,
        position: {
          x: pNode.position.x + GENERATOR_OFFSET_X,
          y: fieldNodeY(pNode.position.y, index),
        },
        data: {
          id: data.schema + "#" + data.tableName + "#" + key + "#" + value,
          columnName: key,
          datafaker: value,
          datafakerName: datafakersObj.value[value] || "未命名",
          config: {},
          ...columnData,
        },
      };
    }
  );
  // 添加生成器节点到画布
  addNodes(generatorNodes);

  // 自动创建连线
  const edges = generatorNodes.map((item) => {
    const field = data.children.find(
      (child) => child.name === item.data.columnName
    );
    return {
      id: `${item.id}#edge`,
      source: item.id,
      target: `${field.database}#${field.schema}#${field.tableName}#${field.name}`,
      animated: true,
      markerEnd: MarkerType.ArrowClosed,
    };
  });
  addEdges(edges);
  fitView({ duration: 300, padding: 0.2 });
  refreshReferenceOptions();
};

// 打开生成器弹窗
const showDatafakerDialog = ref(false);
// 生成器弹窗数据
const datafakerData = ref();
const referenceOptions = ref([]);
const refreshReferenceOptions = () => {
  const flow = toObject();
  referenceOptions.value = flow.nodes
    .filter((node) => node.type === "column")
    .map((node) => {
      const data = node.data;
      return {
        label: `${data.schema}.${data.tableName}.${data.name}`,
        value: `${data.schema}#${data.tableName}#${data.name}`,
        schema: data.schema,
        tableName: data.tableName,
        column: data.name,
      };
    });
};
// 节点双击事件处理
onNodeDoubleClick((event) => {
  const node = event.node;
  const data = node.data;
  if (node.type === "datafaker") {
    refreshReferenceOptions();
    datafakerData.value = data;
    showDatafakerDialog.value = true;
  }
});

onNodeDragStart(({ node }) => {
  if (node.type !== "table") {
    return;
  }
  draggingTablePositions.set(node.id, { ...node.position });
});

onNodeDrag(({ node }) => {
  if (node.type !== "table") {
    return;
  }
  const lastPosition = draggingTablePositions.get(node.id);
  if (!lastPosition) {
    draggingTablePositions.set(node.id, { ...node.position });
    return;
  }
  const deltaX = node.position.x - lastPosition.x;
  const deltaY = node.position.y - lastPosition.y;
  if (deltaX === 0 && deltaY === 0) {
    return;
  }
  draggingTablePositions.set(node.id, { ...node.position });
  const tableData = node.data;
  toObject()
    .nodes.filter((item) => item.id !== node.id && sameTableNode(item, tableData))
    .forEach((item) => {
      updateNode(item.id, (current) => ({
        position: {
          x: current.position.x + deltaX,
          y: current.position.y + deltaY,
        },
      }));
    });
});

onNodeDragStop(({ node }) => {
  draggingTablePositions.delete(node.id);
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

const currentConfig = () => ({
  datasourceId: props.datasourceId,
  rowCount: rowCount.value,
  nodes: toObject().nodes,
  edges: toObject().edges,
});

const saveConfig = async () => {
  await saveDatafakerConfigApi(currentConfig());
  message.success("配置已保存");
};

const clearCanvas = () => {
  nodes.value = [];
  edges.value = [];
  setNodes([]);
  setEdges([]);
  runLogs.value = [];
  runLogVisible.value = false;
  refreshReferenceOptions();
  message.success("画布已清空，保存配置后生效");
};

const deleteTable = (tableId) => {
  const flow = toObject();
  const tableNode = flow.nodes.find((node) => node.id === tableId);
  if (!tableNode) {
    return;
  }
  const tableData = tableNode.data;
  const removeNodeIds = new Set(
    flow.nodes
      .filter((node) => {
        if (node.id === tableId) {
          return true;
        }
        const data = node.data || {};
        return (
          data.schema === tableData.schema && data.tableName === tableData.tableName
        );
      })
      .map((node) => node.id)
  );
  const nextNodes = flow.nodes.filter((node) => !removeNodeIds.has(node.id));
  const nextEdges = flow.edges.filter(
    (edge) => !removeNodeIds.has(edge.source) && !removeNodeIds.has(edge.target)
  );
  nodes.value = nextNodes;
  edges.value = nextEdges;
  setNodes(nextNodes);
  setEdges(nextEdges);
  refreshReferenceOptions();
  message.success(`已删除表 ${tableData.tableName}，保存配置后生效`);
};

const loadConfig = async () => {
  if (!props.datasourceId) {
    return;
  }
  const config = await loadDatafakerConfigApi(props.datasourceId);
  if (!config) {
    return;
  }
  rowCount.value = config.rowCount || 10;
  nodes.value = config.nodes || [];
  edges.value = config.edges || [];
  setNodes(nodes.value);
  setEdges(edges.value);
  fitView({ duration: 200, padding: 0.2 });
};

const buildRunTables = () => {
  const flow = toObject();
  const tableNodes = flow.nodes.filter((node) => node.type === "table");
  const generatorNodes = flow.nodes.filter((node) => node.type === "datafaker");
  return tableNodes.map((tableNode) => {
    const tableData = tableNode.data;
    const columns = tableData.children.map((column) => {
      const generatorNode = generatorNodes.find((node) => {
        const data = node.data;
        return (
          data.schema === column.schema &&
          data.tableName === column.tableName &&
          data.columnName === column.name
        );
      });
      return {
        name: column.name,
        columnType: column.type,
        rustType: column.rustType,
        generator: generatorNode?.data?.datafaker || "text",
        config: generatorNode?.data?.config || {},
      };
    });
    return {
      schema: tableData.schema,
      tableName: tableData.tableName,
      columns,
    };
  });
};

const stopRunLogListener = () => {
  if (runLogUnlisten.value) {
    runLogUnlisten.value();
    runLogUnlisten.value = null;
  }
};

const startRunLogListener = async (runId: string) => {
  stopRunLogListener();
  runLogUnlisten.value = await listen<RunLogPayload>(
    "datafaker-run-log",
    (event) => {
      if (event.payload.runId === runId) {
        runLogs.value.push(event.payload.message);
        scrollRunLogToBottom();
      }
    }
  );
};

watch(
  () => runLogs.value.length,
  () => {
    scrollRunLogToBottom();
  }
);

const runConfig = async () => {
  const tables = buildRunTables();
  if (!tables.length) {
    message.warning("请先拖入至少一张表");
    return;
  }
  runLogs.value = [];
  runLogVisible.value = true;
  running.value = true;
  const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  try {
    await startRunLogListener(runId);
    await saveConfig();
    const res = await invoke("datafaker_run_config", {
      datasourceInfo: datasourceInfo.value,
      rowCount: rowCount.value,
      tables,
      runId,
    });
    if ((res.logs || []).length > runLogs.value.length) {
      runLogs.value = res.logs || [];
    }
    if (res.success) {
      message.success(`运行完成，插入 ${res.insertedRows} 行`);
    } else {
      message.error(`运行失败，已插入 ${res.insertedRows} 行，失败 ${res.failedRows} 行`);
    }
  } catch (err) {
    message.error(`${err}`);
    runLogs.value.push(`运行异常: ${err}`);
  } finally {
    stopRunLogListener();
    running.value = false;
  }
};

// 初始化
const init = async () => {
  // 数据源详情
  const detail = await datasourceDetailApi(props.datasourceId);
  datasourceInfo.value = detail[0];
  if (datasourceInfo.value) {
    // 数据表
    allDatasourceTables.value = await databaseTableTreeApi(datasourceInfo.value);
    datasourceTables.value = allDatasourceTables.value;
  }

  // 生成器列表
  datafakersObj.value = await datafakersApi();
  await loadConfig();
};

onMounted(async () => {
  await init();
});

onUnmounted(() => {
  stopRunLogListener();
  draggingTablePositions.clear();
});
</script>

<template>
  <div class="flow-content">
    <!-- 左侧列表 -->
    <n-list class="sidebar" hoverable clickable :show-divider="false">
      <template #header>
        <div class="sidebar-header">
          <n-button secondary block @click="goBack">返回上一页</n-button>
          <n-input v-model:value="searchTable" placeholder="搜索表..." />
        </div>
      </template>
      <n-scrollbar class="sidebar-scroll">
        <n-list-item
          v-for="item in datasourceTables"
          :key="item.tableName"
          :draggable="true"
          @dragstart="onDragStart($event, 'input', item)"
        >
          <div class="sidebar-table">
            <div class="sidebar-table__name">{{ item.tableName }}</div>
            <div v-if="item.tableComment" class="sidebar-table__comment">
              {{ item.tableComment }}
            </div>
          </div>
        </n-list-item>
      </n-scrollbar>
    </n-list>

    <div class="flow-main">
      <!-- 右侧画布 -->
      <VueFlow
        :nodes="nodes"
        :edges="edges"
        :nodes-draggable="true"
        :zoom-on-double-click="false"
        @drop="onDrop"
        @dragover="onDragOver"
        @dragleave="onDragLeave"
      >
        <div class="canvas-toolbar">
          <n-space align="center">
            <span>每张表生成</span>
            <n-input-number v-model:value="rowCount" :min="1" :max="100000" />
            <span>行</span>
            <n-button @click="saveConfig">保存配置</n-button>
            <n-popconfirm @positive-click="clearCanvas">
              <template #trigger>
                <n-button secondary type="warning">清空画布</n-button>
              </template>
              清空后会移除所有表、字段、生成器和连线，确定继续？
            </n-popconfirm>
            <n-button type="primary" :loading="running" @click="runConfig">
              运行配置
            </n-button>
            <n-button
              v-if="runLogs.length && !runLogVisible"
              secondary
              @click="showRunLog"
            >
              查看日志（{{ runLogs.length }}）
            </n-button>
          </n-space>
        </div>
        <n-card
          v-if="runLogs.length && runLogVisible"
          class="run-log"
          size="small"
          title="运行日志"
        >
          <template #header-extra>
            <n-button text size="small" @click="closeRunLog">关闭</n-button>
          </template>
          <n-log
            ref="runLogRef"
            :log="runLogs.join('\n')"
            language="text"
            :rows="8"
          />
        </n-card>
        <template #node-table="props">
          <TableNode
            :id="props.id"
            :data="props.data"
            @delete-table="deleteTable"
          />
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
    </div>

    <ColumnDatafakerDrawer
      v-if="showDatafakerDialog"
      v-model:show="showDatafakerDialog"
      :data="datafakerData"
      :reference-options="referenceOptions"
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
    border-right: 1px solid #eee;
  }

  .sidebar-header {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .sidebar-scroll {
    max-height: calc(100vh - 100px);
  }

  .sidebar-table {
    width: 100%;
  }

  .sidebar-table__name {
    font-weight: 600;
  }

  .sidebar-table__comment {
    margin-top: 2px;
    color: #888;
    font-size: 12px;
  }

  .flow-main {
    flex: 1;
    display: flex;
    flex-direction: column;
  }

  .canvas-toolbar {
    position: absolute;
    top: 12px;
    left: 12px;
    z-index: 10;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.96);
    border: 1px solid #eee;
    border-radius: 6px;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.08);
  }

  .run-log {
    position: absolute;
    top: 64px;
    left: 12px;
    right: 12px;
    z-index: 10;
    max-width: 960px;
  }

  .vue-flow {
    flex: 1;
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
