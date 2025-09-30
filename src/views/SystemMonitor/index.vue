<script setup>
import { ref, reactive, onMounted, provide } from "vue";
import { invoke } from "@tauri-apps/api/core";
import 'echarts';
import 'echarts/theme/blue'
import VChart, { THEME_KEY } from 'vue-echarts';
import { gaugeOption } from "./options/options.js";
import { setSpecialGuage } from "./options/gaugeOption.js";
import BatteryPanel from "./BatteryPanel.vue";
import CpuPanel from "./CpuPanel.vue";
import ProcessPanel from "./ProcessPanel.vue";
import MemoryPanel from "./MemoryPanel.vue";
import SensorsPanel from "./SensorsPanel.vue";
provide(THEME_KEY, 'blue')

const payloadChart = ref(null);

const flushLoadAvgData = () => {
    invoke("monitor_system_info", {}).then((sysMonitor) => {
        setSpecialGuage(payloadChart.value, "负载", sysMonitor.load_avg.toFixed(2));
    })
    return flushLoadAvgData;
}

onMounted(async () => {
    payloadChart.value?.setOption(gaugeOption);
    setInterval(flushLoadAvgData(), 5000);
})
</script>

<template>
    <CpuPanel />
    <n-row :gutter="8">
        <n-col :span="12">
            <BatteryPanel />
            <MemoryPanel />
        </n-col>
        <n-col :span="6">
            <n-card>
                <template #header>
                    <div class="card-header">
                        <span>系统负载</span>
                    </div>
                </template>
                <v-chart class="chart" ref="payloadChart" :manual-update="true" autoresize />
            </n-card>
        </n-col>
        <n-col :span="6">
            <SensorsPanel />
        </n-col>
    </n-row>
    <ProcessPanel />

</template>

<style lang="scss" scoped>
.chart {
    height: 100px;
    width: 100%;
}

.n-row {
    margin-bottom: 8px;

    &:last-child {
        margin-bottom: 0;
    }
}

:deep() .n-card {
    --n-padding: 8px;
    --n-border-radius: 10px;
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