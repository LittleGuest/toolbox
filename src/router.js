import { createRouter, createWebHashHistory } from "vue-router";

import Hash from "./components/generator/Hash.vue";
import UUID from "./components/generator/UUID.vue";

import Html from "./components/encodedecode/Html.vue";
import Base64Text from "./components/encodedecode/Base64Text.vue";
import JWT from "./components/encodedecode/JWT.vue";
import URL from "./components/encodedecode/URL.vue";

import Number from "./components/converter/Number.vue";

import SQL from "./components/formatter/SQL.vue";

import EscapeUnescape from "./components/text/EscapeUnescape.vue";
import RegexTester from "./components/text/RegexTester.vue";






const routes = [
  { path: "/", component: Hash },
  { path: "/hash", component: Hash },
  { path: "/uuid", component: UUID },
  { path: "/html", component: Html },
  { path: "/base64Text", component: Base64Text },
  { path: "/jwt", component: JWT },
  { path: "/url", component: URL },
  { path: "/number", component:Number },
  { path: "/sql", component: SQL },
  { path: "/escapeunescape", component: EscapeUnescape},
  { path: "/regexTester", component: RegexTester },
];

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

export default router;
