<script setup>
import App from "./App.vue";
</script>

<template>
    <n-message-provider placement="top-right">
        <n-loading-bar-provider>
            <App />
        </n-loading-bar-provider>
    </n-message-provider>
</template>
