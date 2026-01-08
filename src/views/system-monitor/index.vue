<script setup>
import { ref, reactive, computed, onMounted, onUnmounted, inject } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { NProgress, NCard, NDataTable, NInput } from 'naive-ui';

// 注入主题设置
const theme = inject('theme');

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
  usage: 0
});

const diskData = ref({
  read: 0,
  write: 0
});

const networkData = ref({
  upload: 0,
  download: 0
});

const processes = ref([]);
const searchValue = ref('');

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
  }
];

// 定时器引用
let timer = null;

// 获取系统监控数据
async function fetchSystemData() {
  try {
    // 获取CPU信息
    const cpuInfo = await invoke('monitor_cpu_info');
    cpuData.value.usage = cpuInfo.global_usage || 0;
    
    // 获取传感器信息（包括温度）
    const systemInfo = await invoke('monitor_system_info');
    if (systemInfo.sensors) {
      // 尝试获取CPU温度（不同系统可能有不同的传感器名称）
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
    
    // 注意：GPU、磁盘读写和网络数据在当前后端实现中可能不可用
    // 这里使用模拟数据
    gpuData.value.usage = Math.random() * 100;
    gpuData.value.temperature = Math.random() * 50 + 30;
    gpuData.value.memory = Math.random() * 100;
    
    diskData.value.read = Math.random() * 50;
    diskData.value.write = Math.random() * 50;
    
    networkData.value.upload = Math.random() * 10;
    networkData.value.download = Math.random() * 20;
    
    // 获取进程信息
    const processInfo = await invoke('monitor_process_info');
    processes.value = processInfo;
    
  } catch (error) {
    console.error('获取系统监控数据失败:', error);
  }
}

// 格式化数字，保留两位小数
function formatNumber(num) {
  return num.toFixed(2);
}

// 格式化字节为合适的单位
function formatBytes(bytes) {
  if (bytes === 0) return '0 B/s';
  const k = 1024;
  const sizes = ['B/s', 'KB/s', 'MB/s', 'GB/s'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// 格式化内存大小为合适的单位
function formatMemorySize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}

// 组件挂载时开始定时获取数据
onMounted(() => {
  // 立即获取一次数据
  fetchSystemData();
  // 设置定时器，每2秒更新一次数据
  timer = setInterval(fetchSystemData, 2000);
});

// 组件卸载时清除定时器
onUnmounted(() => {
  if (timer) {
    clearInterval(timer);
  }
});
</script>

<template>
  <div class="system-monitor-container">
    <!-- CPU监控 -->
    <n-card class="monitor-card" :bordered="true">
      <template #header>
        <div class="item-header">
          <span class="item-icon">🖥️</span>
          <span class="item-title">CPU</span>
        </div>
      </template>
      <div class="item-content">
        <div class="metric">
          <span class="metric-label">CPU使用率</span>
          <span class="metric-value">{{ formatNumber(cpuData.usage) }}%</span>
        </div>
        <n-progress type="line" :percentage="cpuData.usage" :color="'#4caf50'" :show-indicator="false" />
        <div class="metric">
          <span class="metric-label">CPU温度</span>
          <span class="metric-value">{{ formatNumber(cpuData.temperature) }}°C</span>
        </div>
      </div>
    </n-card>
    
    <!-- GPU监控 -->
    <!-- <div class="monitor-item">
      <div class="item-header">
        <span class="item-icon">🎮</span>
        <span class="item-title">GPU</span>
      </div>
      <div class="item-content">
        <div class="metric">
          <span class="metric-label">GPU使用率</span>
          <span class="metric-value">{{ formatNumber(gpuData.usage) }}%</span>
        </div>
        <n-progress type="line" :percentage="gpuData.usage" :color="'#f44336'" :show-indicator="false" />
        <div class="metric">
          <span class="metric-label">GPU温度</span>
          <span class="metric-value">{{ formatNumber(gpuData.temperature) }}°C</span>
        </div>
      </div>
    </div> -->
    
    <!-- 显存监控 -->
    <!-- <div class="monitor-item">
      <div class="item-header">
        <span class="item-icon">📊</span>
        <span class="item-title">显存占用</span>
      </div>
      <div class="item-content">
        <div class="metric">
          <span class="metric-value">{{ formatNumber(gpuData.memory) }}%</span>
        </div>
        <n-progress type="line" :percentage="gpuData.memory" :color="'#ff9800'" :show-indicator="false" />
      </div>
    </div> -->
    
    <!-- 内存监控 -->
    <n-card class="monitor-card" :bordered="true">
      <template #header>
        <div class="item-header">
          <span class="item-icon">🧠</span>
          <span class="item-title">内存</span>
        </div>
      </template>
      <div class="item-content">
        <div class="metric">
          <span class="metric-label">内存占用</span>
          <span class="metric-value">{{ formatNumber(memoryData.usage) }}%</span>
        </div>
        <n-progress type="line" :percentage="memoryData.usage" :color="'#2196f3'" :show-indicator="false" />
      </div>
    </n-card>
    
    <!-- 进程列表 -->
    <n-card class="monitor-card" :bordered="true">
      <template #header>
        <div class="item-header" style="display: flex; justify-content: space-between; align-items: center;">
          <div style="display: flex; align-items: center;">
            <span class="item-icon">📋</span>
            <span class="item-title">进程列表</span>
          </div>
          <n-input
            v-model:value="searchValue"
            placeholder="搜索进程名称"
            clearable
            style="width: 200px;"
          />
        </div>
      </template>
      <n-data-table
        :columns="processColumns"
        :data="filteredProcesses"
        :pagination="{ pageSize: 20 }"
        :bordered="false"
        size="small"
        :row-key="(row) => row.pid"
      />
    </n-card>
    
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
  </div>
</template>

<style lang="scss" scoped>
.system-monitor-container {
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

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

.item-header {
  display: flex;
  align-items: center;
}

.item-icon {
  font-size: 20px;
  margin-right: 10px;
}

.item-title {
  font-size: 18px;
  font-weight: bold;
}

.item-content {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.metric {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.metric-label {
  font-size: 14px;
  color: var(--n-secondary-text-color);
}

.metric-value {
  font-size: 16px;
  font-weight: bold;
}

.disk-metrics,
.network-metrics {
  display: flex;
  flex-direction: column;
  gap: 10px;
}
</style>