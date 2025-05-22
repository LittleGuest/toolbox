import { createRouter, createWebHashHistory } from "vue-router";
import All from "./components/All.vue";
import BaseConversion from "./components/transform/BaseConversion.vue";
import Cffc from "./components/transform/Cffc.vue";
import Timestamp from "./components/transform/Timestamp.vue";
import URL from "./components/encodedecode/URL.vue";
import Base64Text from "./components/encodedecode/Base64Text.vue";
import JWT from "./components/encodedecode/JWT.vue";
import JsonEditor from "./components/formatter/JsonEditor.vue";
import SqlFormatter from "./components/formatter/SqlFormatter.vue";
import XmlFormatter from "./components/formatter/XmlFormatter.vue";
import Hash from "./components/generator/Hash.vue";
import UUID from "./components/generator/UUID.vue";
import Checksum from "./components/generator/Checksum.vue";
import Markdown from "./components/text/Markdown.vue";
import IP from "./components/network/IP.vue";
import Setting from "./components/Setting.vue";
import ClipboardManager from "./components/other/ClipboardManager.vue";

const routes = [
	{ path: "/", component: All },
	{ path: "/home", component: All },
	{ path: "/setting", component: Setting },

	{ path: "/transform/filetype", component: Cffc },
	{ path: "/transform/time", component: Timestamp },
	{ path: "/transform/baseconversion", component: BaseConversion },

	{ path: "/encodedecode/url", component: URL },
	{ path: "/encodedecode/base64text", component: Base64Text },
	{ path: "/encodedecode/jwt", component: JWT },

	{ path: "/formatter/jsoneditor", component: JsonEditor },
	{ path: "/formatter/sqlformatter", component: SqlFormatter },
	{ path: "/formatter/xmlformatter", component: XmlFormatter },

	{ path: "/generator/hash", component: Hash },
	{ path: "/generator/uuid", component: UUID },
	{ path: "/generator/checksum", component: Checksum },

	{ path: "/text/markdown", component: Markdown },

	{ path: "/network/ip", component: IP },

	{ path: "/other/clipboard", component: ClipboardManager },
];

const router = createRouter({
	history: createWebHashHistory(),
	routes,
});

export default router;
