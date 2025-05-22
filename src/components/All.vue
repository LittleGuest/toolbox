<script setup>
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";

const router = useRouter();
const route = useRoute();

const all = ref([
  {
    title: "文件格式转换",
    img: "../assets/vue.svg",
    route: "/cffc",
  },
  {
    title: "时间戳",
    img: "../assets/vue.svg",
    route: "/timestamp",
  },
  {
    title: "数值转换",
    img: "../assets/vue.svg",
    route: "/number",
  },
  {
    title: "Cron",
    img: "../assets/vue.svg",
    route: "/cron",
  },
  {
    title: "Html",
    img: "../assets/vue.svg",
    route: "/html",
  },
  {
    title: "URL",
    img: "../assets/vue.svg",
    route: "/url",
  },
  {
    title: "Base64文本",
    img: "../assets/vue.svg",
    route: "/base64Text",
  },
  {
    title: "JWT",
    img: "../assets/vue.svg",
    route: "/jwt",
  },
  {
    title: "GZip",
    img: "../assets/vue.svg",
    route: "/gzip",
  },
  {
    title: "JSON",
    img: "../assets/vue.svg",
    route: "/json",
  },
  {
    title: "SQL",
    img: "../assets/vue.svg",
    route: "/sql",
  },
  {
    title: "XMl",
    img: "../assets/vue.svg",
    route: "/xml",
  },
  {
    title: "Hash",
    img: "../assets/vue.svg",
    route: "/hash",
  },
  {
    title: "UUID",
    img: "../assets/vue.svg",
    route: "/uuid",
  },
  {
    title: "Lorem Ipsum",
    img: "../assets/vue.svg",
    route: "/loremIpsum",
  },
  {
    title: "Checksum",
    img: "../assets/vue.svg",
    route: "/checksum",
  },
  {
    title: "Inpector & Case converter",
    img: "../assets/vue.svg",
    route: "/caseConverter",
  },
  {
    title: "Regex Tester",
    img: "../assets/vue.svg",
    route: "/regexTester",
  },
  {
    title: "文本比较",
    img: "../assets/vue.svg",
    route: "/textDiff",
  },
  {
    title: "Markdown预览",
    img: "../assets/vue.svg",
    route: "/markdown",
  },
  {
    title: "PNG/JPEG comperssor",
    img: "../assets/vue.svg",
    route: "/pngComperssor",
  },
]);

const api = async () => {
  //const value = await invoke("uuid", { hyphens: hyphens.value, uppercase: uppercase.value, version: uuidVersion.value, number: number.value, });
};

const to = (path) => {
  router.push(path);
};
</script>

<template>
  <el-row>
    <el-col v-for="a in all" :span="6">
      <el-card shadow="hover" :body-style="{ padding: '0px' }" @click="to(a.route)">
        <img src="../assets/vue.svg" class="image" />
        <div style="padding: 14px">
          <span>{{ a.title }}</span>
        </div>
      </el-card>
    </el-col>
  </el-row>
</template>
