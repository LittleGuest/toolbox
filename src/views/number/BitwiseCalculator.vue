<script setup lang="ts">
import { ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy } from "@vicons/carbon";

const message = useMessage();

type Base = "dec" | "hex" | "bin";
type Op = "and" | "or" | "xor" | "nand" | "nor" | "xnor" | "not" | "shl" | "shr";

const baseOptions: { label: string; value: Base }[] = [
  { label: "十进制", value: "dec" },
  { label: "十六进制", value: "hex" },
  { label: "二进制", value: "bin" },
];

const aInput = ref("");
const aBase = ref<Base>("dec");
const bInput = ref("");
const bBase = ref<Base>("dec");
const bits = ref<number>(32);
const shift = ref<number | null>(1);

const OPERATIONS: { key: Op; symbol: string; label: string; needsB: boolean }[] = [
  { key: "and", symbol: "&", label: "AND", needsB: true },
  { key: "or", symbol: "|", label: "OR", needsB: true },
  { key: "xor", symbol: "^", label: "XOR", needsB: true },
  { key: "nand", symbol: "~&", label: "NAND", needsB: true },
  { key: "nor", symbol: "~|", label: "NOR", needsB: true },
  { key: "xnor", symbol: "~^", label: "XNOR", needsB: true },
  { key: "not", symbol: "~", label: "NOT(A)", needsB: false },
  { key: "shl", symbol: "<<", label: "左移", needsB: false },
  { key: "shr", symbol: ">>", label: "右移", needsB: false },
];

interface CalcResult {
  expr: string;
  decimal: string;
  hex: string;
  binary: string;
}

const result = ref<CalcResult | null>(null);

const copy = (value: string) => {
  if (!value) return;
  writeText(value);
  message.success("复制成功");
};

const parseOperand = (text: string, base: Base, width: number): bigint | null => {
  const s = text.trim();
  if (!s) return null;
  let v: bigint;
  if (base === "hex") {
    const h = s.replace(/^0x/i, "");
    if (!/^[0-9a-fA-F]+$/.test(h)) return null;
    v = BigInt("0x" + h);
  } else if (base === "bin") {
    const b = s.replace(/^0b/i, "");
    if (!/^[01]+$/.test(b)) return null;
    v = BigInt("0b" + b);
  } else {
    if (!/^-?\d+$/.test(s)) return null;
    v = BigInt(s);
  }
  const max = 1n << BigInt(width);
  if (v < 0n) {
    v = v + max; // 负数按补码取模
  }
  if (v < 0n || v >= max) return null;
  return v;
};

const calc = (op: Op) => {
  const width = bits.value;
  const mask = (1n << BigInt(width)) - 1n;
  const a = parseOperand(aInput.value, aBase.value, width);
  if (a === null) {
    message.error("操作数无效");
    result.value = null;
    return;
  }
  const info = OPERATIONS.find((o) => o.key === op)!;
  let b: bigint | null = null;
  if (info.needsB) {
    b = parseOperand(bInput.value, bBase.value, width);
    if (b === null) {
      message.error("操作数无效");
      result.value = null;
      return;
    }
  }
  const n = Math.max(1, shift.value ?? 1);
  let value: bigint;
  let expr = "";
  switch (op) {
    case "and":
      value = a & b!;
      expr = "A & B";
      break;
    case "or":
      value = a | b!;
      expr = "A | B";
      break;
    case "xor":
      value = a ^ b!;
      expr = "A ^ B";
      break;
    case "nand":
      value = ~(a & b!) & mask;
      expr = "~(A & B)";
      break;
    case "nor":
      value = ~(a | b!) & mask;
      expr = "~(A | B)";
      break;
    case "xnor":
      value = ~(a ^ b!) & mask;
      expr = "~(A ^ B)";
      break;
    case "not":
      value = a ^ mask;
      expr = "~A";
      break;
    case "shl":
      value = (a << BigInt(n)) & mask;
      expr = `A << ${n}`;
      break;
    case "shr":
      value = a >> BigInt(n);
      expr = `A >> ${n}`;
      break;
  }
  value &= mask;
  result.value = {
    expr,
    decimal: value.toString(),
    hex: "0x" + value.toString(16).padStart(Math.ceil(width / 4), "0"),
    binary: value.toString(2).padStart(width, "0"),
  };
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">位宽</span>
          <n-radio-group v-model:value="bits">
            <n-radio-button :value="8">8 位</n-radio-button>
            <n-radio-button :value="16">16 位</n-radio-button>
            <n-radio-button :value="32">32 位</n-radio-button>
            <n-radio-button :value="64">64 位</n-radio-button>
          </n-radio-group>
        </div>
      </div>

      <div class="tb-config-row">
        <div class="tb-config-item" style="flex: 1; min-width: 220px">
          <span class="tb-config-label">A</span>
          <n-input
            v-model:value="aInput"
            clearable
            class="tb-mono"
            placeholder="如 255、0xFF、0b1111"
            @keyup.enter="calc('and')"
          />
        </div>
        <div class="tb-config-item">
          <n-select v-model:value="aBase" :options="baseOptions" style="width: 110px" />
        </div>
      </div>

      <div class="tb-config-row">
        <div class="tb-config-item" style="flex: 1; min-width: 220px">
          <span class="tb-config-label">B</span>
          <n-input
            v-model:value="bInput"
            clearable
            class="tb-mono"
            placeholder="单目运算（NOT、移位）可留空"
          />
        </div>
        <div class="tb-config-item">
          <n-select v-model:value="bBase" :options="baseOptions" style="width: 110px" />
        </div>
      </div>

      <div class="ops-wrap">
        <n-space :size="8" wrap>
          <n-tooltip v-for="op in OPERATIONS" :key="op.key" trigger="hover">
            <template #trigger>
              <n-button type="primary" @click="calc(op.key)">
                <template #icon>
                  <span class="op-symbol">{{ op.symbol }}</span>
                </template>
              </n-button>
            </template>
            {{ op.label }}
          </n-tooltip>
        </n-space>
        <div class="shift-box">
          <n-input-number v-model:value="shift" :min="1" size="small" style="width: 90px" />
          <span class="shift-hint">移位位数</span>
        </div>
      </div>

      <template v-if="result">
        <n-divider />
        <n-descriptions bordered :column="1" label-placement="left" size="medium">
          <n-descriptions-item label="说明">
            <span class="mono-value">{{ result.expr }}</span>
          </n-descriptions-item>
          <n-descriptions-item label="十进制">
            <div class="result-row">
              <span class="mono-value">{{ result.decimal }}</span>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary circle @click="copy(result.decimal)">
                    <template #icon>
                      <n-icon><Copy /></n-icon>
                    </template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
            </div>
          </n-descriptions-item>
          <n-descriptions-item label="十六进制">
            <div class="result-row">
              <span class="mono-value">{{ result.hex }}</span>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary circle @click="copy(result.hex)">
                    <template #icon>
                      <n-icon><Copy /></n-icon>
                    </template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
            </div>
          </n-descriptions-item>
          <n-descriptions-item label="二进制">
            <div class="result-row">
              <span class="mono-value">{{ result.binary }}</span>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary circle @click="copy(result.binary)">
                    <template #icon>
                      <n-icon><Copy /></n-icon>
                    </template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
            </div>
          </n-descriptions-item>
        </n-descriptions>
      </template>
    </section>
  </div>
</template>

<style scoped>
.ops-wrap {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  margin: 8px 0 4px;
}

.shift-box {
  display: flex;
  align-items: center;
  gap: 8px;
}

.shift-hint {
  font-size: 12px;
  color: var(--tb-text-3);
  white-space: nowrap;
}

.op-symbol {
  font-family: var(--tb-font-mono);
  font-size: 14px;
  font-weight: 600;
  line-height: 1;
}

.result-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.mono-value {
  font-family: var(--tb-font-mono);
  font-size: 13px;
  word-break: break-all;
}
</style>
