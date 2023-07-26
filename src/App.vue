<script setup>
import { ref } from "vue";

import Hash from "./components/generator/Hash.vue";
import UUID from "./components/generator/UUID.vue";

import Html from "./components/encodedecode/Html.vue";
import Base64Text from "./components/encodedecode/Base64Text.vue";
import JWT from "./components/encodedecode/JWT.vue";
import URL from "./components/encodedecode/URL.vue";

import Number from "./components/converter/Number.vue";

import SQL from "./components/formatter/SQL.vue";

import EscapeUnescape from "./components/text/EscapeUnescape.vue";
import RegexTester from "./components/text/RegexTester.vue";

const isCollapse = ref(true);
const handleOpen = (key, keyPath) => {
  console.log(key, keyPath);
};
const handleClose = (key, keyPath) => {
  console.log(key, keyPath);
};
</script>

<template>
  <el-container>
    <el-aside
      ><el-radio-group v-model="isCollapse" style="margin-bottom: 20px">
        <el-radio-button :label="false">expand</el-radio-button>
        <el-radio-button :label="true">collapse</el-radio-button>
      </el-radio-group>
      <el-menu
        default-active="2"
        class="el-menu-vertical-demo"
        :collapse="isCollapse"
        @open="handleOpen"
        @close="handleClose"
      >
        <el-sub-menu index="1">
          <template #title>
            <span>All tools</span>
          </template>
        </el-sub-menu>

        <el-sub-menu index="convert">
          <template #title><span>转换器</span></template>
          <el-menu-item index="1-4-1">JSON {{ "<>" }} YAML</el-menu-item>
          <el-menu-item index="1-4-1">时间戳</el-menu-item>
          <el-menu-item index="1-4-1">Number Base</el-menu-item>
          <el-menu-item index="1-4-1">Cron parser</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="encoder_decoder">
          <template #title><span>编码/解码</span></template>
          <el-menu-item index="1-4-1">HTML</el-menu-item>
          <el-menu-item index="1-4-1">URL</el-menu-item>
          <el-menu-item index="1-4-1">Base 64</el-menu-item>
          <el-menu-item index="1-4-1">JWT Decoder</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="formatter">
          <template #title><span>格式化</span></template>
          <el-menu-item index="1-4-1">JSON</el-menu-item>
          <el-menu-item index="1-4-1">SQL</el-menu-item>
          <el-menu-item index="1-4-1">XML</el-menu-item>
          <!-- <el-menu-item index="1-4-1">YAML</el-menu-item> -->
          <!-- <el-menu-item index="1-4-1">TOML</el-menu-item> -->
        </el-sub-menu>

        <el-sub-menu index="generator">
          <template #title><span>文本</span></template>
          <el-menu-item index="1-4-1">Inspector & Case Converter</el-menu-item>
          <el-menu-item index="1-4-1">Regex Tester</el-menu-item>
          <el-menu-item index="1-4-1">文本比较</el-menu-item>
          <el-menu-item index="1-4-1">Markdown预览</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="graphic">
          <template #title><span>图像</span></template>
          <el-menu-item index="1-4-1">PNG /JPEG comperssor</el-menu-item>
        </el-sub-menu>
      </el-menu></el-aside
    >
    <el-container>
      <el-header>Header</el-header>
      <el-main><SQL /></el-main>
    </el-container>
  </el-container>
</template>

<style scoped>
.el-menu-vertical-demo:not(.el-menu--collapse) {
  width: 200px;
  min-height: 600px;
}
</style>
