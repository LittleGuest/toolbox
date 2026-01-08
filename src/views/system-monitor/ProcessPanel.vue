<script setup>
import { ref, onMounted, provide, inject, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import 'echarts';
import 'echarts/theme/blue'
import VChart, { THEME_KEY } from 'vue-echarts';
import { processOption } from "./options/options.js";
provide(THEME_KEY, 'blue')

const isMonitoring = inject('isMonitoring');

const processChart = ref(null);
let processInterval = null;

const updateProcessMem = (processes) => {
    const source = []
    for (const key in processes) {
        source.push([processes[key].pid, processes[key].name, (processes[key].memory * 1024.).toFixed(0)])
    }
    processChart.value?.setOption({
        dataset: {
            source: source
        }
    });
}

const flushProcessData = () => {
    if (!isMonitoring.value) return;
    invoke("monitor_process_info", {}).then(processes => {
        updateProcessMem(processes.slice(0, 10).reverse())
    })
}

const startProcessMonitoring = () => {
    if (processInterval) return;
    flushProcessData();
    processInterval = setInterval(flushProcessData, 10 * 1000);
}

const stopProcessMonitoring = () => {
    if (processInterval) {
        clearInterval(processInterval);
        processInterval = null;
    }
}

watch(isMonitoring, (newValue) => {
    if (newValue) {
        startProcessMonitoring();
    } else {
        stopProcessMonitoring();
    }
})

onMounted(async () => {
    processChart.value?.setOption(processOption);
    if (isMonitoring.value) {
        startProcessMonitoring();
    }
})
</script>

<template>
    <n-row>
        <n-col :span="24">
            <n-card>
                <v-chart class="chart" ref="processChart" :manual-update="true" autoresize />
            </n-card>
        </n-col>
    </n-row>
</template>

<style lang="scss" scoped>
.chart {
    height: 290px;
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