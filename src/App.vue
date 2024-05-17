<script setup>
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import {
  Menu as IconMenu,
  Setting,
  Switch,
  Memo,
  Operation,
  Picture,
  ScaleToOriginal,
  Tickets,
  Van,
} from "@element-plus/icons-vue";

const router = useRouter();
const route = useRoute();

const isCollapse = ref(false);
const breadcrumb = ref("");

const changeCollapse = () => {
  isCollapse.value = !isCollapse.value;
};

const change = () => {
  // breadcrumb.value = router.currentRoute.value.name;
};

</script>

<template>
  <el-container style="height: 800px;">
    <el-aside width="170px">
      <el-scrollbar max-height="800px">
        <el-button v-model="isCollapse" @click="changeCollapse">
          <el-icon>
            <Operation />
          </el-icon>
        </el-button>
        <el-menu default-active="all" class="el-menu-vertical-demo" :collapse="isCollapse" router>
          <el-menu-item index="all" route="all">
            <el-icon><icon-menu /></el-icon>
            All tools
          </el-menu-item>

          <el-sub-menu index="convert">
            <template #title>
              <span>
                <el-icon>
                  <Switch />
                </el-icon>
                转换器
              </span>
            </template>
            <el-menu-item index="cffc" route="cffc" @click="change">文件格式转换</el-menu-item>
            <el-menu-item index="timestamp" route="timestamp" @click="change">时间戳</el-menu-item>
            <el-menu-item index="baseconversion" route="baseconversion" @click="change">进制转换</el-menu-item>
            <!-- <el-menu-item index="cron" route="cron" @click="change">Cron</el-menu-item> -->
          </el-sub-menu>

          <el-sub-menu index="encoder_decoder">
            <template #title>
              <span>
                <el-icon>
                  <Tickets />
                </el-icon>
                编码/解码
              </span>
            </template>
            <!-- <el-menu-item index="html" route="html">HTML</el-menu-item> -->
            <el-menu-item index="url" route="url">URL</el-menu-item>
            <el-menu-item index="base64Text" route="base64Text">Base 64</el-menu-item>
            <!-- <el-menu-item index="jwt" route="jwt">JWT Decoder</el-menu-item> -->
            <!-- <el-menu-item index="gzip" route="gzip">GZip</el-menu-item> -->
          </el-sub-menu>

          <el-sub-menu index="formatter">
            <template #title>
              <span>
                <el-icon>
                  <ScaleToOriginal />
                </el-icon>
                格式化
              </span>
            </template>
            <el-menu-item index="jsonviewer" rout="jsonviewer">JSON Viewer</el-menu-item>
            <el-menu-item index="jsoneditor" rout="jsoneditor">JSON Editor</el-menu-item>
            <el-menu-item index="sqlformatter" route="sqlformatter">SQL</el-menu-item>
            <el-menu-item index="xmlformatter" route="xmlformatter">XML</el-menu-item>
            <!-- <el-menu-item index="yaml" route="yaml">YAML</el-menu-item> -->
            <!-- <el-menu-item index="toml" route="toml">TOML</el-menu-item> -->
          </el-sub-menu>

          <el-sub-menu index="generator">
            <template #title>
              <span>
                <el-icon>
                  <Van />
                </el-icon>
                Generators
              </span>
            </template>
            <el-menu-item index="hash" route="hash">Hash</el-menu-item>
            <el-menu-item index="uuid" route="uuid">UUID</el-menu-item>
            <!-- <el-menu-item index="lorem" route="lorem">Lorem Ipsum</el-menu-item> -->
            <!-- <el-menu-item index="checksum" route="checksum">Checksum</el-menu-item> -->
          </el-sub-menu>

          <el-sub-menu index="text">
            <template #title>
              <span>
                <el-icon>
                  <Memo />
                </el-icon>
                文本
              </span>
            </template>
            <!-- <el-menu-item index="inspector" route="inspector">Inspector & Case Converter</el-menu-item> -->
            <!-- <el-menu-item index="regex" route="regex">Regex Tester</el-menu-item> -->
            <!-- <el-menu-item index="text" route="text">文本比较</el-menu-item> -->
            <el-menu-item index="markdown" route="markdown">Markdown预览</el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="network">
            <template #title>
              <span>
                <el-icon>
                  <Picture />
                </el-icon>
                网络
              </span>
            </template>
            <el-menu-item index="ipconverter" route="ipconverter">IP转换器</el-menu-item>
          </el-sub-menu>

          <el-sub-menu index="graphic">
            <template #title>
              <span>
                <el-icon>
                  <Picture />
                </el-icon>
                图像
              </span>
            </template>
            <!-- <el-menu-item index="comperssor" route="comperssor">PNG /JPEG comperssor</el-menu-item> -->
            <!-- <el-menu-item index="qrcode" route="qrcode">QRCode</el-menu-item> -->
          </el-sub-menu>

          <el-menu-item index="setting" route="setting">
            <el-icon>
              <setting />
            </el-icon>
            设置
          </el-menu-item>
        </el-menu>
      </el-scrollbar>
    </el-aside>

    <el-container>
      <el-header>
        {{ breadcrumb }}
      </el-header>
      <el-main>
        <el-scrollbar max-height="800px">
          <router-view></router-view>
        </el-scrollbar>
      </el-main>
    </el-container>
  </el-container>
</template>

<style scoped></style>
