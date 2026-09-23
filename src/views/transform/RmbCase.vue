<script setup lang="ts">
import { ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Paste, Close, Erase, ArrowsHorizontal, CurrencyYen } from "@vicons/carbon";

const message = useMessage();

const yuanUnit = ref<"元" | "圆">("元");
const zhengUnit = ref<"整" | "正">("整");
const jiaoZheng = ref(false);

interface RmbParseResult {
  amount: string;
  grouped: string;
  upper: string;
}

// ---------------- 数字 → 大写 ----------------
const amountInput = ref("");
const upperResult = ref("");
const upperError = ref("");

const amountExamples = ["1,234.56", "6007.14", "16409.02", "0.56", "1000000.00"];
let amountIndex = 0;

const convertToUpper = async () => {
  const value = amountInput.value.trim();
  if (!value) {
    upperResult.value = "";
    upperError.value = "";
    return;
  }
  try {
    upperResult.value = await invoke<string>("rmb_to_upper", {
      input: value,
      yuan: yuanUnit.value,
      zheng: zhengUnit.value,
      jiaoZheng: jiaoZheng.value,
    });
    upperError.value = "";
  } catch (error) {
    upperResult.value = "";
    upperError.value = String(error);
  }
};

// 元位 / 结尾 / 角后加整变化后，已输入的金额需要重新换算
watch([yuanUnit, zhengUnit, jiaoZheng], convertToUpper);

const fillAmount = () => {
  amountInput.value = amountExamples[amountIndex % amountExamples.length];
  amountIndex += 1;
  convertToUpper();
};

const pasteAmount = async () => {
  try {
    amountInput.value = (await readText()) ?? "";
    convertToUpper();
  } catch {
    message.warning("读取剪贴板失败");
  }
};

const clearAmount = () => {
  amountInput.value = "";
  upperResult.value = "";
  upperError.value = "";
};

// ---------------- 大写 → 数字 ----------------
const upperInput = ref("");
const parseError = ref("");
const parsed = ref<RmbParseResult | null>(null);

const upperExamples = [
  "壹仟肆佰零玖元伍角",
  "壹万陆仟肆佰零玖元零贰分",
  "壹亿元人民币整",
  "人民币叁佰贰拾壹元零伍分",
];
let upperIndex = 0;

const parseUpper = async () => {
  const value = upperInput.value.trim();
  if (!value) {
    parsed.value = null;
    parseError.value = "";
    return;
  }
  try {
    parsed.value = await invoke<RmbParseResult>("rmb_to_amount", {
      input: value,
      yuan: yuanUnit.value,
      zheng: zhengUnit.value,
    });
    parseError.value = "";
  } catch (error) {
    parsed.value = null;
    parseError.value = String(error);
  }
};

const fillUpper = () => {
  upperInput.value = upperExamples[upperIndex % upperExamples.length];
  upperIndex += 1;
  parseUpper();
};

const pasteUpper = async () => {
  try {
    upperInput.value = (await readText()) ?? "";
    parseUpper();
  } catch {
    message.warning("读取剪贴板失败");
  }
};

const clearUpper = () => {
  upperInput.value = "";
  parsed.value = null;
  parseError.value = "";
};

// ---------------- 通用操作 ----------------
const copyText = (value?: string | null) => {
  if (!value) {
    message.warning("内容为空");
    return;
  }
  writeText(value);
  message.success("复制成功");
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">元位</span>
          <n-radio-group v-model:value="yuanUnit" size="small">
            <n-radio-button value="元">元</n-radio-button>
            <n-radio-button value="圆">圆</n-radio-button>
          </n-radio-group>
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">结尾</span>
          <n-radio-group v-model:value="zhengUnit" size="small">
            <n-radio-button value="整">整</n-radio-button>
            <n-radio-button value="正">正</n-radio-button>
          </n-radio-group>
        </div>
        <div class="tb-config-item">
          <n-checkbox size="small" v-model:checked="jiaoZheng">
            角位金额后加「{{ zhengUnit }}」
          </n-checkbox>
        </div>
      </div>

      <n-tabs type="line" animated>
        <!-- 数字 → 大写 -->
        <n-tab-pane name="toUpper" tab="数字 → 大写">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">金额</span>
              <n-input
                class="rmb-amount-input"
                placeholder="如 1234.56、¥1,234.56"
                clearable
                v-model:value="amountInput"
                @keyup.enter="convertToUpper"
              />
            </div>
          </div>

          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="convertToUpper">
                  <template #icon
                    ><n-icon><ArrowsHorizontal /></n-icon
                  ></template>
                </n-button>
              </template>
              转换
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="fillAmount">
                  <template #icon
                    ><n-icon><Erase /></n-icon
                  ></template>
                </n-button>
              </template>
              填入示例
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="pasteAmount">
                  <template #icon
                    ><n-icon><Paste /></n-icon
                  ></template>
                </n-button>
              </template>
              粘贴金额
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="copyText(upperResult)">
                  <template #icon
                    ><n-icon><Copy /></n-icon
                  ></template>
                </n-button>
              </template>
              复制大写
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="clearAmount">
                  <template #icon
                    ><n-icon><Close /></n-icon
                  ></template>
                </n-button>
              </template>
              清除
            </n-tooltip>
          </div>

          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">大写金额</span>
            <n-input
              readonly
              type="textarea"
              :rows="5"
              placeholder="转换后的大写金额"
              v-model:value="upperResult"
            />
          </div>
          <n-text v-if="upperError" type="error" class="rmb-error">{{ upperError }}</n-text>
        </n-tab-pane>

        <!-- 大写 → 数字 -->
        <n-tab-pane name="toAmount" tab="大写 → 数字">
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">中文大写金额</span>
            <n-input
              v-model:value="upperInput"
              type="textarea"
              :rows="5"
              placeholder="如：壹仟贰佰叁拾肆元伍角陆分（支持壹/一、元/圆、整/正等写法）"
            />
          </div>

          <div class="tb-action-row">
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button type="primary" @click="parseUpper">
                  <template #icon
                    ><n-icon><ArrowsHorizontal /></n-icon
                  ></template>
                </n-button>
              </template>
              转换
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="fillUpper">
                  <template #icon
                    ><n-icon><Erase /></n-icon
                  ></template>
                </n-button>
              </template>
              填入示例
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="pasteUpper">
                  <template #icon
                    ><n-icon><Paste /></n-icon
                  ></template>
                </n-button>
              </template>
              粘贴
            </n-tooltip>
            <n-tooltip trigger="hover">
              <template #trigger>
                <n-button @click="clearUpper">
                  <template #icon
                    ><n-icon><Close /></n-icon
                  ></template>
                </n-button>
              </template>
              清除
            </n-tooltip>
          </div>

          <template v-if="parsed">
            <span class="tb-editor-label">解析结果</span>
            <div class="rmb-list">
              <div class="rmb-row">
                <span class="rmb-name">数字金额</span>
                <n-input readonly v-model:value="parsed.amount" class="rmb-value" />
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button quaternary circle size="small" @click="copyText(parsed.amount)">
                      <template #icon
                        ><n-icon><Copy /></n-icon
                      ></template>
                    </n-button>
                  </template>
                  复制
                </n-tooltip>
              </div>
              <div class="rmb-row">
                <span class="rmb-name">千分位</span>
                <n-input readonly v-model:value="parsed.grouped" class="rmb-value" />
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button quaternary circle size="small" @click="copyText(parsed.grouped)">
                      <template #icon
                        ><n-icon><Copy /></n-icon
                      ></template>
                    </n-button>
                  </template>
                  复制
                </n-tooltip>
              </div>
              <div class="rmb-row">
                <span class="rmb-name">规范大写</span>
                <n-input readonly v-model:value="parsed.upper" class="rmb-value" />
                <n-tooltip trigger="hover">
                  <template #trigger>
                    <n-button quaternary circle size="small" @click="copyText(parsed.upper)">
                      <template #icon
                        ><n-icon><Copy /></n-icon
                      ></template>
                    </n-button>
                  </template>
                  复制
                </n-tooltip>
              </div>
            </div>
          </template>
          <n-text v-if="parseError" type="error" class="rmb-error">{{ parseError }}</n-text>
        </n-tab-pane>
      </n-tabs>
    </section>
  </div>
</template>

<style scoped>
.rmb-amount-input {
  width: 260px;
}

.rmb-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
}

.rmb-row {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.rmb-name {
  flex-shrink: 0;
  width: 88px;
  font-size: 13px;
  color: var(--tb-text-2);
}

.rmb-value {
  flex: 1;
  min-width: 0;
}

.rmb-error {
  display: block;
  margin-top: 10px;
  font-size: 12.5px;
}
</style>
