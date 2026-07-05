<script setup lang="ts">
import { ref, computed, onUnmounted, h } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { NProgress, NCard, NDataTable, NInput, NButton, NPopconfirm, NSwitch, useMessage, NDrawer, NDrawerContent } from 'naive-ui';

// 消息提示
const message = useMessage();

// 定义监控数据的响应式变量
const cpuData = ref({
  usage: 0,
  temperature: 0
});

const gpuData = ref({
  usage: 0,
  temperature: 0,
  memory: 0
});

const memoryData = ref({
  usage: 0,
  total: 0,
  used: 0,
  swapUsage: 0,
  swapTotal: 0,
  swapUsed: 0
});

const diskData = ref({
  usage: 0,
  total: 0,
  used: 0
});

const networkData = ref({
  upload: 0,
  download: 0
});

const processes = ref([]);
const searchValue = ref('');
const monitoringEnabled = ref(false);

const diskDrawerVisible = ref(false);
const diskDetails = ref([]);

const cpuDrawerVisible = ref(false);
const cpuDetails = ref({});

const filteredProcesses = computed(() => {
  if (!searchValue.value) {
    return processes.value;
  }
  const searchTerm = searchValue.value.toLowerCase();
  return processes.value.filter(process =>
    process.name.toLowerCase().includes(searchTerm)
  );
});

const processColumns = [
  {
    title: '名称',
    key: 'name',
    sorter: (row1, row2) => row1.name.localeCompare(row2.name)
  },
  { title: 'PID', key: 'pid' },
  {
    title: '内存',
    key: 'memory',
    render: (row) => formatMemorySize(row.memory),
    sorter: (row1, row2) => row1.memory - row2.memory,
    defaultSortOrder: 'descend'
  },
  {
    title: 'CPU(%)',
    key: 'cpu',
    render: (row) => row.cpu.toFixed(2),
    sorter: (row1, row2) => row1.cpu - row2.cpu
  },
  {
    title: '操作',
    key: 'actions',
    width: 80,
    render: (row) => {
      return h(NPopconfirm, {
        onPositiveClick: () => killProcess(row.pid)
      }, {
        trigger: () => h(NButton, {
          size: 'small',
          type: 'error',
          ghost: true
        }, { default: () => '终止' }),
        default: () => `确定要终止进程 ${row.name} (PID: ${row.pid}) 吗？`
      });
    }
  }
];

// 定时器引用
let cpuMemoryTimer = null;
let diskTimer = null;
let processTimer = null;

const resetMonitorData = () => {
  cpuData.value = {
    usage: 0,
    temperature: 0
  };
  gpuData.value = {
    usage: 0,
    temperature: 0,
    memory: 0
  };
  memoryData.value = {
    usage: 0,
    total: 0,
    used: 0,
    swapUsage: 0,
    swapTotal: 0,
    swapUsed: 0
  };
  diskData.value = {
    usage: 0,
    total: 0,
    used: 0
  };
  networkData.value = {
    upload: 0,
    download: 0
  };
  processes.value = [];
  diskDetails.value = [];
  cpuDetails.value = {};
  searchValue.value = '';
}

const stopMonitoring = () => {
  if (cpuMemoryTimer) {
    clearInterval(cpuMemoryTimer);
    cpuMemoryTimer = null;
  }
  if (diskTimer) {
    clearInterval(diskTimer);
    diskTimer = null;
  }
  if (processTimer) {
    clearInterval(processTimer);
    processTimer = null;
  }
  diskDrawerVisible.value = false;
  cpuDrawerVisible.value = false;
  resetMonitorData();
}

const startMonitoring = () => {
  fetchCpuMemoryData();
  fetchDiskData();
  fetchProcessData();

  cpuMemoryTimer = setInterval(fetchCpuMemoryData, 2000);
  diskTimer = setInterval(fetchDiskData, 30000);
  processTimer = setInterval(fetchProcessData, 10000);
}

const handleMonitoringChange = (enabled) => {
  monitoringEnabled.value = enabled;
  if (enabled) {
    startMonitoring();
  } else {
    stopMonitoring();
  }
}

// 获取CPU和内存信息
const fetchCpuMemoryData = async () => {
  if (!monitoringEnabled.value) {
    return;
  }
  try {
    // 获取CPU信息
    const cpuInfo = await invoke('monitor_cpu_info');
    cpuData.value.usage = cpuInfo.global_usage || 0;

    // 保存CPU详细信息
    cpuDetails.value = cpuInfo;

    // 获取传感器信息（包括温度）
    const systemInfo = await invoke('monitor_system_info');
    if (systemInfo.sensors) {
      const cpuTempKey = Object.keys(systemInfo.sensors).find(key =>
        key.toLowerCase().includes('cpu') ||
        key.toLowerCase().includes('core')
      );
      if (cpuTempKey) {
        cpuData.value.temperature = systemInfo.sensors[cpuTempKey];
      }
    }

    // 获取内存信息
    const memoryInfo = await invoke('monitor_memory_info');
    if (memoryInfo.total_memory && memoryInfo.used_memory) {
      memoryData.value.usage = (memoryInfo.used_memory / memoryInfo.total_memory) * 100;
      memoryData.value.total = memoryInfo.total_memory;
      memoryData.value.used = memoryInfo.used_memory;
    }
    if (memoryInfo.total_swap && memoryInfo.used_swap) {
      memoryData.value.swapUsage = (memoryInfo.used_swap / memoryInfo.total_swap) * 100;
      memoryData.value.swapTotal = memoryInfo.total_swap;
      memoryData.value.swapUsed = memoryInfo.used_swap;
    }

    if (!monitoringEnabled.value) {
      return;
    }

    // 注意：GPU、磁盘读写和网络数据在当前后端实现中可能不可用
    // 这里使用模拟数据
    gpuData.value.usage = Math.random() * 100;
    gpuData.value.temperature = Math.random() * 50 + 30;
    gpuData.value.memory = Math.random() * 100;

    networkData.value.upload = Math.random() * 10;
    networkData.value.download = Math.random() * 20;

  } catch (error) {
    console.error('获取CPU和内存数据失败:', error);
  }
}

// 获取磁盘信息
const fetchDiskData = async () => {
  if (!monitoringEnabled.value) {
    return;
  }
  try {
    const diskInfo = await invoke('monitor_disk_info');
    if (!monitoringEnabled.value) {
      return;
    }
    if (diskInfo && diskInfo.length > 0) {
      let totalSpace = 0;
      let totalUsedSpace = 0;

      diskInfo.forEach(disk => {
        totalSpace += disk.total_space;
        totalUsedSpace += (disk.total_space - disk.available_space);
      });

      diskData.value.usage = (totalUsedSpace / totalSpace) * 100;
      diskData.value.total = totalSpace;
      diskData.value.used = totalUsedSpace;

      diskDetails.value = diskInfo;
    }
  } catch (error) {
    console.error('获取磁盘数据失败:', error);
  }
}

const openDiskDrawer = () => {
  diskDrawerVisible.value = true;
}

const openCpuDrawer = () => {
  cpuDrawerVisible.value = true;
}

// 获取进程信息
const fetchProcessData = async () => {
  if (!monitoringEnabled.value) {
    return;
  }
  try {
    const processInfo = await invoke('monitor_process_info');
    if (!monitoringEnabled.value) {
      return;
    }
    processes.value = processInfo;
  } catch (error) {
    console.error('获取进程数据失败:', error);
  }
}

// 格式化数字，保留两位小数
const formatNumber = (num) => {
  return num.toFixed(2);
}

// 格式化字节为合适的单位
const formatBytes = (bytes) => {
  if (bytes === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// 格式化内存大小为合适的单位
const formatMemorySize = (bytes) => {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// 删除进程
const killProcess = async (pid) => {
  try {
    await invoke('kill_process', { pid: parseInt(pid) });
    message.success(`进程 ${pid} 已终止`);
    // 刷新进程列表
    if (monitoringEnabled.value) {
      const processInfo = await invoke('monitor_process_info');
      processes.value = processInfo;
    }
  } catch (error) {
    message.error(`终止进程失败: ${error}`);
  }
}

// 组件卸载时清除定时器
onUnmounted(() => {
  stopMonitoring();
});
</script>

<template>
  <div class="system-monitor-container">
    <div v-if="!monitoringEnabled" class="monitor-switch-empty">
      <div class="monitor-switch-panel">
        <n-switch :value="monitoringEnabled" size="large" @update:value="handleMonitoringChange" />
        <div class="monitor-switch-title">系统监控已关闭</div>
        <div class="monitor-switch-desc">打开开关后开始采集 CPU、内存、磁盘和进程信息</div>
      </div>
    </div>

    <template v-else>
      <!-- CPU和内存监控 - 同一行 -->
      <div class="monitor-row">
        <!-- CPU监控 -->
        <n-card class="monitor-card" :bordered="true">
          <div class="item-content">
            <div class="metric" style="cursor: pointer;" @click="openCpuDrawer">
              <span class="metric-label">CPU使用率</span>
              <span class="metric-value">{{ formatNumber(cpuData.usage) }}%</span>
            </div>
            <n-progress type="line" :percentage="cpuData.usage" :color="'#4caf50'" :show-indicator="false" clickable
              @click="openCpuDrawer" :height="12" />
            <div class="metric" style="cursor: pointer;" @click="openDiskDrawer">
              <span class="metric-label">磁盘使用率</span>
              <span class="metric-value">{{ formatNumber(diskData.usage) }}%</span>
            </div>
            <n-progress type="line" :percentage="diskData.usage" :color="'#9c27b0'" :show-indicator="false" clickable
              @click="openDiskDrawer" :height="12" />
          </div>
        </n-card>

        <!-- 内存监控 -->
        <n-card class="monitor-card" :bordered="true">
          <div class="item-content">
            <div class="metric">
              <span class="metric-label">物理内存</span>
              <span class="metric-value">{{ formatNumber(memoryData.usage) }}%</span>
            </div>
            <n-progress type="line" :percentage="memoryData.usage" :color="'#2196f3'" :show-indicator="false" />
            <div class="metric">
              <span class="metric-label">交换内存</span>
              <span class="metric-value">{{ formatNumber(memoryData.swapUsage) }}%</span>
            </div>
            <n-progress type="line" :percentage="memoryData.swapUsage" :color="'#ff9800'" :show-indicator="false" />
          </div>
        </n-card>
      </div>

      <!-- 进程列表 -->
      <n-card class="monitor-card process-list-card" :bordered="true">
        <template #header>
          <div class="item-header" style="display: flex; justify-content: end; align-items: center;">
            <n-input v-model:value="searchValue" placeholder="搜索进程名称" clearable style="width: 200px;" />
          </div>
        </template>
        <n-data-table :columns="processColumns" :data="filteredProcesses" :pagination="{ pageSize: 20 }" :bordered="false"
          size="small" :row-key="(row) => row.pid" max-height="calc(100vh - 420px)" />
      </n-card>
    </template>

    <!-- 磁盘监控 -->
    <!-- <div class="monitor-item">
      <div class="item-header">
        <span class="item-icon">💾</span>
        <span class="item-title">磁盘</span>
      </div>
      <div class="item-content">
        <div class="disk-metrics">
          <div class="metric">
            <span class="metric-label">读取</span>
            <span class="metric-value">{{ formatBytes(diskData.read * 1024 * 1024) }}</span>
          </div>
          <div class="metric">
            <span class="metric-label">写入</span>
            <span class="metric-value">{{ formatBytes(diskData.write * 1024 * 1024) }}</span>
          </div>
        </div>
      </div>
    </div> -->

    <!-- 网络监控 -->
    <!-- <div class="monitor-item">
      <div class="item-header">
        <span class="item-icon">🌐</span>
        <span class="item-title">网络</span>
      </div>
      <div class="item-content">
        <div class="network-metrics">
          <div class="metric">
            <span class="metric-label">上传</span>
            <span class="metric-value">{{ formatBytes(networkData.upload * 1024 * 1024) }}</span>
          </div>
          <div class="metric">
            <span class="metric-label">下载</span>
            <span class="metric-value">{{ formatBytes(networkData.download * 1024 * 1024) }}</span>
          </div>
        </div>
      </div>
    </div> -->

    <!-- 磁盘详情抽屉 -->
    <n-drawer v-model:show="diskDrawerVisible" width="45%" placement="right">
      <n-drawer-content title="磁盘详情">
        <div v-for="(disk, index) in diskDetails" :key="index" style="margin-bottom: 15px;">
          <n-card :bordered="true" style="margin-bottom: 5px; ">
            <template #header>
              <div style="font-weight: bold; font-size: 14px;">{{ disk.mount_point }}</div>
            </template>
            <div style="display: flex; flex-direction: column; gap: 8px; font-size: 13px;">
              <div style="display: flex; justify-content: space-between;">
                <span style="color: #999;">文件系统:</span>
                <span>{{ disk.file_system }}</span>
              </div>
              <div style="display: flex; justify-content: space-between;">
                <span style="color: #999;">总空间:</span>
                <span>{{ formatMemorySize(disk.total_space) }}</span>
              </div>
              <div style="display: flex; justify-content: space-between;">
                <span style="color: #999;">可用空间:</span>
                <span>{{ formatMemorySize(disk.available_space) }}</span>
              </div>
              <div style="display: flex; justify-content: space-between;">
                <span style="color: #999;">已用空间:</span>
                <span>{{ formatMemorySize(disk.total_space - disk.available_space) }}</span>
              </div>
              <div>
                <div style="display: flex; justify-content: space-between; margin-bottom: 3px;">
                  <span style="color: #999;">使用率:</span>
                  <span>{{ formatNumber(((disk.total_space - disk.available_space) / disk.total_space) * 100) }}%</span>
                </div>
                <n-progress type="line"
                  :percentage="((disk.total_space - disk.available_space) / disk.total_space) * 100"
                  :show-indicator="false" />
              </div>
            </div>
          </n-card>
        </div>
      </n-drawer-content>
    </n-drawer>

    <!-- CPU详情抽屉 -->
    <n-drawer v-model:show="cpuDrawerVisible" width="45%" placement="right">
      <n-drawer-content title="CPU详情">
        <n-card :bordered="true" style="margin-bottom: 15px;">
          <div style="display: flex; flex-direction: column; gap: 8px; font-size: 13px;">
            <div style="display: flex; justify-content: space-between;">
              <span style="color: #999;">芯片名称:</span>
              <span>{{ cpuDetails.chip_name }}</span>
            </div>
            <div style="display: flex; justify-content: space-between;">
              <span style="color: #999;">物理核心数:</span>
              <span>{{ cpuDetails.physical_core_count }}</span>
            </div>
            <div style="display: flex; justify-content: space-between;">
              <span style="color: #999;">全局使用率:</span>
              <span>{{ formatNumber(cpuDetails.global_usage) }}%</span>
            </div>
          </div>
        </n-card>
        <div style="font-weight: bold; margin-bottom: 10px; font-size: 14px;">核心详情</div>
        <div v-for="(core, index) in cpuDetails.cores" :key="index" style="margin-bottom: 10px;">
          <n-card :bordered="true">
            <div style="display: flex; justify-content: space-between; margin-bottom: 5px; font-size: 13px;">
              <span style="color: #999;">核心 {{ index + 1 }}:</span>
              <span>{{ formatNumber(core.usage) }}%</span>
            </div>
            <n-progress type="line" :percentage="core.usage" :show-indicator="false" />
            <div style="display: flex; justify-content: space-between; margin-top: 5px; font-size: 13px;">
              <span style="color: #999;">频率:</span>
              <span>{{ core.frequency }} MHz</span>
            </div>
          </n-card>
        </div>
      </n-drawer-content>
    </n-drawer>
  </div>
</template>

<style lang="scss" scoped>
.system-monitor-container {
  padding: 10px;
  height: calc(100vh - 300px);
  display: flex;
  flex-direction: column;

  .monitor-title {
    font-size: 24px;
    font-weight: bold;
    margin-bottom: 30px;
    text-align: center;
  }

  .monitor-card {
    margin-bottom: 20px;
    border-radius: 12px;
  }

  .monitor-switch-empty {
    flex: 1;
    min-height: calc(100vh - 160px);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .monitor-switch-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 32px 40px;
    border: 1px solid var(--n-border-color);
    border-radius: 16px;
    background: var(--n-color);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.06);
  }

  .monitor-switch-title {
    font-size: 18px;
    font-weight: 600;
  }

  .monitor-switch-desc {
    font-size: 13px;
    color: var(--n-secondary-text-color);
  }

  .monitor-toolbar {
    display: flex;
    justify-content: flex-end;
    align-items: center;
    gap: 10px;
    margin-bottom: 12px;
    flex-shrink: 0;
  }

  .monitor-toolbar-label {
    font-size: 13px;
    color: var(--n-secondary-text-color);
  }

  .process-list-card {
    flex: 1;
    display: flex;
    flex-direction: column;

    :deep(.n-card__content) {
      flex: 1;
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }
  }

  .monitor-row {
    display: flex;
    gap: 20px;
    margin-bottom: 20px;
    flex-shrink: 0;

    .monitor-card {
      flex: 1;
      margin-bottom: 0;
    }
  }

  .item-header {
    display: flex;
    align-items: center;

    .item-icon {
      font-size: 20px;
      margin-right: 10px;
    }

    .item-title {
      font-size: 18px;
      font-weight: bold;
    }
  }

  .item-content {
    display: flex;
    flex-direction: column;
    gap: 15px;

    .metric {
      display: flex;
      justify-content: space-between;
      align-items: center;

      .metric-label {
        font-size: 14px;
        color: var(--n-secondary-text-color);
      }

      .metric-value {
        font-size: 16px;
        font-weight: bold;
      }
    }
  }

  .disk-metrics,
  .network-metrics {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
}
</style>
