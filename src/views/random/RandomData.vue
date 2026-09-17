<script setup lang="ts">
import { ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Play, Shuffle, Copy, Close } from "@vicons/carbon";

const message = useMessage();

const pad2 = (n: number) => String(n).padStart(2, "0");
const randInt = (lo: number, hi: number) => lo + Math.floor(Math.random() * (hi - lo + 1));
const pick = <T,>(arr: T[]): T => arr[Math.floor(Math.random() * arr.length)];

// ---------- 字段数据生成 ----------
const SURNAMES = "赵钱孙李周吴郑王冯陈褚卫蒋沈韩杨朱秦许何吕张孔曹严华金魏陶姜戚谢邹喻柏水窦章云苏潘葛奚范彭郎鲁韦昌马苗凤花方俞任袁柳酆鲍史唐费廉岑薛雷贺倪汤滕殷罗毕郝邬安常乐于时傅皮卞齐康伍余元卜顾孟平黄和穆萧尹姚邵湛汪祁毛禹狄米贝明臧计伏成戴谈宋茅庞熊纪舒屈项祝董梁杜阮蓝闵席季麻强贾路娄危江童颜郭梅盛林刁钟徐邱骆高夏蔡田樊胡凌霍虞万支柯昝管卢莫经房裘缪干解应宗丁宣贲邓郁单杭洪包诸左石崔吉钮龚程嵇邢滑裴陆荣翁荀羊於惠甄麹家封芮羿储靳汲邴糜松井段富巫乌焦巴弓牧隗山谷车侯宓蓬全郗班仰秋仲伊宫宁仇栾暴甘钭厉戎祖武符刘景詹束龙叶幸司韶郜黎蓟薄印宿白怀蒲邰从鄂索咸籍赖卓蔺屠蒙池乔阴欎胥能苍双闻莘党翟谭贡劳逄姬申扶堵冉宰郦雍舄璩桑桂濮牛寿通边扈燕冀郏浦尚农温别庄晏柴瞿阎充慕连茹习宦艾鱼容向古易慎戈廖庾终暨居衡步都耿满弘匡国文寇广禄阙东欧殳沃利蔚越夔隆师巩厍聂晁勾敖融冷訾辛阚那简饶空曾毋沙乜养鞠须丰巢关蒯相查后荆红游竺权逯盖益桓公";
const GIVEN_CHARS = "伟刚勇毅俊峰强军平保东文辉力明永健世广志义兴良海山仁波宁贵福生龙元全国胜学祥才发武新利清飞彬富顺信子杰涛昌成康星光天达安岩中茂进林有坚和彪博诚先敬震振壮会思群豪心邦承乐绍功松善厚庆磊民友裕河哲江超浩亮政谦亨奇固之轮翰朗伯宏言若鸣朋斌梁栋维启克伦翔旭鹏泽晨辰士以建家致树炎德行时泰盛雄琛钧冠策腾楠榕风航弘";
const EMAIL_DOMAINS = ["example.com", "mail.com", "test.org", "demo.net", "qq.com", "163.com", "gmail.com"];
const PHONE_SECONDS = ["3", "5", "7", "8", "9"];
const WORD_CHARS = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";

const chineseName = () => {
  const givenLen = Math.random() < 0.5 ? 1 : 2;
  let g = "";
  for (let i = 0; i < givenLen; i++) g += GIVEN_CHARS[randInt(0, GIVEN_CHARS.length - 1)];
  return SURNAMES[randInt(0, SURNAMES.length - 1)] + g;
};

const randomEmail = () => {
  const user = Array.from({ length: randInt(5, 10) }, () =>
    "abcdefghijklmnopqrstuvwxyz0123456789"[randInt(0, 35)]
  ).join("");
  return `${user}@${pick(EMAIL_DOMAINS)}`;
};

const randomPhone = () =>
  "1" + pick(PHONE_SECONDS) + Array.from({ length: 9 }, () => String(randInt(0, 9))).join("");

const randomDateStr = () => {
  const start = new Date(1970, 0, 1).getTime();
  const end = new Date(2030, 11, 31).getTime();
  const d = new Date(start + Math.floor(Math.random() * (end - start)));
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`;
};

const randomString = () =>
  Array.from({ length: randInt(6, 12) }, () => WORD_CHARS[randInt(0, WORD_CHARS.length - 1)]).join("");

// 按字段名推断类型
const fieldValue = (name: string): string | number | boolean => {
  const n = name.trim().toLowerCase();
  if (n === "name" || n === "username" || n === "author") return chineseName();
  if (n === "age") return randInt(18, 80);
  if (n === "email" || n === "mail") return randomEmail();
  if (n === "phone" || n === "tel" || n === "mobile") return randomPhone();
  if (n === "active" || n === "enabled" || n === "isactive" || n === "status") {
    return Math.random() < 0.5;
  }
  if (n === "joined" || n === "date" || n === "created" || n === "birthday" || n === "time") {
    return randomDateStr();
  }
  if (n === "id" || n === "no" || n === "num" || n === "count") return randInt(1, 99999);
  return randomString();
};

const parseFields = (raw: string): string[] =>
  raw.split(",").map((s) => s.trim()).filter(Boolean);

const buildRow = (fields: string[]): Record<string, string | number | boolean> => {
  const obj: Record<string, string | number | boolean> = {};
  for (const f of fields) obj[f] = fieldValue(f);
  return obj;
};

// ---------- JSON ----------
const jsonFields = ref("name,age,email,phone,active,joined");
const jsonRecords = ref(5);
const jsonOut = ref("");

const genJson = () => {
  const fields = parseFields(jsonFields.value);
  if (!fields.length) {
    message.warning("请填写至少一个字段");
    return;
  }
  const arr = Array.from({ length: jsonRecords.value }, () => buildRow(fields));
  jsonOut.value = JSON.stringify(arr, null, 2);
};

// ---------- XML ----------
const xmlRoot = ref("items");
const xmlFields = ref("name,age,email,phone,active,joined");
const xmlRecords = ref(5);
const xmlOut = ref("");

const xmlEscape = (v: string) =>
  v.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

const genXml = () => {
  const root = xmlRoot.value.trim() || "items";
  const fields = parseFields(xmlFields.value);
  if (!fields.length) {
    message.warning("请填写至少一个字段");
    return;
  }
  const rows = Array.from({ length: xmlRecords.value }, () => buildRow(fields));
  const itemXml = rows
    .map((row) => {
      const inner = fields.map((f) => `    <${f}>${xmlEscape(String(row[f]))}</${f}>`).join("\n");
      return `  <item>\n${inner}\n  </item>`;
    })
    .join("\n");
  xmlOut.value = `<${root}>\n${itemXml}\n</${root}>`;
};

// ---------- CSV / TSV ----------
const csvSeparator = ref<"comma" | "tab">("comma");
const csvColumns = ref("name,age,email");
const csvRows = ref(5);
const csvOut = ref("");

const csvEscape = (v: string | number | boolean): string => {
  const s = String(v);
  if (/[",\t\r\n]/.test(s)) {
    return `"${s.replace(/"/g, '""')}"`;
  }
  return s;
};

const genCsv = () => {
  const sep = csvSeparator.value === "comma" ? "," : "\t";
  const cols = parseFields(csvColumns.value);
  if (!cols.length) {
    message.warning("请填写至少一个列名");
    return;
  }
  const header = cols.map((c) => csvEscape(c)).join(sep);
  const rows = Array.from({ length: csvRows.value }, () =>
    cols.map((c) => csvEscape(fieldValue(c))).join(sep)
  );
  csvOut.value = [header, ...rows].join("\n");
};

// ---------- 正则随机数据 ----------
const regexPattern = ref("[a-z]{5}\\d{2}");
const regexCount = ref(10);
const regexOut = ref("");

const PRINTABLE =
  "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~ \t";
const ANY_CHARS = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
const WS_CHARS = " \t\n\r\f\v";
const MAX_LEN = 20000;

interface ClassPart {
  kind: "set" | "range";
  chars?: string;
  from?: string;
  to?: string;
}

interface RegNode {
  type: "lit" | "any" | "class" | "group";
  lit?: string;
  negated?: boolean;
  parts?: ClassPart[];
  alts?: RegNode[][];
  quant?: { min: number; max: number | null };
}

const notOf = (set: string) => Array.from(PRINTABLE).filter((c) => !set.includes(c)).join("");

const rangeChars = (from: string, to: string): string => {
  let a = from.codePointAt(0) ?? 0;
  let b = to.codePointAt(0) ?? 0;
  if (a > b) [a, b] = [b, a];
  let s = "";
  for (let code = a; code <= b; code++) s += String.fromCodePoint(code);
  return s;
};

const classChars = (node: RegNode): string => {
  let s = "";
  for (const p of node.parts ?? []) {
    if (p.kind === "set") s += p.chars ?? "";
    else s += rangeChars(p.from ?? "", p.to ?? "");
  }
  const deduped = Array.from(new Set(s)).join("");
  if (node.negated) {
    return Array.from(PRINTABLE).filter((c) => !deduped.includes(c)).join("");
  }
  return deduped;
};

class RegexParser {
  private src: string;
  private pos = 0;

  constructor(src: string) {
    this.src = src;
  }

  parse(): RegNode[][] {
    const alts = this.parseAlts();
    if (this.pos < this.src.length) throw new Error("invalid pattern");
    return alts;
  }

  private parseAlts(): RegNode[][] {
    const alts: RegNode[][] = [this.parseSeq()];
    while (this.pos < this.src.length && this.src[this.pos] === "|") {
      this.pos++;
      alts.push(this.parseSeq());
    }
    return alts;
  }

  private parseSeq(): RegNode[] {
    const nodes: RegNode[] = [];
    while (this.pos < this.src.length) {
      const c = this.src[this.pos];
      if (c === "|" || c === ")") break;
      nodes.push(this.parseAtom());
    }
    return nodes;
  }

  private parseAtom(): RegNode {
    const c = this.src[this.pos];
    let node: RegNode;
    if (c === "(") {
      this.pos++;
      if (this.src[this.pos] === "?") {
        if (this.src[this.pos + 1] === ":") {
          this.pos += 2; // 非捕获组
        } else {
          throw new Error("unsupported lookahead");
        }
      }
      const alts = this.parseAlts();
      if (this.pos >= this.src.length || this.src[this.pos] !== ")") {
        throw new Error("unbalanced paren");
      }
      this.pos++;
      node = { type: "group", alts };
    } else if (c === "[") {
      node = this.parseClass();
    } else if (c === "\\") {
      node = this.parseEscape();
    } else if (c === ".") {
      this.pos++;
      node = { type: "any" };
    } else if (c === "^" || c === "$") {
      throw new Error("anchors unsupported");
    } else if (c === "*" || c === "+" || c === "?") {
      throw new Error("lone quantifier");
    } else if (c === "{") {
      if (/^\{(\d+)(?:,(\d*))?\}/.test(this.src.slice(this.pos))) {
        throw new Error("lone quantifier");
      }
      this.pos++;
      node = { type: "lit", lit: "{" };
    } else {
      this.pos++;
      node = { type: "lit", lit: c };
    }
    const q = this.parseQuant();
    if (q) node = { ...node, quant: q };
    return node;
  }

  private parseQuant(): { min: number; max: number | null } | null {
    const c = this.src[this.pos];
    if (c === "*") {
      this.pos++;
      return { min: 0, max: null };
    }
    if (c === "+") {
      this.pos++;
      return { min: 1, max: null };
    }
    if (c === "?") {
      this.pos++;
      return { min: 0, max: 1 };
    }
    if (c === "{") {
      const m = /^\{(\d+)(?:,(\d*))?\}/.exec(this.src.slice(this.pos));
      if (m) {
        this.pos += m[0].length;
        const n = parseInt(m[1], 10);
        if (m[2] === undefined) return { min: n, max: n };
        if (m[2] === "") return { min: n, max: null };
        const mm = parseInt(m[2], 10);
        if (mm < n) throw new Error("bad quantifier range");
        return { min: n, max: mm };
      }
    }
    return null;
  }

  private parseEscape(): RegNode {
    this.pos++; // 消费反斜杠
    if (this.pos >= this.src.length) throw new Error("trailing backslash");
    const c = this.src[this.pos];
    this.pos++;
    switch (c) {
      case "d":
        return { type: "class", parts: [{ kind: "set", chars: "0123456789" }] };
      case "w":
        return { type: "class", parts: [{ kind: "set", chars: WORD_CHARS + "_" }] };
      case "s":
        return { type: "class", parts: [{ kind: "set", chars: WS_CHARS }] };
      case "D":
        return { type: "class", negated: true, parts: [{ kind: "set", chars: "0123456789" }] };
      case "W":
        return { type: "class", negated: true, parts: [{ kind: "set", chars: WORD_CHARS + "_" }] };
      case "S":
        return { type: "class", negated: true, parts: [{ kind: "set", chars: WS_CHARS }] };
      case "n":
        return { type: "lit", lit: "\n" };
      case "t":
        return { type: "lit", lit: "\t" };
      case "r":
        return { type: "lit", lit: "\r" };
      default:
        return { type: "lit", lit: c };
    }
  }

  private parseClass(): RegNode {
    this.pos++; // 消费 [
    let negated = false;
    if (this.src[this.pos] === "^") {
      negated = true;
      this.pos++;
    }
    const parts: ClassPart[] = [];
    let chars: string[] = [];
    const flush = () => {
      if (chars.length) {
        parts.push({ kind: "set", chars: chars.join("") });
        chars = [];
      }
    };
    while (this.pos < this.src.length && this.src[this.pos] !== "]") {
      const c = this.src[this.pos];
      if (c === "\\") {
        this.pos++;
        const e = this.src[this.pos];
        if (e === undefined) throw new Error("bad class escape");
        this.pos++;
        flush();
        if (e === "d") parts.push({ kind: "set", chars: "0123456789" });
        else if (e === "w") parts.push({ kind: "set", chars: WORD_CHARS + "_" });
        else if (e === "s") parts.push({ kind: "set", chars: WS_CHARS });
        else if (e === "D") parts.push({ kind: "set", chars: notOf("0123456789") });
        else if (e === "W") parts.push({ kind: "set", chars: notOf(WORD_CHARS + "_") });
        else if (e === "S") parts.push({ kind: "set", chars: notOf(WS_CHARS) });
        else if (e === "n") parts.push({ kind: "set", chars: "\n" });
        else if (e === "t") parts.push({ kind: "set", chars: "\t" });
        else if (e === "r") parts.push({ kind: "set", chars: "\r" });
        else chars.push(e);
      } else if (
        c === "-" &&
        chars.length > 0 &&
        this.pos + 1 < this.src.length &&
        this.src[this.pos + 1] !== "]"
      ) {
        const from = chars.pop() ?? "";
        this.pos++; // 消费 '-'
        const to = this.src[this.pos];
        this.pos++;
        parts.push({ kind: "range", from, to });
      } else {
        chars.push(c);
        this.pos++;
      }
    }
    if (this.pos >= this.src.length) throw new Error("unbalanced bracket");
    this.pos++; // 消费 ]
    flush();
    return { type: "class", negated, parts };
  }
}

const quantCount = (min: number, max: number | null): number => {
  const lo = Math.min(Math.max(0, min), 10000);
  let hi = max == null ? Math.min(lo + 10, 10000) : Math.min(max, 10000);
  if (hi < lo) hi = lo;
  return lo + Math.floor(Math.random() * (hi - lo + 1));
};

const genFromNode = (node: RegNode, out: string[], left: { n: number }): void => {
  if (left.n <= 0) return;
  const count = node.quant ? quantCount(node.quant.min, node.quant.max) : 1;
  for (let k = 0; k < count; k++) {
    if (left.n <= 0) return;
    if (node.type === "lit") {
      out.push(node.lit ?? "");
      left.n -= node.lit?.length ?? 1;
    } else if (node.type === "any") {
      out.push(ANY_CHARS[Math.floor(Math.random() * ANY_CHARS.length)]);
      left.n--;
    } else if (node.type === "class") {
      const cs = classChars(node);
      if (cs) {
        out.push(cs[Math.floor(Math.random() * cs.length)]);
        left.n--;
      }
    } else if (node.type === "group") {
      const alts = node.alts ?? [[]];
      const alt = alts[Math.floor(Math.random() * alts.length)];
      for (const sub of alt) {
        genFromNode(sub, out, left);
        if (left.n <= 0) return;
      }
    }
  }
};

const generateRegexLine = (alts: RegNode[][]): string => {
  const out: string[] = [];
  const left = { n: MAX_LEN };
  const alt = alts[Math.floor(Math.random() * alts.length)];
  for (const node of alt) {
    genFromNode(node, out, left);
    if (left.n <= 0) break;
  }
  return out.join("");
};

const genRegexData = () => {
  const pattern = regexPattern.value.trim();
  if (!pattern) {
    message.warning("请输入正则表达式");
    return;
  }
  try {
    const alts = new RegexParser(pattern).parse();
    const lines: string[] = [];
    for (let i = 0; i < regexCount.value; i++) {
      lines.push(generateRegexLine(alts));
    }
    regexOut.value = lines.join("\n");
  } catch {
    message.error("暂不支持该正则");
  }
};

// ---------- 文本随机排序 ----------
const shuffleInput = ref("");
const shuffleOut = ref("");

const shuffleLines = () => {
  if (!shuffleInput.value.trim()) {
    message.warning("请输入要打乱的文本");
    return;
  }
  const lines = shuffleInput.value.split("\n");
  for (let i = lines.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [lines[i], lines[j]] = [lines[j], lines[i]];
  }
  shuffleOut.value = lines.join("\n");
};

// ---------- 复制 / 清除 ----------
const outputs = {
  json: jsonOut,
  xml: xmlOut,
  csv: csvOut,
  regex: regexOut,
  shuffle: shuffleOut,
};

const copyOut = (key: keyof typeof outputs) => {
  const v = outputs[key].value;
  if (!v) {
    message.warning("暂无内容可复制");
    return;
  }
  writeText(v);
  message.success("复制成功");
};

const clearOut = (key: keyof typeof outputs) => {
  outputs[key].value = "";
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <n-tabs type="line" animated>
        <n-tab-pane name="json" tab="JSON">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">字段</span>
              <n-input
                v-model:value="jsonFields"
                style="width: 380px"
                placeholder="逗号分隔，按名称推断类型，如 name,age,email,phone,active,joined"
              />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">记录数</span>
              <n-input-number v-model:value="jsonRecords" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genJson">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input v-model:value="jsonOut" type="textarea" :rows="12" placeholder="生成的 JSON 数组" />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('json')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('json')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="xml" tab="XML">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">根元素</span>
              <n-input v-model:value="xmlRoot" style="width: 140px" placeholder="items" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">字段</span>
              <n-input v-model:value="xmlFields" style="width: 340px" placeholder="逗号分隔，同 JSON 推断逻辑" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">记录数</span>
              <n-input-number v-model:value="xmlRecords" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genXml">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input v-model:value="xmlOut" type="textarea" :rows="12" placeholder="生成的 XML 文档" />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('xml')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('xml')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="csv" tab="CSV / TSV">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <n-radio-group v-model:value="csvSeparator">
                <n-radio-button value="comma">逗号分隔</n-radio-button>
                <n-radio-button value="tab">制表符分隔</n-radio-button>
              </n-radio-group>
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">列名</span>
              <n-input v-model:value="csvColumns" style="width: 240px" placeholder="逗号分隔，如 name,age,email" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">行数</span>
              <n-input-number v-model:value="csvRows" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genCsv">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input v-model:value="csvOut" type="textarea" :rows="12" placeholder="生成的数据（首行为表头）" />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('csv')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('csv')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="regex" tab="正则随机数据">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">正则</span>
              <n-input
                v-model:value="regexPattern"
                style="width: 300px"
                placeholder="如 [a-z]{5}\d{2}"
                class="tb-mono"
              />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="regexCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genRegexData">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input v-model:value="regexOut" type="textarea" :rows="12" placeholder="匹配正则的随机字符串（每行一个）" />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('regex')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('regex')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="shuffle" tab="文本随机排序">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="shuffleInput"
              type="textarea"
              :rows="8"
              placeholder="输入多行文本，每行将被随机打乱顺序"
            />
          </div>
          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="shuffleLines">
                  <template #icon>
                    <n-icon><Shuffle /></n-icon>
                  </template>
                </n-button>
              </template>
              打乱
            </n-tooltip>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input v-model:value="shuffleOut" type="textarea" :rows="8" placeholder="打乱行序后的文本" />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('shuffle')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('shuffle')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>
      </n-tabs>
    </section>
  </div>
</template>
