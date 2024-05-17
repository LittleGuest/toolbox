import { createRouter, createWebHashHistory } from "vue-router";


import All from "./components/All.vue";

import BaseConversion from "./components/converter/BaseConversion.vue";
import Cffc from "./components/converter/Cffc.vue";
import Timestamp from "./components/converter/Timestamp.vue";
import Cron from "./components/converter/Cron.vue";

import Html from "./components/encodedecode/Html.vue";
import URL from "./components/encodedecode/URL.vue";
import Base64Text from "./components/encodedecode/Base64Text.vue";
import JWT from "./components/encodedecode/JWT.vue";
import GZip from "./components/encodedecode/GZip.vue";

import JsonViewer from "./components/formatter/JsonViewer.vue";
import JsonEditor from "./components/formatter/JsonEditor.vue";
import SqlFormatter from "./components/formatter/SqlFormatter.vue";
import XmlFormatter from "./components/formatter/XmlFormatter.vue";

import Hash from "./components/generator/Hash.vue";
import UUID from "./components/generator/UUID.vue";
import Checksum from "./components/generator/Checksum.vue";

import EscapeUnescape from "./components/text/EscapeUnescape.vue";
import RegexTester from "./components/text/RegexTester.vue";
import Markdown from "./components/text/Markdown.vue";

import IPConverter from "./components/network/IPConvert.vue";

import QRCode from "./components/graphic/QRCode.vue";

import Setting from "./components/Setting.vue";


const routes = [
  { path: "/", component: All },
  { path: "/all", component: All },

  { path: "/cffc", component: Cffc},
  { path: "/timestamp", component: Timestamp },
  { path: "/baseconversion", component: BaseConversion },
  { path: "/cron", component: Cron},

  { path: "/html", component: Html },
  { path: "/url", component: URL },
  { path: "/base64Text", component: Base64Text },
  { path: "/jwt", component: JWT },
  { path: "/gzip", component: GZip },

  { path: "/jsonviewer", component: JsonViewer },
  { path: "/jsoneditor", component: JsonEditor },
  { path: "/sqlformatter", component: SqlFormatter },
  { path: "/xmlformatter", component: XmlFormatter},

  { path: "/hash", component: Hash},
  { path: "/uuid", component: UUID },
  { path: "/checksum", component: Checksum },

  { path: "/escapeunescape", component: EscapeUnescape},
  { path: "/regexTester", component: RegexTester },
  { path: "/markdown", component: Markdown,name:"Markdown预览"},

  { path: "/ipconverter", component: IPConverter },

  { path: "/qrcode", component: QRCode },

  { path: "/setting", component: Setting },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
