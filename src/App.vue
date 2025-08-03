<script setup>
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { NIcon } from "naive-ui";
import { Home, LetterUu, Link, NetworkPublic, Sql, TextUnderline, Xml, Time, Json, Image, Barcode, DataFormat, TextItalic, DocumentExport, ToolKit, DataBase, DataStructured, CdCreateExchange, QrCode } from "@vicons/carbon";
import { Binary, Clipboard, File, Hash, Markdown } from "@vicons/tabler";
import { TransformFilled } from "@vicons/material";

const router = useRouter();
const route = useRoute();

const renderIcon = (icon) => {
  return () => h(NIcon, null, { default: () => h(icon) });
};

const menuOptions = [
  {
    label: "首页",
    key: "/home",
    icon: renderIcon(Home),
    closable: true,
  },
  {
    label: "转换",
    key: "/transform",
    icon: renderIcon(TransformFilled),
    children: [
      {
        label: "文件格式转换",
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
      },
      {
        label: "OpenApi",
        key: "/transform/openapi",
        icon: renderIcon(DocumentExport),
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
    icon: renderIcon(CdCreateExchange),
    children: [
      {
        label: "UUID",
        key: "/generator/uuid",
        icon: renderIcon(LetterUu),
      },
      {
        label: "文本Hash",
        key: "/generator/hash",
        icon: renderIcon(Hash),
      },
      {
        label: "文件校验",
        key: "/generator/checksum",
        icon: renderIcon(File),
      }
    ]
  },
  {
    label: "数据库",
    key: "/database",
    icon: renderIcon(DataBase),
    children: [
      {
        label: "假数据生成",
        key: "/database/datafaker",
        icon: renderIcon(DataStructured),
      },
      {
        label: "数据库差异",
        key: "/database/diff",
        icon: renderIcon(DataStructured),
      },
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
  // {
  //   label: "网络",
  //   key: "/network",
  //   icon: renderIcon(NetworkPublic),
  //   children: [
  //     {
  //       label: "IP",
  //       key: "/network/ip",
  //       icon: renderIcon(NetworkPublic),
  //     }
  //   ]
  // },
  // {
  //   label: "图像",
  //   key: "/image",
  //   icon: renderIcon(Image),
  //   children: [
  //     {
  //       label: "Excalidraw",
  //       key: "/graphic/excalidraw",
  //       icon: renderIcon(NetworkPublic),
  //     }
  //   ]
  // },
  // {
  //   label: "其它",
  //   key: "/other",
  //   icon: renderIcon(ToolKit),
  //   children: [
  //     {
  //       label: "剪切板",
  //       key: "/other/clipboard",
  //       icon: renderIcon(Clipboard),
  //     },
  //     {
  //       label: "二维码",
  //       key: "/other/qrcode",
  //       icon: renderIcon(QrCode),
  //     }
  //   ]
  // },
  {
    label: "外链",
    key: "external",
    icon: renderIcon(Link),
    children: [
      {
        label: "Excalidraw",
        key: "https://excalidraw.com/",
        icon: renderIcon(Link),
        external: true,
      },
      {
        label: "正则可视化",
        key: "https://regex-vis.com/",
        icon: renderIcon(Link),
        external: true,
      },
      {
        label: "正则测试",
        key: "https://regex101.com/",
        icon: renderIcon(Link),
        external: true,
      }
    ]
  },
];

const activeTab = ref();
const openTabs = ref([
  menuOptions[0]
]);

const id = () => {
  return new Date().getTime() + Math.random().toString(36);
};

const handleMenuChange = (key, item) => {
  if (item.external) {
    window.open(key);
    return;
  }
  if (!(key === '/home' || key === '')) {
    openTabs.value.push({ ...item, id: id() });
  }
  router.push(key);
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
  <!-- <KeepAlive :include="openTabs"> -->
  <n-layout has-sider position="absolute">
    <n-layout-sider collapse-mode="width" :collapsed-width="120" :width="260" show-trigger="arrow-circle"
      content-style="padding: 24px;" bordered :native-scrollbar="false">
      <n-menu :options="menuOptions" v-model:value="activeTab" default-expand-all="true"
        @update:value="handleMenuChange" />
    </n-layout-sider>
    <n-layout>
      <n-layout-header bordered>
        <n-tabs v-model:value="activeTab" type="card" tab-style="min-width:80px" @close="handleTabClose"
          @update:value="handleTabChange">
          <n-tab-pane display-directive="show:lazy" :closable="!tab.closable" v-for="tab in openTabs" :key="tab.key"
            :tab="tab.label" :name="tab.id" />
        </n-tabs>
      </n-layout-header>
      <n-layout-content content-style="padding: 24px;">
        <router-view />
      </n-layout-content>
    </n-layout>
  </n-layout>
  <!-- </KeepAlive> -->
</template>


<style scoped></style>
