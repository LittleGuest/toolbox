<script setup>
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { NIcon, createDiscreteApi } from "naive-ui";
import { Home, LetterUu, Link, NetworkPublic, Sql, TextUnderline, Xml, Time, Json, Image, Barcode, DataFormat, TextItalic } from "@vicons/carbon";
import { Binary, File, Hash, Markdown } from "@vicons/tabler";
import { TransformFilled } from "@vicons/material";
import { MoreOutlined } from "@vicons/antd";

const router = useRouter();
const route = useRoute();

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi(["message", "dialog", "notification", "loadingBar", "modal"]);


const renderIcon = (icon) => {
  return () => h(NIcon, null, { default: () => h(icon) });
};

const menuOptions = [
  {
    label: "首页",
    key: "/home",
    icon: renderIcon(Home),
  },
  {
    label: "转换",
    key: "/transform",
    icon: renderIcon(TransformFilled),
    children: [
      {
        label: "文件格式",
        key: "/transform/filetype",
        icon: renderIcon(File),
      },
      {
        label: "时间戳",
        key: "/transform/time",
        icon: renderIcon(Time),
      },
      {
        label: "进制转换",
        key: "/transform/baseconversion",
        icon: renderIcon(Binary),
      }
    ]
  },
  {
    label: "编码/解码",
    key: "/encodedecode",
    icon: renderIcon(Barcode),
    children: [
      {
        label: "Base64",
        key: "/encodedecode/base64text",
        icon: renderIcon(TextUnderline),
      },
      {
        label: "URL",
        key: "/encodedecode/url",
        icon: renderIcon(Link),
      },
    ]
  },
  {
    label: "格式化",
    key: "/formatter",
    icon: renderIcon(DataFormat),
    children: [
      {
        label: "JSON Editor",
        key: "/formatter/jsoneditor",
        icon: renderIcon(Json),
      },
      {
        label: "SQL",
        key: "/formatter/sql",
        icon: renderIcon(Sql),
      },
      {
        label: "XML",
        key: "/formatter/xml",
        icon: renderIcon(Xml),
      },
    ]
  },
  {
    label: "生成器",
    key: "/generator",
    icon: renderIcon(Home),
    children: [
      {
        label: "Hash",
        key: "/generator/hash",
        icon: renderIcon(Hash),
      },
      {
        label: "UUID",
        key: "/generator/uuid",
        icon: renderIcon(LetterUu),
      }
    ]
  },
  {
    label: "文本",
    key: "/text",
    icon: renderIcon(TextItalic),
    children: [
      {
        label: "Markdown",
        key: "/text/markdown",
        icon: renderIcon(Markdown),
      }
    ]
  },
  {
    label: "网络",
    key: "/network",
    icon: renderIcon(NetworkPublic),
    children: [
      {
        label: "IP",
        key: "/network/ip",
        icon: renderIcon(NetworkPublic),
      }
    ]
  },
  {
    label: "图像",
    key: "/image",
    icon: renderIcon(Image),
    children: [
    ]
  },
  {
    label: "外链",
    key: "external",
    icon: renderIcon(Link),
    children: [
    ]
  },
  {
    label: "其它",
    key: "/other",
    icon: renderIcon(MoreOutlined),
    children: [
      {
        label: "剪切板",
        key: "/other/clipboard",
        icon: renderIcon(NetworkPublic),
      }
    ]
  },
];

const activeTab = ref();
const openTabs = ref([]);

const id = () => {
  return new Date().getTime() + Math.random().toString(36);
};


const handleMenuChange = (key, item) => {
  router.push(key);
  openTabs.value.push({ ...item, id: id() });
};

const handleTabChange = (key) => {
  const index = openTabs.value.findIndex((tab) => tab.id === key);
  router.push(openTabs.value[index].key);
};

const handleTabClose = (key) => {
  if (openTabs.value.length === 1) {
    return;
  }
  const index = openTabs.value.findIndex((tab) => tab.key === key);
  openTabs.value.splice(index, 1);
  activeTab.value = openTabs.value[openTabs.value.length - 1].key;
  router.push(activeTab.value);
};

</script>

<template>
  <n-layout has-sider position="absolute">
    <n-layout-sider collapse-mode="width" :collapsed-width="120" :width="260" show-trigger="arrow-circle"
      content-style="padding: 24px;" bordered :native-scrollbar="false">
      <n-menu :options="menuOptions" v-model:value="activeTab" default-expand-all="true"
        @update:value="handleMenuChange" />
    </n-layout-sider>
    <n-layout>
      <n-layout-header bordered>
        <n-tabs v-model:value="activeTab" type="card" closable tab-style="min-width:80px" @close="handleTabClose"
          @update:value="handleTabChange">
          <n-tab-pane v-for="tab in openTabs" :key="tab.key" :tab="tab.label" :name="tab.id" />
        </n-tabs>
      </n-layout-header>
      <n-layout-content content-style="padding: 24px;">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
</template>


<style scoped></style>
