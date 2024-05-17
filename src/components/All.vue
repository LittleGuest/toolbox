<script setup>
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { invoke } from "@tauri-apps/api/tauri";

const router = useRouter();
const route = useRoute();

const all = ref([
  {
    title: "文件格式转换",
    img: "../assets/icon.png",
    route: "/cffc",
  },
  {
    title: "时间戳",
    img: "../assets/icon.png",
    route: "/timestamp",
  },
  {
    title: "数值转换",
    img: "../assets/icon.png",
    route: "/number",
  },
  {
    title: "Cron",
    img: "../assets/icon.png",
    route: "/cron",
  },
  {
    title: "Html",
    img: "../assets/icon.png",
    route: "/html",
  },
  {
    title: "URL",
    img: "../assets/icon.png",
    route: "/url",
  },
  {
    title: "Base64文本",
    img: "../assets/icon.png",
    route: "/base64Text",
  },
  {
    title: "JWT",
    img: "../assets/icon.png",
    route: "/jwt",
  },
  {
    title: "GZip",
    img: "../assets/icon.png",
    route: "/gzip",
  },
  {
    title: "JSON",
    img: "../assets/icon.png",
    route: "/json",
  },
  {
    title: "SQL",
    img: "../assets/icon.png",
    route: "/sql",
  },
  {
    title: "XMl",
    img: "../assets/icon.png",
    route: "/xml",
  },
  {
    title: "Hash",
    img: "../assets/icon.png",
    route: "/hash",
  },
  {
    title: "UUID",
    img: "../assets/icon.png",
    route: "/uuid",
  },
  {
    title: "Lorem Ipsum",
    img: "../assets/icon.png",
    route: "/loremIpsum",
  },
  {
    title: "Checksum",
    img: "../assets/icon.png",
    route: "/checksum",
  },
  {
    title: "Inpector & Case converter",
    img: "../assets/icon.png",
    route: "/caseConverter",
  },
  {
    title: "Regex Tester",
    img: "../assets/icon.png",
    route: "/regexTester",
  },
  {
    title: "文本比较",
    img: "../assets/icon.png",
    route: "/textDiff",
  },
  {
    title: "Markdown预览",
    img: "../assets/icon.png",
    route: "/markdown",
  },
  {
    title: "PNG/JPEG comperssor",
    img: "../assets/icon.png",
    route: "/pngComperssor",
  },
]);

const api = async () => {
  //const value = await invoke("uuid", { hyphens: hyphens.value, uppercase: uppercase.value, version: uuidVersion.value, number: number.value, });
}

const to = (path) => {
  router.push(path);
}

</script>

<template>
  <el-row>
    <el-col v-for="a in   all  " :span="6">
      <el-card shadow="hover" :body-style="{ padding: '0px' }" @click="to(a.route)">
        <img src="../assets/icon.png" class="image" />
        <div style="padding: 14px">
          <span>{{ a.title }}</span>
        </div>
      </el-card>
    </el-col>
  </el-row>
</template>
