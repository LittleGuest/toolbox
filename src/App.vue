<script setup>
import { ref } from "vue";
import { useRouter, useRoute, RouterLink } from "vue-router";
import { NIcon } from "naive-ui";
import { Home, LetterUu, Link, NetworkPublic, Number2, Sql, TextUnderline, Xml, Time, Json, Image } from "@vicons/carbon";
import { File, Hash, Markdown } from "@vicons/tabler";

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
  },
  {
    label: "转换",
    key: "/transform",
    icon: renderIcon(Home),
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
        icon: renderIcon(Number2),
      }
    ]
  },
  {
    label: "编码/解码",
    key: "/encodedecode",
    icon: renderIcon(Home),
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
    icon: renderIcon(Home),
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
    icon: renderIcon(Home),
    children: [
      {
        label: "Markdown预览",
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
];

const activeTab = ref();
const openTabs = ref([]);


const handleMenuChange = (key, item) => {
  router.push(key);
  openTabs.value.push(item);
};

const handleTabChange = (key) => {
  router.push(key);
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
  <n-layout has-sider>
    <n-layout-sider collapse-mode="width" :collapsed-width="120" :width="240" show-trigger="arrow-circle"
      content-style="padding: 24px;" bordered>
      <n-menu :options="menuOptions" v-model:value="activeTab" default-expand-all="true"
        @update:value="handleMenuChange" />
    </n-layout-sider>
    <n-layout-content content-style="padding: 24px;">
      <n-tabs v-model:value="activeTab" type="card" closable tab-style="min-width:80px" @close="handleTabClose"
        @update:value="handleTabChange">
        <n-tab-pane v-for="tab in openTabs" :key="tab.key" :tab="tab.label" :name="tab.key">
        </n-tab-pane>
      </n-tabs>

      <router-view />
    </n-layout-content>
  </n-layout>


  <!-- <el-container style="height: 800px"> -->
  <!--   <el-aside width="170px"> -->
  <!--     <el-scrollbar max-height="800px"> -->
  <!--       <el-button v-model="isCollapse" @click="changeCollapse"> -->
  <!--         <el-icon> -->
  <!--           <Operation /> -->
  <!--         </el-icon> -->
  <!--       </el-button> -->
  <!--       <el-menu default-active="all" class="el-menu-vertical-demo" :collapse="isCollapse" router> -->
  <!--         <el-menu-item index="all" route="all"> -->
  <!--           <el-icon><icon-menu /></el-icon> -->
  <!--           All tools -->
  <!--         </el-menu-item> -->
  <!---->
  <!--         <el-sub-menu index="convert"> -->
  <!--           <template #title> -->
  <!--             <span> -->
  <!--               <el-icon> -->
  <!--                 <Switch /> -->
  <!--               </el-icon> -->
  <!--               转换器 -->
  <!--             </span> -->
  <!--           </template> -->
  <!--           <el-menu-item index="cffc" route="cffc" @click="change">文件格式转换</el-menu-item> -->
  <!--           <el-menu-item index="timestamp" route="timestamp" @click="change">时间戳</el-menu-item> -->
  <!--           <el-menu-item index="baseconversion" route="baseconversion" @click="change">进制转换</el-menu-item> -->
  <!--         </el-sub-menu> -->
  <!---->
  <!--         <el-sub-menu index="encoder_decoder"> -->
  <!--           <template #title> -->
  <!--             <span> -->
  <!--               <el-icon> -->
  <!--                 <Tickets /> -->
  <!--               </el-icon> -->
  <!--               编码/解码 -->
  <!--             </span> -->
  <!--           </template> -->
  <!--           <el-menu-item index="url" route="url">URL</el-menu-item> -->
  <!--           <el-menu-item index="base64Text" route="base64Text">Base 64</el-menu-item> -->
  <!--         </el-sub-menu> -->
  <!---->
  <!--         <el-sub-menu index="formatter"> -->
  <!--           <template #title> -->
  <!--             <span> -->
  <!--               <el-icon> -->
  <!--                 <ScaleToOriginal /> -->
  <!--               </el-icon> -->
  <!--               格式化 -->
  <!--             </span> -->
  <!--           </template> -->
  <!--           <el-menu-item index="jsonviewer" rout="jsonviewer">JSON Viewer</el-menu-item> -->
  <!--           <el-menu-item index="jsoneditor" rout="jsoneditor">JSON Editor</el-menu-item> -->
  <!--           <el-menu-item index="sqlformatter" route="sqlformatter">SQL</el-menu-item> -->
  <!--           <el-menu-item index="xmlformatter" route="xmlformatter">XML</el-menu-item> -->
  <!--         </el-sub-menu> -->
  <!---->
  <!--         <el-sub-menu index="generator"> -->
  <!--           <template #title> -->
  <!--             <span> -->
  <!--               <el-icon> -->
  <!--                 <Van /> -->
  <!--               </el-icon> -->
  <!--               Generators -->
  <!--             </span> -->
  <!--           </template> -->
  <!--           <el-menu-item index="hash" route="hash">Hash</el-menu-item> -->
  <!--           <el-menu-item index="uuid" route="uuid">UUID</el-menu-item> -->
  <!--         </el-sub-menu> -->
  <!---->
  <!--         <el-sub-menu index="text"> -->
  <!--           <template #title> -->
  <!--             <span> -->
  <!--               <el-icon> -->
  <!--                 <Memo /> -->
  <!--               </el-icon> -->
  <!--               文本 -->
  <!--             </span> -->
  <!--           </template> -->
  <!--           <el-menu-item index="markdown" route="markdown">Markdown预览</el-menu-item> -->
  <!--         </el-sub-menu> -->
  <!---->
  <!--         <el-sub-menu index="network"> -->
  <!--           <template #title> -->
  <!--             <span> -->
  <!--               <el-icon> -->
  <!--                 <Picture /> -->
  <!--               </el-icon> -->
  <!--               网络 -->
  <!--             </span> -->
  <!--           </template> -->
  <!--           <el-menu-item index="ipconverter" route="ipconverter">IP转换器</el-menu-item> -->
  <!--         </el-sub-menu> -->
  <!---->
  <!--         <el-sub-menu index="graphic"> -->
  <!--           <template #title> -->
  <!--             <span> -->
  <!--               <el-icon> -->
  <!--                 <Picture /> -->
  <!--               </el-icon> -->
  <!--               图像 -->
  <!--             </span> -->
  <!--           </template> -->
  <!--         </el-sub-menu> -->
  <!---->
  <!--         <el-menu-item index="setting" route="setting"> -->
  <!--           <el-icon> -->
  <!--             <setting /> -->
  <!--           </el-icon> -->
  <!--           设置 -->
  <!--         </el-menu-item> -->
  <!--       </el-menu> -->
  <!--     </el-scrollbar> -->
  <!--   </el-aside> -->
  <!---->
  <!--   <el-container> -->
  <!--     <el-header> -->
  <!--       {{ breadcrumb }} -->
  <!--     </el-header> -->
  <!--     <el-main> -->
  <!--       <el-scrollbar max-height="800px"> -->
  <!--         <router-view></router-view> -->
  <!--       </el-scrollbar> -->
  <!--     </el-main> -->
  <!--   </el-container> -->
  <!-- </el-container> -->
</template>


<style scoped></style>
