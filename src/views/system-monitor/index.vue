<script setup>
import { ref, reactive, onMounted, provide, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import 'echarts';
import 'echarts/theme/blue'
import VChart, { THEME_KEY } from 'vue-echarts';
import CpuPanel from "./CpuPanel.vue";
import ProcessPanel from "./ProcessPanel.vue";
import MemoryPanel from "./MemoryPanel.vue";
provide(THEME_KEY, 'blue')

const theme = inject('theme');

const isMonitoring = ref(false);
provide('isMonitoring', isMonitoring);

const toggleMonitoring = (value) => {
    isMonitoring.value = value;
}
</script>

<template>
    <div class="monitor-container">
        <CpuPanel />
        <MemoryPanel />
        <ProcessPanel />
        
        <div v-if="!isMonitoring" class="monitor-mask">
            <div class="mask-content">
                <n-switch v-model:value="isMonitoring" @update:value="toggleMonitoring" size="large"/>
            </div>
        </div>
    </div>
</template>

<style lang="scss" scoped>
.monitor-container {
    position: relative;
    min-height: 400px;
    overflow: hidden;
}

.monitor-mask {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: var(--n-color);
    backdrop-filter: blur(12px);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    overflow: hidden;
}

.mask-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
    padding: 32px;
    background: var(--n-color-modal);
    border-radius: 16px;
    border: 1px solid var(--n-border-color);
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
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
    display: flex;
    justify-content: space-between;
    align-items: center;
    color: var(--n-text-color);
}
</style>