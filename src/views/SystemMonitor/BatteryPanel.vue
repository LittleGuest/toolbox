<script setup>
import { ref, reactive, onMounted, provide } from "vue";
import { invoke } from "@tauri-apps/api/core";
import 'echarts';
import 'echarts/theme/blue'
import VChart, { THEME_KEY } from 'vue-echarts';
import { BatteryData } from "./options/monitor.js";
import { batteryOption } from "./options/options.js";
import { NGrid, NGridItem, NCard, NDescriptions, NDescriptionsItem } from 'naive-ui';
provide(THEME_KEY, 'blue')

const battery = ref(null);

const batteryChart = ref(null);

const updateBattery = (batteryData) => {
    batteryChart.value.setOption({
        dataset: {
            source: [['battery', batteryData.percentage, 100]]
        }
    })

    battery.value = batteryData;
}

const flushBattery = () => {
    invoke("monitor_battery_info", {}).then(battery => {
        updateBattery(battery);
    })
    return flushBattery;
}

onMounted(async () => {
    batteryChart.value.setOption(batteryOption);
    setInterval(flushBattery(), 30 * 1000);
})
</script>

<template>
    <n-grid>
        <n-grid-item>
            <n-card>
                <template #header>
                    <div class="card-header">
                        <span>电量</span>

                        <div class="card-header">
                            <v-chart class="chart" style="height: 24px; width: 60%" ref="batteryChart"
                                :manual-update="true" autoresize />
                            <span>{{ battery?.percentage.toFixed(0) }}%</span>
                        </div>
                    </div>
                </template>

                <n-descriptions :column="2" size="small" bordered>
                    <n-descriptions-item label="状态">
                        <span v-if="battery?.state == 1"> 已充满 </span>
                        <span v-if="battery?.state == 2"> 充电中 </span>
                        <span v-if="battery?.state == 3"> 未充电 </span>
                        <span v-if="battery?.state == 0"> 电量用尽 </span>
                        <span v-if="battery?.state == -1"> 未知 </span>
                    </n-descriptions-item>
                    <n-descriptions-item label="健康">
                        {{ battery?.state_of_health }}
                    </n-descriptions-item>
                </n-descriptions>

            </n-card>
        </n-grid-item>
    </n-grid>
</template>

<style lang="scss" scoped>
.chart {
    height: 80px;
    width: 100%;
}

.n-grid {
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