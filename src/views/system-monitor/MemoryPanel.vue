<script setup>
import { ref, onMounted, provide, inject, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import 'echarts';
import 'echarts/theme/blue'
import VChart, { THEME_KEY } from 'vue-echarts';
import { memoryOption, swapOption } from "./options/options.js";
provide(THEME_KEY, 'blue')

const isMonitoring = inject('isMonitoring');

const totalMemory = ref("16");
const totalSwap = ref("8");
const memoryChart = ref(null);
const swapChart = ref(null);
let memoryInterval = null;

const updateMemoryUsage = (memory) => {
    const total = Number.parseFloat(totalMemory.value);
    const used = Number.parseFloat(memory.used_memory.toFixed(1));
    const free = total - used;
    
    const swapTotal = Number.parseFloat(totalSwap.value);
    const usedSwap = Number.parseFloat(memory.used_swap.toFixed(1));
    const freeSwap = swapTotal - usedSwap;
    
    console.log('Memory update:', {
        totalMemory: totalMemory.value,
        used_memory: memory.used_memory,
        total_swap: memory.total_swap,
        used_swap: memory.used_swap,
        total,
        used,
        free,
        swapTotal,
        usedSwap,
        freeSwap
    });
    
    memoryChart.value?.setOption({
        xAxis: {
            max: total,
            interval: Math.floor(total / 16) * 4,
        },
        dataset: {
            source: [['memory', used, free]]
        }
    });
    
    swapChart.value?.setOption({
        xAxis: {
            max: swapTotal,
            interval: Math.floor(swapTotal / 16) * 4,
        },
        dataset: {
            source: [['swap', usedSwap, freeSwap]]
        }
    });
}

const flushMemoryUsage = () => {
    if (!isMonitoring.value) return;
    invoke("monitor_memory_info", {}).then(memory => {
        console.log(memory)
        totalMemory.value = memory.total_memory.toFixed(0);
        totalSwap.value = memory.total_swap.toFixed(0);
        updateMemoryUsage(memory);
    });
}

const startMemoryMonitoring = () => {
    if (memoryInterval) return;
    flushMemoryUsage();
    memoryInterval = setInterval(flushMemoryUsage, 10 * 1000);
}

const stopMemoryMonitoring = () => {
    if (memoryInterval) {
        clearInterval(memoryInterval);
        memoryInterval = null;
    }
}

watch(isMonitoring, (newValue) => {
    if (newValue) {
        startMemoryMonitoring();
    } else {
        stopMemoryMonitoring();
    }
})

onMounted(async () => {
    console.log('Memory chart mounted');
    memoryChart.value?.setOption(memoryOption);
    swapChart.value?.setOption(swapOption);
    console.log('Memory option set:', memoryOption);
    console.log('Swap option set:', swapOption);
    if (isMonitoring.value) {
        console.log('Starting memory monitoring on mount');
        startMemoryMonitoring();
    }
})
</script>

<template>
    <n-row :gutter="8">
        <n-col :span="12">
            <n-card>
                <div class="card-header">
                    <span style="width: 40px">内存</span>
                    <v-chart class="chart" ref="memoryChart" :manual-update="true" autoresize />
                </div>
            </n-card>
        </n-col>
        <n-col :span="12">
            <n-card>
                <div class="card-header">
                    <span style="width: 40px">交换</span>
                    <v-chart class="chart" ref="swapChart" :manual-update="true" autoresize />
                </div>
            </n-card>
        </n-col>
    </n-row>
</template>

<style lang="scss" scoped>
.chart {
    height: 41px;
    width: 100%;
}

.n-row {
    margin-bottom: 8px;

    &:last-child {
        margin-bottom: 0;
    }
}

.n-card {
    --n-padding: 8px;
}

.card-header {
    font-size: smaller;
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--n-text-color);
}
</style>