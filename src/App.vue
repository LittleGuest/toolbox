<script setup>
import { ref } from "vue";
import { Message } from "@arco-design/web-vue";
import { IconHome, IconCalendar } from "@arco-design/web-vue/es/icon";

import Hash from "./components/Hash.vue";

const collapsed = ref(false);
const onCollapse = (val, type) => {
  const content = type === "responsive" ? "触发响应式收缩" : "点击触发收缩";
  Message.info({
    content,
    duration: 2000,
  });
  collapsed.value = val;
};

const onClickMenuItem = (key) => {
  Message.info({ content: `You select ${key}`, showIcon: true });
};
</script>

<template>
  <a-layout>
    <a-layout-sider
      theme="dark"
      breakpoint="lg"
      collapsible
      :collapsed="collapsed"
      @collapse="onCollapse"
    >
      <a-menu
        :defaultOpenKeys="['1']"
        :defaultSelectedKeys="['all']"
        @menuItemClick="onClickMenuItem"
      >
        <a-menu-item key="all">
          <IconHome />
          All tools
        </a-menu-item>

        <a-divider direction="horizontal" />

        <a-sub-menu key="convert">
          <template #title>
            <span><IconCalendar />转换器</span>
          </template>
          <a-menu-item key="">JSON {{ "<>" }} YAML</a-menu-item>
          <a-menu-item key="">时间戳</a-menu-item>
          <a-menu-item key="">Number Base</a-menu-item>
          <a-menu-item key="">Cron parser</a-menu-item>
        </a-sub-menu>

        <a-sub-menu key="encoder_decoder">
          <template #title>
            <span><IconCalendar />编码/解码</span>
          </template>
          <a-menu-item key="">HTML</a-menu-item>
          <a-menu-item key="">URL</a-menu-item>
          <a-menu-item key="">Base 64</a-menu-item>
          <a-menu-item key="">JWT Decoder</a-menu-item>
        </a-sub-menu>

        <a-sub-menu key="formatter">
          <template #title>
            <span><IconCalendar />格式化</span>
          </template>
          <a-menu-item key="">JSON</a-menu-item>
          <a-menu-item key="">SQL</a-menu-item>
          <a-menu-item key="">XML</a-menu-item>
          <!-- <a-menu-item key="">YAML</a-menu-item> -->
          <!-- <a-menu-item key="">TOML</a-menu-item> -->
        </a-sub-menu>

        <a-sub-menu key="generator">
          <template #title>
            <span><IconCalendar />生成器</span>
          </template>
          <a-menu-item key="">Hash</a-menu-item>
          <a-menu-item key="">UUID</a-menu-item>
          <a-menu-item key="">Lorem Ipsum</a-menu-item>
          <a-menu-item key="">Checksum</a-menu-item>
        </a-sub-menu>

        <a-sub-menu key="generator">
          <template #title>
            <span><IconCalendar />文本</span>
          </template>
          <a-menu-item key="">Inspector & Case Converter</a-menu-item>
          <a-menu-item key="">Regex Tester</a-menu-item>
          <a-menu-item key="">文本比较</a-menu-item>
          <a-menu-item key="">Markdown预览</a-menu-item>
        </a-sub-menu>

        <a-sub-menu key="graphic">
          <template #title>
            <span><IconCalendar />图像</span>
          </template>
          <a-menu-item key="">PNG /JPEG comperssor</a-menu-item>
        </a-sub-menu>
      </a-menu>
    </a-layout-sider>
    <a-layout>
      <a-breadcrumb>
        <a-breadcrumb-item>Home</a-breadcrumb-item>
      </a-breadcrumb>

      <a-layout-content style="background-color: rgb(185, 175, 175)">
        <Hash
      /></a-layout-content>
    </a-layout>
  </a-layout>
</template>

<style scoped></style>
