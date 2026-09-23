<script setup lang="ts">
import { computed, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import {
  FilterRemove,
  TextLineSpacing,
  TextWrap,
  Erase,
  TextNewLine,
  LetterAa,
  Delete,
  Hashtag,
  Html,
  SortAscending,
  SortDescending,
  Shuffle,
  TextAlignLeft,
  Filter,
  Play,
  Repeat,
  AddAlt,
  Undo,
  ArrowsHorizontal,
  ArrowsVertical,
  MagicWand,
  Copy,
  Close,
} from "@vicons/carbon";
import CaseConverter from "./CaseConverter.vue";

const message = useMessage();

const statText = ref("");

const stats = computed(() => {
  const s = statText.value;
  return [
    { label: "字符数", value: [...s].length },
    { label: "单词数", value: s.trim() ? s.trim().split(/\s+/).length : 0 },
    { label: "行数", value: s ? s.split(/\r\n|\r|\n/).length : 0 },
    { label: "非空白字符数", value: s.replace(/\s/g, "").length },
    { label: "字节数 (UTF-8)", value: new TextEncoder().encode(s).length },
    { label: "中文字符数", value: (s.match(/[\u4e00-\u9fff]/g) || []).length },
    { label: "英文字母数", value: (s.match(/[a-zA-Z]/g) || []).length },
    { label: "数字个数", value: (s.match(/[0-9]/g) || []).length },
    { label: "标点符号数", value: (s.match(/[\p{P}]/gu) || []).length },
  ];
});

const copyStat = () => {
  const text = stats.value.map((i) => `${i.label}：${i.value}`).join("\n");
  writeText(text);
  message.success("复制成功");
};

const clearStat = () => {
  statText.value = "";
};

const cleanInput = ref("");
const cleanOutput = ref("");

const cleanApply = (fn: (s: string) => string) => {
  cleanOutput.value = fn(cleanInput.value);
};

const removeDuplicateLines = () =>
  cleanApply((s) => [...new Set(s.split(/\r\n|\r|\n/))].join("\n"));
const removeEmptyLines = () =>
  cleanApply((s) => s.split(/\r\n|\r|\n/).filter((l) => l.trim() !== "").join("\n"));
const collapseWhitespace = () => cleanApply((s) => s.replace(/\s+/g, " "));
const removeAllWhitespace = () => cleanApply((s) => s.replace(/\s/g, ""));
const removeLineBreaks = () => cleanApply((s) => s.replace(/\r\n|\r|\n/g, ""));
const removeDiacritics = () =>
  cleanApply((s) => s.normalize("NFD").replace(/[\u0300-\u036f]/g, ""));
const removePunctuation = () => cleanApply((s) => s.replace(/[\p{P}\p{S}]/gu, ""));
const removeDigits = () => cleanApply((s) => s.replace(/[0-9]/g, ""));
const stripHtml = () => cleanApply((s) => s.replace(/<[^>]*>/g, ""));

const copyClean = () => {
  if (!cleanOutput.value) {
    message.warning("内容为空");
    return;
  }
  writeText(cleanOutput.value);
  message.success("复制成功");
};

const clearClean = () => {
  cleanInput.value = "";
  cleanOutput.value = "";
};

const sortInput = ref("");
const sortOutput = ref("");
const extractSep = ref("");

const sortApply = (fn: (lines: string[]) => string[]) => {
  sortOutput.value = fn(sortInput.value.split(/\r\n|\r|\n/)).join("\n");
};

const sortAsc = () => sortApply((lines) => [...lines].sort((a, b) => a.localeCompare(b)));
const sortDesc = () => sortApply((lines) => [...lines].sort((a, b) => b.localeCompare(a)));
const shuffleLines = () =>
  sortApply((lines) => {
    const arr = [...lines];
    for (let i = arr.length - 1; i > 0; i--) {
      const j = Math.floor(Math.random() * (i + 1));
      [arr[i], arr[j]] = [arr[j], arr[i]];
    }
    return arr;
  });
const trimLines = () => sortApply((lines) => lines.map((l) => l.trim()));

const doExtract = () => {
  if (!extractSep.value.trim()) {
    message.warning("请输入分隔符或正则表达式");
    return;
  }
  let re: RegExp;
  try {
    re = new RegExp(extractSep.value, "g");
  } catch {
    message.error("正则表达式无效");
    return;
  }
  const matches = sortInput.value.match(re) || [];
  sortOutput.value = matches.join("\n");
  if (matches.length === 0) message.info("未匹配到任何内容");
};

const copySort = () => {
  if (!sortOutput.value) {
    message.warning("内容为空");
    return;
  }
  writeText(sortOutput.value);
  message.success("复制成功");
};

const clearSort = () => {
  sortInput.value = "";
  sortOutput.value = "";
  extractSep.value = "";
};

const findInput = ref("");
const findOutput = ref("");
const findValue = ref("");
const replaceValue = ref("");
const isRegex = ref(false);
const repeatCount = ref<number | null>(3);

const doReplace = () => {
  if (!findValue.value) {
    message.warning("请输入查找内容");
    return;
  }
  if (isRegex.value) {
    try {
      findOutput.value = findInput.value.replace(
        new RegExp(findValue.value, "g"),
        replaceValue.value
      );
    } catch {
      message.error("正则表达式无效");
    }
  } else {
    findOutput.value = findInput.value.split(findValue.value).join(replaceValue.value);
  }
};

const doRepeat = () => {
  findOutput.value = findInput.value.repeat(repeatCount.value ?? 1);
};

const copyFind = () => {
  if (!findOutput.value) {
    message.warning("内容为空");
    return;
  }
  writeText(findOutput.value);
  message.success("复制成功");
};

const clearFind = () => {
  findInput.value = "";
  findOutput.value = "";
  findValue.value = "";
  replaceValue.value = "";
  repeatCount.value = 3;
};

const slashInput = ref("");
const slashOutput = ref("");

const slashApply = (fn: (s: string) => string) => {
  slashOutput.value = fn(slashInput.value);
};

const addSlashes = () =>
  slashApply((s) =>
    s.replace(/\\/g, "\\\\").replace(/'/g, "\\'").replace(/"/g, '\\"').replace(/\0/g, "\\0")
  );
const stripSlashes = () => slashApply((s) => s.replace(/\\(.)/g, "$1"));
const reverseString = () => slashApply((s) => [...s].reverse().join(""));
const reverseWords = () =>
  slashApply((s) => s.split(/\s+/).filter(Boolean).reverse().join(" "));

const UPSIDE_DOWN_MAP: Record<string, string> = {
  a: "ɐ", b: "q", c: "ɔ", d: "p", e: "ǝ", f: "ɟ", g: "ƃ", h: "ɥ", i: "ᴉ", j: "ɾ",
  k: "ʞ", l: "l", m: "ɯ", n: "u", o: "o", p: "d", q: "b", r: "ɹ", s: "s", t: "ʇ",
  u: "n", v: "ʌ", w: "ʍ", x: "x", y: "ʎ", z: "z",
  "0": "0", "1": "Ɩ", "2": "ᄅ", "3": "Ɛ", "4": "ㄣ", "5": "ʎ", "6": "9", "7": "ㄥ", "8": "8", "9": "6",
  "?": "¿", "!": "¡", "(": ")", ")": "(", "[": "]", "]": "[", "{": "}", "}": "{",
  "<": ">", ">": "<", ",": "'", "'": ",",
};
const upsideDown = () =>
  slashApply((s) => [...s].map((c) => UPSIDE_DOWN_MAP[c] ?? c).join(""));

const copySlash = () => {
  if (!slashOutput.value) {
    message.warning("内容为空");
    return;
  }
  writeText(slashOutput.value);
  message.success("复制成功");
};

const clearSlash = () => {
  slashInput.value = "";
  slashOutput.value = "";
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <n-tabs type="line" animated>
        <n-tab-pane name="stats" tab="字符统计">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="statText"
              type="textarea"
              :rows="10"
              placeholder="在此输入文本，统计信息将实时更新"
            />
          </div>
          <div class="stat-grid">
            <div v-for="item in stats" :key="item.label" class="stat-card">
              <div class="stat-value">{{ item.value }}</div>
              <div class="stat-label">{{ item.label }}</div>
            </div>
          </div>
          <div class="tb-toolbar">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button size="small" quaternary @click="copyStat">
                  <template #icon><n-icon><Copy /></n-icon></template>
                </n-button>
              </template>
              复制统计
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button size="small" quaternary @click="clearStat">
                  <template #icon><n-icon><Close /></n-icon></template>
                </n-button>
              </template>
              清除
            </n-tooltip>
          </div>
        </n-tab-pane>

        <n-tab-pane name="clean" tab="清理工具">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="cleanInput"
              type="textarea"
              :rows="8"
              placeholder="请输入文本"
            />
          </div>
          <div class="tb-toolbar" style="margin: 12px 0">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="removeDuplicateLines">
                  <template #icon><n-icon><FilterRemove /></n-icon></template>
                </n-button>
              </template>
              删除重复行
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="removeEmptyLines">
                  <template #icon><n-icon><TextLineSpacing /></n-icon></template>
                </n-button>
              </template>
              删除空行
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="collapseWhitespace">
                  <template #icon><n-icon><TextWrap /></n-icon></template>
                </n-button>
              </template>
              合并多余空格
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="removeAllWhitespace">
                  <template #icon><n-icon><Erase /></n-icon></template>
                </n-button>
              </template>
              删除全部空白
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="removeLineBreaks">
                  <template #icon><n-icon><TextNewLine /></n-icon></template>
                </n-button>
              </template>
              删除换行符
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="removeDiacritics">
                  <template #icon><n-icon><LetterAa /></n-icon></template>
                </n-button>
              </template>
              删除重音符号
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="removePunctuation">
                  <template #icon><n-icon><Delete /></n-icon></template>
                </n-button>
              </template>
              删除标点符号
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="removeDigits">
                  <template #icon><n-icon><Hashtag /></n-icon></template>
                </n-button>
              </template>
              删除数字
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="stripHtml">
                  <template #icon><n-icon><Html /></n-icon></template>
                </n-button>
              </template>
              剥离 HTML 标签
            </n-tooltip>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="cleanOutput"
              type="textarea"
              :rows="8"
              placeholder="处理结果"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyClean">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearClean">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="sort" tab="排序与提取">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="sortInput"
              type="textarea"
              :rows="8"
              placeholder="每行一条数据，按行处理"
            />
          </div>
          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="sortAsc">
                  <template #icon><n-icon><SortAscending /></n-icon></template>
                </n-button>
              </template>
              按行升序排序
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="sortDesc">
                  <template #icon><n-icon><SortDescending /></n-icon></template>
                </n-button>
              </template>
              按行降序排序
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="shuffleLines">
                  <template #icon><n-icon><Shuffle /></n-icon></template>
                </n-button>
              </template>
              随机打乱行
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="trimLines">
                  <template #icon><n-icon><TextAlignLeft /></n-icon></template>
                </n-button>
              </template>
              每行去首尾空格
            </n-tooltip>
          </div>
          <n-divider style="margin: 4px 0 16px" />
          <div class="tb-config-row" style="margin-bottom: 16px">
            <div class="tb-config-item">
              <span class="tb-config-label">分隔符 / 正则</span>
              <n-input
                v-model:value="extractSep"
                placeholder="分隔符或正则表达式，如逗号 或 \d+"
                class="tb-mono"
                style="width: 360px"
                clearable
                @keyup.enter="doExtract"
              />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="doExtract">
                    <template #icon><n-icon><Filter /></n-icon></template>
                  </n-button>
                </template>
                提取
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="sortOutput"
              type="textarea"
              :rows="8"
              placeholder="处理结果"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copySort">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearSort">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="find" tab="查找替换与重复">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="findInput"
              type="textarea"
              :rows="7"
              placeholder="请输入文本"
            />
          </div>
          <div class="tb-config-row" style="margin: 16px 0 4px">
            <div class="tb-config-item">
              <n-input v-model:value="findValue" placeholder="查找内容" class="tb-mono" style="width: 220px" />
            </div>
            <div class="tb-config-item">
              <n-input v-model:value="replaceValue" placeholder="替换内容" class="tb-mono" style="width: 220px" />
            </div>
            <div class="tb-config-item">
              <n-checkbox v-model:checked="isRegex">正则</n-checkbox>
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="doReplace">
                    <template #icon><n-icon><Play /></n-icon></template>
                  </n-button>
                </template>
                替换
              </n-tooltip>
            </div>
          </div>
          <n-divider style="margin: 12px 0" />
          <div class="tb-config-row">
            <div class="tb-config-item">
              <n-input-number v-model:value="repeatCount" :min="1" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button @click="doRepeat">
                    <template #icon><n-icon><Repeat /></n-icon></template>
                  </n-button>
                </template>
                重复文本
              </n-tooltip>
              <n-text depth="3" style="font-size: 12px">将上方输入整体重复 N 次</n-text>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="findOutput"
              type="textarea"
              :rows="7"
              placeholder="处理结果"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyFind">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearFind">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="slash" tab="斜线与翻转">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输入</span>
            <n-input
              v-model:value="slashInput"
              type="textarea"
              :rows="8"
              placeholder="请输入文本"
            />
          </div>
          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="addSlashes">
                  <template #icon><n-icon><AddAlt /></n-icon></template>
                </n-button>
              </template>
              添加斜线
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="stripSlashes">
                  <template #icon><n-icon><Undo /></n-icon></template>
                </n-button>
              </template>
              去除斜线
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="reverseString">
                  <template #icon><n-icon><ArrowsHorizontal /></n-icon></template>
                </n-button>
              </template>
              反向字符串
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="reverseWords">
                  <template #icon><n-icon><ArrowsVertical /></n-icon></template>
                </n-button>
              </template>
              单词顺序反转
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="upsideDown">
                  <template #icon><n-icon><MagicWand /></n-icon></template>
                </n-button>
              </template>
              字符倒置
            </n-tooltip>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="slashOutput"
              type="textarea"
              :rows="8"
              placeholder="处理结果"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copySlash">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制输出
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearSlash">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="case" tab="大小写转换"><CaseConverter /></n-tab-pane>
      </n-tabs>
    </section>
  </div>
</template>

<style scoped>
.stat-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 10px;
  margin-top: 14px;
}

.stat-card {
  padding: 12px 14px;
  border: 1px solid var(--tb-border);
  border-radius: 12px;
  background: var(--tb-bg-app);
  text-align: center;
}

.stat-value {
  font-size: 20px;
  font-weight: 700;
  font-family: var(--tb-font-mono);
  color: var(--tb-primary);
}

.stat-label {
  margin-top: 4px;
  font-size: 12px;
  color: var(--tb-text-2);
}
</style>
