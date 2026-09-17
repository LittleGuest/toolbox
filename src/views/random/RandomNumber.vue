<script setup lang="ts">
import { ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Play, Copy, Close } from "@vicons/carbon";

const message = useMessage();

// ---------- 整数 ----------
const intMin = ref(1);
const intMax = ref(100);
const intCount = ref(10);
const intOut = ref("");

const genInt = () => {
  const lo = Math.min(intMin.value, intMax.value);
  const hi = Math.max(intMin.value, intMax.value);
  const lines: string[] = [];
  for (let i = 0; i < intCount.value; i++) {
    lines.push(String(Math.floor(Math.random() * (hi - lo + 1)) + lo));
  }
  intOut.value = lines.join("\n");
};

// ---------- 小数 ----------
const floatMin = ref(0);
const floatMax = ref(1);
const floatDecimals = ref(2);
const floatCount = ref(10);
const floatOut = ref("");

const genFloat = () => {
  const lo = Math.min(floatMin.value, floatMax.value);
  const hi = Math.max(floatMin.value, floatMax.value);
  const lines: string[] = [];
  for (let i = 0; i < floatCount.value; i++) {
    const v = lo + Math.random() * (hi - lo);
    lines.push(v.toFixed(floatDecimals.value));
  }
  floatOut.value = lines.join("\n");
};

// ---------- 素数 ----------
const primeLower = ref(1);
const primeUpper = ref(100);
const primeCount = ref(10);
const primeOut = ref("");

const SIEVE_SPAN_LIMIT = 2_000_000;
const SIEVE_BASE_LIMIT = 5_000_000;

const sievePrimesInRange = (lo: number, hi: number): number[] => {
  const limit = Math.floor(Math.sqrt(hi));
  const isComp = new Uint8Array(limit + 1);
  const base: number[] = [];
  for (let i = 2; i <= limit; i++) {
    if (!isComp[i]) {
      base.push(i);
      for (let j = i * i; j <= limit; j += i) isComp[j] = 1;
    }
  }
  const size = hi - lo + 1;
  const seg = new Uint8Array(size);
  for (const p of base) {
    let start = Math.max(p * p, Math.ceil(lo / p) * p);
    for (let j = start; j <= hi; j += p) seg[j - lo] = 1;
  }
  const res: number[] = [];
  for (let i = 0; i < size; i++) {
    if (!seg[i]) res.push(lo + i);
  }
  return res;
};

const modPow = (base: bigint, exp: bigint, mod: bigint): bigint => {
  let result = 1n;
  let b = base % mod;
  let e = exp;
  while (e > 0n) {
    if (e & 1n) result = (result * b) % mod;
    b = (b * b) % mod;
    e >>= 1n;
  }
  return result;
};

// 确定性 Miller-Rabin（witness 集对 Number 可表示范围内均确定）
const isProbablePrime = (n: number): boolean => {
  if (!Number.isInteger(n) || n < 2) return false;
  const SMALL = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
  for (const p of SMALL) {
    if (n === p) return true;
    if (n % p === 0) return false;
  }
  const bn = BigInt(n);
  let d = bn - 1n;
  let s = 0;
  while ((d & 1n) === 0n) {
    d >>= 1n;
    s += 1;
  }
  const WITNESSES = [2n, 325n, 9375n, 28178n, 450775n, 9780504n, 1795265022n];
  for (const a of WITNESSES) {
    if (a % bn === 0n) continue;
    let x = modPow(a, d, bn);
    if (x === 1n || x === bn - 1n) continue;
    let composite = true;
    for (let r = 1; r < s; r++) {
      x = (x * x) % bn;
      if (x === bn - 1n) {
        composite = false;
        break;
      }
    }
    if (composite) return false;
  }
  return true;
};

// 大区间：随机采样 + 素性测试，尝试次数有上限，避免死循环
const randomPrimeInRange = (lo: number, hi: number): number => {
  const span = hi - lo + 1;
  const attempts = Math.min(Math.max(1, span), 2000);
  for (let i = 0; i < attempts; i++) {
    const candidate = lo + Math.floor(Math.random() * span);
    if (isProbablePrime(candidate)) return candidate;
  }
  return -1;
};

const genPrime = () => {
  let lo = primeLower.value;
  let hi = primeUpper.value;
  if (lo > hi) [lo, hi] = [hi, lo];
  lo = Math.max(lo, 2);
  if (hi < 2) {
    message.warning("区间上限需 ≥ 2");
    return;
  }
  if (lo > hi) {
    message.warning("区间内没有素数");
    primeOut.value = "";
    return;
  }
  const span = hi - lo + 1;
  // 区间过大时（跨度或筛法基过大）走采样方案，只生成到上限为止
  const useSieve = span <= SIEVE_SPAN_LIMIT && Math.floor(Math.sqrt(hi)) <= SIEVE_BASE_LIMIT;
  const lines: string[] = [];
  if (useSieve) {
    const list = sievePrimesInRange(lo, hi);
    if (!list.length) {
      message.warning("区间内没有素数");
      primeOut.value = "";
      return;
    }
    for (let i = 0; i < primeCount.value; i++) {
      lines.push(String(list[Math.floor(Math.random() * list.length)]));
    }
  } else {
    for (let i = 0; i < primeCount.value; i++) {
      const p = randomPrimeInRange(lo, hi);
      if (p < 0) {
        message.warning("在指定区间内未找到素数");
        break;
      }
      lines.push(String(p));
    }
  }
  primeOut.value = lines.join("\n");
};

// ---------- 十六进制 ----------
const hexLength = ref(8);
const hexCount = ref(10);
const hexOut = ref("");

const HEX_CHARS = "0123456789abcdef";

const genHex = () => {
  const lines: string[] = [];
  for (let i = 0; i < hexCount.value; i++) {
    const buf = new Uint8Array(hexLength.value);
    crypto.getRandomValues(buf);
    let s = "";
    for (let j = 0; j < buf.length; j++) s += HEX_CHARS[buf[j] & 0x0f];
    lines.push(s);
  }
  hexOut.value = lines.join("\n");
};

// ---------- 二进制 ----------
const binBits = ref(8);
const binCount = ref(10);
const binOut = ref("");

const genBin = () => {
  const bits = binBits.value;
  const lines: string[] = [];
  for (let i = 0; i < binCount.value; i++) {
    const buf = new Uint8Array(Math.ceil(bits / 8));
    crypto.getRandomValues(buf);
    let s = "";
    for (let j = 0; j < bits; j++) {
      s += (buf[Math.floor(j / 8)] >> (7 - (j % 8))) & 1 ? "1" : "0";
    }
    // 首字符不能为 0（位数 = 1 时除外）
    if (bits > 1) s = "1" + s.slice(1);
    lines.push(s);
  }
  binOut.value = lines.join("\n");
};

// ---------- 字节 ----------
const bytePerLine = ref(8);
const byteLines = ref(10);
const byteOut = ref("");

const genByte = () => {
  const lines: string[] = [];
  for (let i = 0; i < byteLines.value; i++) {
    const buf = new Uint8Array(bytePerLine.value);
    crypto.getRandomValues(buf);
    lines.push(Array.from(buf).join(", "));
  }
  byteOut.value = lines.join("\n");
};

// ---------- 复制 / 清除 ----------
const outputs = {
  int: intOut,
  float: floatOut,
  prime: primeOut,
  hex: hexOut,
  bin: binOut,
  byte: byteOut,
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
        <n-tab-pane name="int" tab="整数">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">最小值</span>
              <n-input-number v-model:value="intMin" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">最大值</span>
              <n-input-number v-model:value="intMax" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="intCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genInt">
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
            <n-input
              v-model:value="intOut"
              type="textarea"
              :rows="10"
              placeholder="区间内随机整数（每行一个）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('int')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('int')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="float" tab="小数">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">最小值</span>
              <n-input-number v-model:value="floatMin" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">最大值</span>
              <n-input-number v-model:value="floatMax" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">小数位数</span>
              <n-input-number v-model:value="floatDecimals" :min="0" :max="10" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="floatCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genFloat">
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
            <n-input
              v-model:value="floatOut"
              type="textarea"
              :rows="10"
              placeholder="区间内随机小数（每行一个）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('float')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('float')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="prime" tab="素数">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">下限</span>
              <n-input-number v-model:value="primeLower" :min="0" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">上限</span>
              <n-input-number v-model:value="primeUpper" :min="0" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="primeCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genPrime">
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
            <n-input
              v-model:value="primeOut"
              type="textarea"
              :rows="10"
              placeholder="区间内随机素数（每行一个）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('prime')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('prime')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="hex" tab="十六进制">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">长度</span>
              <n-input-number v-model:value="hexLength" :min="1" :max="4096" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="hexCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genHex">
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
            <n-input
              v-model:value="hexOut"
              type="textarea"
              :rows="10"
              placeholder="随机十六进制字符串（每行一个，不含 0x 前缀）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('hex')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('hex')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="bin" tab="二进制">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">位数</span>
              <n-input-number v-model:value="binBits" :min="1" :max="4096" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="binCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genBin">
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
            <n-input
              v-model:value="binOut"
              type="textarea"
              :rows="10"
              placeholder="随机二进制字符串（每行一个，首字符不为 0）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('bin')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('bin')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="byte" tab="字节">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">每行字节数</span>
              <n-input-number v-model:value="bytePerLine" :min="1" :max="4096" style="width: 140px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">行数</span>
              <n-input-number v-model:value="byteLines" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genByte">
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
            <n-input
              v-model:value="byteOut"
              type="textarea"
              :rows="10"
              placeholder="随机字节序列（每行逗号分隔的 0-255 十进制数）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('byte')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('byte')">
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
