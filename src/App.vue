<script setup>
import { useRouter, useRoute } from "vue-router";
import { menus, navigateToMenu } from "./menu.js";

const router = useRouter();
const route = useRoute();

// 菜单点击事件
const toMenu = navigateToMenu(router);
</script>

<template>
  <n-layout has-sider position="absolute">
    <n-layout-sider
      collapse-mode="width"
      :collapsed-width="55"
      :width="260"
      show-trigger="arrow-circle"
      bordered
      :native-scrollbar="false"
    >
      <n-menu
        :options="menus"
        default-expand-all="true"
        @update:value="toMenu"
      />
    </n-layout-sider>
    <n-layout>
      <n-layout-content style="padding: 24px; height: 100%">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>

<style scoped></style>
