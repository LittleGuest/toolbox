import { createRouter, createWebHashHistory } from "vue-router";

import BaseConversion from "./components/converter/BaseConversion.vue";
import Cffc from "./components/converter/Cffc.vue";
import Cron from "./components/converter/Cron.vue";
import Timestamp from "./components/converter/Timestamp.vue";

import Checksum from "./components/generator/Checksum.vue";
import Hash from "./components/generator/Hash.vue";
import UUID from "./components/generator/UUID.vue";

import Html from "./components/encodedecode/Html.vue";
import Base64Text from "./components/encodedecode/Base64Text.vue";
import JWT from "./components/encodedecode/JWT.vue";
import URL from "./components/encodedecode/URL.vue";
import GZip from "./components/encodedecode/GZip.vue";

import JsonViewer from "./components/formatter/JsonViewer.vue";
import JsonEditor from "./components/formatter/JsonEditor.vue";
import SQL from "./components/formatter/SQL.vue";

import QRCode from "./components/graphic/QRCode.vue";

import EscapeUnescape from "./components/text/EscapeUnescape.vue";
import Markdown from "./components/text/Markdown.vue";
import RegexTester from "./components/text/RegexTester.vue";

import All from "./components/All.vue";
import Setting from "./components/Setting.vue";


const routes = [
  { path: "/", component: All },
  { path: "/all", component: All },
  { path: "/cffc", component: Cffc,name:"格式转换" },
  { path: "/baseconversion", component: BaseConversion ,name:"进制转换"},
  { path: "/cron", component: Cron,name:"Cron表达式" },
  { path: "/timestamp", component: Timestamp,name:"时间戳" },
  { path: "/html", component: Html },
  { path: "/url", component: URL },
  { path: "/base64Text", component: Base64Text },
  { path: "/jwt", component: JWT },
  { path: "/gzip", component: GZip },
  { path: "/sql", component: SQL },
  { path: "/jsonviewer", component: JsonViewer,name:"JSON Viewer" },
  { path: "/jsoneditor", component: JsonEditor,name:"JSON Editor" },
  { path: "/hash", component: Hash },
  { path: "/uuid", component: UUID },
  { path: "/checksum", component: Checksum },
  { path: "/escapeunescape", component: EscapeUnescape},
  { path: "/markdown", component: Markdown,name:"Markdown预览"},
  { path: "/regexTester", component: RegexTester },
  { path: "/setting", component: Setting },
  { path: "/qrcode", component: QRCode },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
