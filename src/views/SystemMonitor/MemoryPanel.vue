<script setup>
import { ref, onMounted, provide } from "vue";
import { invoke } from "@tauri-apps/api/core";
import 'echarts';
import 'echarts/theme/blue'
import VChart, { THEME_KEY } from 'vue-echarts';
import { memoryOption } from "./options/options.js";
provide(THEME_KEY, 'blue')

const totalMemory = ref("16");
const memoryChart = ref(null);

const updateMemoryUsage = (memory) => {
    memoryChart.value?.setOption({
        xAxis: {
            max: Number.parseInt(totalMemory.value),
            interval: Math.floor(Number.parseInt(totalMemory.value)/16) * 4,
        },
        dataset: {
            source: [['memory', (memory.used_memory+memory.used_swap).toFixed(1), memory.total_memory, memory.total_swap]]
        }
    });
}

const flushMemoryUsage = () => {
    invoke("monitor_memory_info", {}).then(memory => {
        console.log(memory)
        totalMemory.value = (memory.total_memory + memory.total_swap).toFixed(0);
        updateMemoryUsage(memory);
    });
    return flushMemoryUsage;
}

onMounted(async () => {
    memoryChart.value?.setOption(memoryOption);
    setInterval(flushMemoryUsage(), 10 * 1000);
})
</script>

<template>
    <n-row>
        <n-col>
            <n-card>
                <div class="card-header">
                    <span style="width: 40px">内存</span>
                    <v-chart class="chart" ref="memoryChart" :manual-update="true" autoresize />
                    <!-- <span>{{ totalMemory }}GB</span> -->
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
    /* font-weight: bold; */
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--n-text-color);
}
</style>