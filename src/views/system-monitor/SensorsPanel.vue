<script setup>
import { ref, reactive, onMounted, provide, inject, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import 'echarts';
import 'echarts/theme/blue'
import VChart, { THEME_KEY } from 'vue-echarts';
import { gaugeOption } from "./options/options.js";
import { setSpecialGuage } from "./options/gaugeOption.js";
provide(THEME_KEY, 'blue')

const isMonitoring = inject('isMonitoring');

const socSensorChart = ref(null);
let sensorInterval = null;

const flushSensorData = () => {
    if (!isMonitoring.value) return;
    invoke("monitor_system_info", {}).then(sysMonitor => {
        const socTemp = sysMonitor.sensors['SOC MTR Temp Sensor0'];
        setSpecialGuage(socSensorChart.value, '温度', socTemp==null?0:socTemp.toFixed(2),);
    })
}

const startSensorMonitoring = () => {
    if (sensorInterval) return;
    flushSensorData();
    sensorInterval = setInterval(flushSensorData, 10 * 1000);
}

const stopSensorMonitoring = () => {
    if (sensorInterval) {
        clearInterval(sensorInterval);
        sensorInterval = null;
    }
}

watch(isMonitoring, (newValue) => {
    if (newValue) {
        startSensorMonitoring();
    } else {
        stopSensorMonitoring();
    }
})

onMounted(async () => {
    socSensorChart.value?.setOption(gaugeOption);
    if (isMonitoring.value) {
        startSensorMonitoring();
    }
})
</script>

<template>
    <n-card>
        <template #header>
            <div class="card-header">
                <span>SOC温度</span>
            </div>
        </template>
        <v-chart class="chart" ref="socSensorChart" :manual-update="true" autoresize />
    </n-card>
</template>

<style lang="scss" scoped>
.chart {
    height: 100px;
    width: 100%;
}

.n-row:last-child {
    margin-bottom: 0;
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