<script setup>
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { NIcon } from "naive-ui";
import { Home, LetterUu, Link, NetworkPublic, Sql, TextUnderline, Xml, Time, Json, Image, Barcode, DataFormat, TextItalic, DocumentExport, ToolKit, DataBase, DataStructured, CdCreateExchange, QrCode } from "@vicons/carbon";
import { Binary, Clipboard, File, Hash, Markdown } from "@vicons/tabler";
import { TransformFilled } from "@vicons/material";

const router = useRouter();
const route = useRoute();


const all = ref([
  {
    label: "文件格式转换",
    key: "/transform/filetype",
    icon: File,
  },
  {
    label: "时间戳",
    key: "/transform/time",
    icon: Time,
  },
  {
    label: "进制转换",
    key: "/transform/baseconversion",
    icon: Binary,
  },
  {
    label: "OpenApi",
    key: "/transform/openapi",
    icon: DocumentExport,
  },
  {
    label: "Base64",
    key: "/encodedecode/base64text",
    icon: TextUnderline,
  },
  {
    label: "URL",
    key: "/encodedecode/url",
    icon: Link,
  },
  {
    label: "JSON Editor",
    key: "/formatter/jsoneditor",
    icon: Json,
  },
  {
    label: "SQL",
    key: "/formatter/sql",
    icon: Sql,
  },
  {
    label: "XML",
    key: "/formatter/xml",
    icon: Xml,
  },
  {
    label: "Hash",
    key: "/generator/hash",
    icon: Hash,
  },
  {
    label: "UUID",
    key: "/generator/uuid",
    icon: LetterUu,
  },
  {
    label: "数据生成",
    key: "/database/datafaker",
    icon: DataStructured,
  },
  {
    label: "Markdown",
    key: "/text/markdown",
    icon: Markdown,
  },
  {
    label: "IP",
    key: "/network/ip",
    icon: NetworkPublic,
  },
  {
    label: "Excalidraw",
    key: "/graphic/excalidraw",
    icon: NetworkPublic,
  },
  {
    label: "剪切板",
    key: "/other/clipboard",
    icon: Clipboard,
  },
  {
    label: "二维码",
    key: "/other/qrcode",
    icon: QrCode,
  },
  {
    label: "Excalidraw",
    key: "https://excalidraw.com/",
    icon: Link,
    external: true,
  },
]);

const to = (path) => {
  router.push(path);
};
</script>

<template>
  <n-flex>
    <n-card hoverable size="small" v-for="a in all" :title="a.label" @click="to(a.key)">
      <template #cover>
        <n-icon size="100" :component="a.icon" />
      </template>
    </n-card>
  </n-flex>
</template>

<style lang="scss" scoped>
.n-card {
  max-width: 150px;
}
</style>
