import { createRouter, createWebHashHistory } from "vue-router";

const routes = [
	{ path: "/", component: ()=>import("@/views/All.vue") },
	{ path: "/home", component: ()=>import("@/views/All.vue") },
	{ path: "/setting", component: ()=>import("@/views/Setting.vue") },

	{ path: "/transform/filetype", component: ()=>import("@/views/transform/Cffc.vue") },
	{ path: "/transform/time", component: ()=>import("@/views/transform/Timestamp.vue") },
	{ path: "/transform/baseconversion", component: ()=>import("@/views/transform/BaseConversion.vue") },
	{ path: "/transform/openapi", component: ()=>import("@/views/transform/OpenApi.vue") },

	{ path: "/encodedecode/url", component: ()=>import("@/views/encodedecode/URL.vue") },
	{ path: "/encodedecode/base64text", component: ()=>import("@/views/encodedecode/Base64Text.vue") },
	{ path: "/encodedecode/jwt", component: ()=>import("@/views/encodedecode/JWT.vue") },

	{ path: "/formatter/jsoneditor", component: ()=>import("@/views/formatter/JsonEditor.vue") },
	{ path: "/formatter/sql", component: ()=>import("@/views/formatter/SqlFormatter.vue") },
	{ path: "/formatter/xml", component: ()=>import("@/views/formatter/XmlFormatter.vue") },

	{ path: "/generator/hash", component: ()=>import("@/views/generator/Hash.vue") },
	{ path: "/generator/uuid", component: ()=>import("@/views/generator/UUID.vue") },
	{ path: "/generator/checksum", component: ()=>import("@/views/generator/Checksum.vue") },
	{
		path: "/database/datafaker",
		component: () => import("@/views/database/datafaker/DatabaseFaker.vue"),
	},
	{
		path: "/database/datafaker/generator",
		name: "DataGenerator",
		// props: true,
		props: (route)=>({...route.query}),
		component: () => import("@/views/database/datafaker/DataGenerator.vue"),
	},
	{ path: "/database/diff", component: ()=>import("@/views/database/diff/DatabaseDiff.vue") },

	{ path: "/text/markdown", component: ()=>import("@/views/text/Markdown.vue") },

	{ path: "/network/ip", component: ()=>import("@/views/network/IP.vue") },

	{ path: "/graphic/excalidraw", component: ()=>import("@/views/graphic/Excalidraw.vue") },

	{ path: "/other/clipboard", component: ()=>import("@/views/other/ClipboardManager.vue") },
	{ path: "/other/qrcode", component: ()=>import("@/views/other/QRCode.vue") },
];

const router = createRouter({
	history: createWebHashHistory(),
	routes,
});

export default router;
