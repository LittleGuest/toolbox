import { createRouter, createWebHashHistory } from "vue-router";
import All from "./views/All.vue";
import BaseConversion from "./views/transform/BaseConversion.vue";
import Cffc from "./views/transform/Cffc.vue";
import Timestamp from "./views/transform/Timestamp.vue";
import URL from "./views/encodedecode/URL.vue";
import Base64Text from "./views/encodedecode/Base64Text.vue";
import JWT from "./views/encodedecode/JWT.vue";
import JsonEditor from "./views/formatter/JsonEditor.vue";
import SqlFormatter from "./views/formatter/SqlFormatter.vue";
import XmlFormatter from "./views/formatter/XmlFormatter.vue";
import Hash from "./views/generator/Hash.vue";
import UUID from "./views/generator/UUID.vue";
import Checksum from "./views/generator/Checksum.vue";
import Markdown from "./views/text/Markdown.vue";
import IP from "./views/network/IP.vue";
import Setting from "./views/Setting.vue";
import ClipboardManager from "./views/other/ClipboardManager.vue";
import OpenApi from "./views/transform/OpenApi.vue";
import Excalidraw from "./views/graphic/Excalidraw.vue";
import DatabaseFaker from "./views/database/DatabaseFaker.vue";
import DatabaseDiff from "./views/database/DatabaseDiff.vue";
import QRCode from "./views/other/QRCode.vue";

const routes = [
  { path: "/", component: All },
  { path: "/home", component: All },
  { path: "/setting", component: Setting },

  { path: "/transform/filetype", component: Cffc },
  { path: "/transform/time", component: Timestamp },
  { path: "/transform/baseconversion", component: BaseConversion },
  { path: "/transform/openapi", component: OpenApi },

  { path: "/encodedecode/url", component: URL },
  { path: "/encodedecode/base64text", component: Base64Text },
  { path: "/encodedecode/jwt", component: JWT },

  { path: "/formatter/jsoneditor", component: JsonEditor },
  { path: "/formatter/sql", component: SqlFormatter },
  { path: "/formatter/xml", component: XmlFormatter },

  { path: "/generator/hash", component: Hash },
  { path: "/generator/uuid", component: UUID },
  { path: "/generator/checksum", component: Checksum },
  { path: "/database/datafaker", component: DatabaseFaker },
  { path: "/database/diff", component: DatabaseDiff },

  { path: "/text/markdown", component: Markdown },

  { path: "/network/ip", component: IP },

  { path: "/graphic/excalidraw", component: Excalidraw },

  { path: "/other/clipboard", component: ClipboardManager },
  { path: "/other/qrcode", component: QRCode },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
