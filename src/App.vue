<script setup>
import { inject } from "vue";
import { useRouter, useRoute } from "vue-router";
import { menus, navigateToMenu } from "./menu.js";
import { NButton } from "naive-ui";
import { Moon, Sun } from "@vicons/carbon";

const router = useRouter();
const route = useRoute();
const theme = inject("theme");
const toggleTheme = inject("toggleTheme");

// 菜单点击事件
const toMenu = navigateToMenu(router);
</script>

<template>
  <n-layout has-sider position="absolute">
    <n-layout-sider collapse-mode="width" :collapsed-width="55" :width="260" show-trigger="arrow-circle" bordered
      :native-scrollbar="false">
      <n-menu :options="menus" default-expand-all="true" @update:value="toMenu" />
    </n-layout-sider>
    <n-layout>
      <!-- <n-layout-header style="height: 30px; padding: 0 24px; display: flex; align-items: center; justify-content: flex-end; border-bottom: 1px solid #e0e0e0">
                <n-button @click="toggleTheme" quaternary circle>
                  <template #icon>
                    <n-icon>
                      <template v-if="theme === 'light'">
                        <Moon />
                      </template>
<template v-else>
                        <Sun />
                      </template>
</n-icon>
</template>
</n-button>
</n-layout-header> -->
      <n-layout-content style="padding: 24px; height: 100%">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<style scoped></style>
