<script setup>
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  Document,
  Menu as IconMenu,
  Location,
  Setting,
} from "@element-plus/icons-vue";

const router = useRouter();
const route = useRoute();

const isCollapse = ref(false);
const changeCollapse = () => {
  isCollapse.value = !isCollapse.value;
};


const goo = (path) => {
  router.push(path);
};

</script>

<template>
  <el-container>
    <el-aside>
      <el-button v-model="isCollapse" v-on:click="changeCollapse">==</el-button>
      <el-menu default-active="all" class="el-menu-vertical-demo" :collapse="isCollapse">
        <el-sub-menu index="all">
          <template #title>
            <span>All tools</span>
          </template>
        </el-sub-menu>

        <el-sub-menu index="convert">
          <template #title><span>转换器</span></template>
          <el-menu-item index="json">JSON {{ "<>" }} YAML</el-menu-item>
          <el-menu-item index="timestamp" v-on:click="goo('timestamp')">时间戳</el-menu-item>
          <el-menu-item index="number" v-on:lick="goo('number')">Number Base</el-menu-item>
          <el-menu-item index="cron" v-on:click="goo('cron')">Cron parser</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="encoder_decoder">
          <template #title><span>编码/解码</span></template>
          <el-menu-item index="html" v-on:click="goo('html')">HTML</el-menu-item>
          <el-menu-item index="url" v-on:click="goo('url')">URL</el-menu-item>
          <el-menu-item index="base64Text" v-on:click="goo('base64Text')">Base 64</el-menu-item>
          <el-menu-item index="jwt" v-on:click="goo('jwt')">JWT Decoder</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="formatter">
          <template #title><span>格式化</span></template>
          <el-menu-item index="json" v-on:click="goo('json')">JSON</el-menu-item>
          <el-menu-item index="sql" v-on:click="goo('sql')">SQL</el-menu-item>
          <el-menu-item index="xml" v-on:click="goo('xml')">XML</el-menu-item>
          <!-- <el-menu-item index="yaml"v-on:click="goo('yaml')">YAML</el-menu-item> -->
          <!-- <el-menu-item index="toml"v-on:click="goo('toml')">TOML</el-menu-item> -->
        </el-sub-menu>

        <el-sub-menu index="generator">
          <template #title><span>Generators</span></template>
          <el-menu-item index="hash" v-on:click="goo('hash')">Hash</el-menu-item>
          <el-menu-item index="uuid" v-on:click="goo('uuid')">UUID</el-menu-item>
          <el-menu-item index="lorem" v-on:click="goo('lorem')">Lorem Ipsum</el-menu-item>
          <el-menu-item index="checksum" v-on:click="goo('checksum')">Checksum</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="text">
          <template #title><span>文本</span></template>
          <el-menu-item index="inspector" v-on:click="goo('inspector')">Inspector & Case Converter</el-menu-item>
          <el-menu-item index="regex" v-on:click="goo('regex')">Regex Tester</el-menu-item>
          <el-menu-item index="text" v-on:click="goo('text')">文本比较</el-menu-item>
          <el-menu-item index="markdown" v-on:click="goo('markdown')">Markdown预览</el-menu-item>
        </el-sub-menu>

        <el-sub-menu index="graphic">
          <template #title><span>图像</span></template>
          <el-menu-item index="comperssor" v-on:click="goo('comperssor')">PNG /JPEG comperssor</el-menu-item>
        </el-sub-menu>
      </el-menu></el-aside>
    <el-container>
      <el-header>Header</el-header>
      <el-main>
        <router-view></router-view>
      </el-main>
    </el-container>
  </el-container>
</template>

<style scoped></style>
