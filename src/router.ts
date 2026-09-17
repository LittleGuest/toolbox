import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
  { path: "/", component: () => import("@/views/Home.vue") },
  { path: "/home", component: () => import("@/views/Home.vue") },
  { path: "/setting", component: () => import("@/views/Setting.vue") },
  { path: "/systemMonitor", component: () => import("@/views/system-monitor/index.vue") },
  {
    path: "/codeSnippet",
    component: () => import("@/views/snippet/CodeSnippet.vue"),
  },
  { path: "/todo", component: () => import("@/views/todo/Todo.vue") },
  {
    path: "/crypto",
    component: () => import("@/views/crypto/SymmetricEncrypt.vue"),
  },
  {
    path: "/color",
    component: () => import("@/views/color/ColorConverter.vue"),
  },
  {
    path: "/regex",
    component: () => import("@/views/regex/RegexVisualizer.vue"),
  },
  {
    path: "/number/bitwise",
    component: () => import("@/views/number/BitwiseCalculator.vue"),
  },

  {
    path: "/transform/filetype",
    component: () => import("@/views/transform/Cffc.vue"),
  },
  {
    path: "/transform/time",
    component: () => import("@/views/transform/Timestamp.vue"),
  },
  {
    path: "/transform/baseconversion",
    component: () => import("@/views/transform/BaseConversion.vue"),
  },
  {
    path: "/transform/cron",
    component: () => import("@/views/transform/Cron.vue"),
  },

  {
    path: "/encodedecode/base",
    component: () => import("@/views/encodedecode/BaseEncoding.vue"),
  },
  {
    path: "/encodedecode/textencode",
    component: () => import("@/views/encodedecode/TextEncoding.vue"),
  },
  {
    path: "/encodedecode/url",
    component: () => import("@/views/encodedecode/URL.vue"),
  },
  {
    path: "/encodedecode/jwt",
    component: () => import("@/views/encodedecode/JWT.vue"),
  },

  {
    path: "/formatter/jsoneditor",
    component: () => import("@/views/formatter/JsonEditor.vue"),
  },
  {
    path: "/formatter/sql",
    component: () => import("@/views/formatter/SqlFormatter.vue"),
  },
  {
    path: "/formatter/xml",
    component: () => import("@/views/formatter/XmlFormatter.vue"),
  },

  {
    path: "/generator/uuid",
    component: () => import("@/views/generator/UUID.vue"),
  },
  {
    path: "/generator/hash",
    component: () => import("@/views/generator/Hash.vue"),
  },

  {
    path: "/database/datafaker",
    component: () => import("@/views/database/datafaker/DatabaseFaker.vue"),
  },
  {
    path: "/database/datafaker/generator",
    name: "DataGenerator",
    // props: true,
    props: (route) => ({ ...route.query }),
    component: () => import("@/views/database/datafaker/DataGenerator.vue"),
  },
  {
    path: "/database/diff",
    component: () => import("@/views/database/diff/DatabaseDiff.vue"),
  },

  {
    path: "/text/markdown",
    component: () => import("@/views/text/Markdown.vue"),
  },
  {
    path: "/text/tools",
    component: () => import("@/views/string/TextTools.vue"),
  },
  {
    path: "/text/diff",
    component: () => import("@/views/diff/TextDiff.vue"),
  },

  {
    path: "/random/string",
    component: () => import("@/views/random/RandomString.vue"),
  },
  {
    path: "/random/number",
    component: () => import("@/views/random/RandomNumber.vue"),
  },
  {
    path: "/random/data",
    component: () => import("@/views/random/RandomData.vue"),
  },

  {
    path: "/network/ip",
    component: () => import("@/views/network/IpConverter.vue"),
  },
  {
    path: "/network/ipmac",
    component: () => import("@/views/network/RandomOther.vue"),
  },

  {
    path: "/graphic/excalidraw",
    component: () => import("@/views/graphic/Excalidraw.vue"),
  },
  {
    path: "/graphic/convert",
    component: () => import("@/views/graphic/ImageConvert.vue"),
  },

  {
    path: "/pdf/images-to-pdf",
    component: () => import("@/views/pdf/ImagesToPdf.vue"),
  },
  {
    path: "/pdf/merge",
    component: () => import("@/views/pdf/PdfMerge.vue"),
  },
  {
    path: "/pdf/edit",
    component: () => import("@/views/pdf/PdfEdit.vue"),
  },
  {
    path: "/pdf/page-number",
    component: () => import("@/views/pdf/PdfPageNumber.vue"),
  },
  {
    path: "/pdf/split",
    component: () => import("@/views/pdf/PdfSplit.vue"),
  },

  {
    path: "/other/qrcode",
    component: () => import("@/views/other/QRCode.vue"),
  },
  {
    path: "/other/clipboard",
    component: () => import("@/views/other/ClipboardManager.vue"),
  },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
