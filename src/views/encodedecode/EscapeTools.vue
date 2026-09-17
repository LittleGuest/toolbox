<script setup lang="ts">
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";

const message = useMessage();

type EscapeMode = "html" | "xml" | "json" | "js" | "java" | "csharp" | "csv" | "sql";

const mode = ref<EscapeMode>("html");

const MODE_OPTIONS: { label: string; value: EscapeMode }[] = [
  { label: "HTML", value: "html" },
  { label: "XML", value: "xml" },
  { label: "JSON", value: "json" },
  { label: "JavaScript", value: "js" },
  { label: "Java", value: "java" },
  { label: "C#", value: "csharp" },
  { label: "CSV", value: "csv" },
  { label: "SQL", value: "sql" },
];

const input = ref("");
const output = ref("");

// ---------------- 通用实体反转义 ----------------
const unescapeEntities = (s: string, map: Record<string, string>): string =>
  s.replace(/&(#x?[0-9a-fA-F]+|[a-zA-Z]+);/g, (m, body: string) => {
    if (body[0] === "#") {
      const hex = body[1] === "x" || body[1] === "X";
      const digits = body.slice(hex ? 2 : 1);
      const cp = hex ? parseInt(digits, 16) : parseInt(digits, 10);
      if (Number.isNaN(cp)) return m;
      try {
        return String.fromCodePoint(cp);
      } catch {
        return m;
      }
    }
    return map[m] ?? m;
  });

// ---------------- HTML ----------------
const htmlEscape = (s: string): string =>
  s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");

const HTML_UNESCAPE: Record<string, string> = {
  "&amp;": "&",
  "&lt;": "<",
  "&gt;": ">",
  "&quot;": '"',
  "&#39;": "'",
  "&apos;": "'",
  "&#x27;": "'",
};

const htmlUnescape = (s: string): string => unescapeEntities(s, HTML_UNESCAPE);

// ---------------- XML ----------------
const xmlEscape = (s: string): string =>
  s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&apos;");

const XML_UNESCAPE: Record<string, string> = {
  "&amp;": "&",
  "&lt;": "<",
  "&gt;": ">",
  "&quot;": '"',
  "&apos;": "'",
};

const xmlUnescape = (s: string): string => unescapeEntities(s, XML_UNESCAPE);

// ---------------- JSON ----------------
const jsonEscape = (s: string): string => {
  let out = "";
  for (const ch of s) {
    const code = ch.charCodeAt(0);
    switch (ch) {
      case '"':
        out += '\\"';
        break;
      case "\\":
        out += "\\\\";
        break;
      case "\n":
        out += "\\n";
        break;
      case "\r":
        out += "\\r";
        break;
      case "\t":
        out += "\\t";
        break;
      case "\b":
        out += "\\b";
        break;
      case "\f":
        out += "\\f";
        break;
      default:
        if (code < 0x20) {
          out += "\\u" + code.toString(16).padStart(4, "0");
        } else {
          out += ch;
        }
    }
  }
  return out;
};

const jsonUnescape = (s: string): string => {
  try {
    return JSON.parse('"' + s + '"') as string;
  } catch {
    throw new Error("无效的 JSON 转义序列");
  }
};

// ---------------- JavaScript / Java / C# ----------------
const jsEscape = (s: string): string => {
  let out = "";
  for (const ch of s) {
    switch (ch) {
      case '"':
        out += '\\"';
        break;
      case "'":
        out += "\\'";
        break;
      case "\\":
        out += "\\\\";
        break;
      case "\n":
        out += "\\n";
        break;
      case "\r":
        out += "\\r";
        break;
      case "\t":
        out += "\\t";
        break;
      case "\b":
        out += "\\b";
        break;
      case "\f":
        out += "\\f";
        break;
      case "\v":
        out += "\\v";
        break;
      case "\0":
        out += "\\0";
        break;
      default:
        out += ch;
    }
  }
  return out;
};

// 通用反转义：处理 \n \r \t \b \f \v \0 \\ \" \' \xHH \uHHHH 与八进制 \NNN（Java）
const jsUnescape = (s: string): string => {
  let out = "";
  let i = 0;
  while (i < s.length) {
    const ch = s[i];
    if (ch !== "\\") {
      out += ch;
      i++;
      continue;
    }
    i++;
    if (i >= s.length) {
      out += "\\";
      break;
    }
    const esc = s[i];
    switch (esc) {
      case "n":
        out += "\n";
        i++;
        break;
      case "r":
        out += "\r";
        i++;
        break;
      case "t":
        out += "\t";
        i++;
        break;
      case "b":
        out += "\b";
        i++;
        break;
      case "f":
        out += "\f";
        i++;
        break;
      case "v":
        out += "\v";
        i++;
        break;
      case "\\":
        out += "\\";
        i++;
        break;
      case '"':
        out += '"';
        i++;
        break;
      case "'":
        out += "'";
        i++;
        break;
      case "x": {
        const hex = s.slice(i + 1, i + 3);
        if (/^[0-9a-fA-F]{2}$/.test(hex)) {
          out += String.fromCharCode(parseInt(hex, 16));
          i += 3;
        } else {
          throw new Error("无效的转义序列 \\x");
        }
        break;
      }
      case "u": {
        const hex = s.slice(i + 1, i + 5);
        if (/^[0-9a-fA-F]{4}$/.test(hex)) {
          out += String.fromCharCode(parseInt(hex, 16));
          i += 5;
        } else {
          throw new Error("无效的转义序列 \\u");
        }
        break;
      }
      default: {
        if (/[0-7]/.test(esc)) {
          let octal = esc;
          let j = i + 1;
          while (j < s.length && /[0-7]/.test(s[j]) && octal.length < 3) {
            octal += s[j];
            j++;
          }
          let val = parseInt(octal, 8);
          if (val > 0xff) val = parseInt(octal.slice(0, 2), 8);
          out += String.fromCharCode(val);
          i = j;
        } else {
          throw new Error(`无效的转义序列 \\${esc}`);
        }
      }
    }
  }
  return out;
};

// ---------------- CSV ----------------
const csvEscapeField = (field: string): string => {
  if (/[",\n\r\t]/.test(field)) {
    return '"' + field.replace(/"/g, '""') + '"';
  }
  return field;
};

const parseCsvLine = (line: string): string[] => {
  const fields: string[] = [];
  let field = "";
  let inQuotes = false;
  let i = 0;
  while (i < line.length) {
    const ch = line[i];
    if (inQuotes) {
      if (ch === '"') {
        if (line[i + 1] === '"') {
          field += '"';
          i += 2;
        } else {
          inQuotes = false;
          i++;
        }
      } else {
        field += ch;
        i++;
      }
    } else if (ch === '"') {
      inQuotes = true;
      i++;
    } else if (ch === ",") {
      fields.push(field);
      field = "";
      i++;
    } else {
      field += ch;
      i++;
    }
  }
  fields.push(field);
  return fields;
};

const csvEscape = (s: string): string => s.split(/\r?\n/).map(csvEscapeField).join(",");

const csvUnescape = (s: string): string => parseCsvLine(s).join("\n");

// ---------------- SQL ----------------
const sqlEscape = (s: string): string => s.replace(/'/g, "''");
const sqlUnescape = (s: string): string => s.replace(/''/g, "'");

// ---------------- 主操作 ----------------
const doEscape = (): void => {
  if (!input.value) return;
  try {
    switch (mode.value) {
      case "html":
        output.value = htmlEscape(input.value);
        break;
      case "xml":
        output.value = xmlEscape(input.value);
        break;
      case "json":
        output.value = jsonEscape(input.value);
        break;
      case "js":
      case "java":
      case "csharp":
        output.value = jsEscape(input.value);
        break;
      case "csv":
        output.value = csvEscape(input.value);
        break;
      case "sql":
        output.value = sqlEscape(input.value);
        break;
    }
  } catch (e) {
    message.error(String(e));
  }
};

const doUnescape = (): void => {
  if (!input.value) return;
  try {
    switch (mode.value) {
      case "html":
        output.value = htmlUnescape(input.value);
        break;
      case "xml":
        output.value = xmlUnescape(input.value);
        break;
      case "json":
        output.value = jsonUnescape(input.value);
        break;
      case "js":
      case "java":
      case "csharp":
        output.value = jsUnescape(input.value);
        break;
      case "csv":
        output.value = csvUnescape(input.value);
        break;
      case "sql":
        output.value = sqlUnescape(input.value);
        break;
    }
  } catch (e) {
    message.error(String(e));
  }
};

// ---------------- 辅助操作 ----------------
const pasteInput = async () => {
  try {
    input.value = await readText();
  } catch {}
};

const pasteOutput = async () => {
  try {
    output.value = await readText();
  } catch {}
};

const copy = (value: string) => {
  if (!value) return;
  writeText(value);
  message.success("复制成功");
};

const clear = () => {
  input.value = "";
  output.value = "";
};
</script>

<template>
  <div>
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">模式</span>
          <n-select v-model:value="mode" :options="MODE_OPTIONS" style="width: 260px" />
        </div>
      </div>

      <!-- 输入区 -->
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输入</span>
        <n-input
          v-model:value="input"
          type="textarea"
          :rows="10"
          placeholder="请输入要转义 / 反转义的内容"
        />
      </div>

      <!-- 主操作行 -->
      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="doEscape">
              <template #icon><n-icon><ArrowDown /></n-icon></template>
            </n-button>
          </template>
          转义
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="doUnescape">
              <template #icon><n-icon><ArrowUp /></n-icon></template>
            </n-button>
          </template>
          反转义
        </n-tooltip>
      </div>

      <!-- 输出区 -->
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输出</span>
        <n-input
          v-model:value="output"
          type="textarea"
          :rows="10"
          placeholder="转换结果将显示在这里"
        />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="pasteInput">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴输入
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="pasteOutput">
                <template #icon><n-icon><Paste /></n-icon></template>
              </n-button>
            </template>
            粘贴输出
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="copy(output)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制输出
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="clear">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            清除
          </n-tooltip>
        </div>
      </div>
  </div>
</template>
