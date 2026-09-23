<script setup lang="ts">
import { computed, ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowsHorizontal, Copy } from "@vicons/carbon";

const message = useMessage();

type IpFormat = "ipv4" | "decimal" | "hex" | "binary" | "octal" | "ipv6";

const formatOptions: { label: string; value: IpFormat }[] = [
  { label: "IPv4", value: "ipv4" },
  { label: "十进制", value: "decimal" },
  { label: "十六进制", value: "hex" },
  { label: "二进制", value: "binary" },
  { label: "八进制", value: "octal" },
  { label: "IPv6", value: "ipv6" },
];

const input = ref("");
const format = ref<IpFormat>("ipv4");

const placeholderText = computed(() => {
  switch (format.value) {
    case "ipv4":
      return "如 192.168.1.1";
    case "decimal":
      return "如 3232235777（0 ~ 4294967295）";
    case "hex":
      return "如 0xC0A80101（0x 前缀，8 位以内）";
    case "binary":
      return "如 11000000101010000000000100000001（32 位以内）";
    case "octal":
      return "如 0177（0 前缀八进制整数）";
    case "ipv6":
      return "如 2001:0db8:85a3::8a2e:0370:7334";
  }
});

interface IpResult {
  ipv4: string;
  decimal: string;
  hex: string;
  binary: string;
  octal: string;
  ipv6Binary: string;
  isV6: boolean;
}

const result = ref<IpResult | null>(null);

const resultItems = computed(() => {
  if (!result.value) return [];
  const r = result.value;
  const items: { label: string; value: string }[] = [
    { label: "IPv4 点分形式", value: r.ipv4 },
    { label: "十进制整数", value: r.decimal },
    { label: "十六进制", value: r.hex },
    { label: "二进制", value: r.binary },
    { label: "八进制", value: r.octal },
  ];
  if (r.isV6) {
    items.push({ label: "二进制展开（16 位 × 8 段）", value: r.ipv6Binary });
  }
  return items;
});

const copy = (value: string) => {
  if (!value) return;
  writeText(value);
  message.success("复制成功");
};

const ipv4ToInt = (ip: string): bigint | null => {
  const parts = ip.trim().split(".");
  if (parts.length !== 4) return null;
  let v = 0n;
  for (const p of parts) {
    if (!/^\d{1,3}$/.test(p)) return null;
    const n = Number(p);
    if (n > 255) return null;
    v = (v << 8n) | BigInt(n);
  }
  return v;
};

const intToIpv4 = (int: bigint): string => {
  const parts: string[] = [];
  for (let i = 3; i >= 0; i--) {
    parts.push(((int >> BigInt(i * 8)) & 0xffn).toString());
  }
  return parts.join(".");
};

const ipv6ToInt = (ip: string): bigint | null => {
  const s = ip.trim().toLowerCase();
  if (!s) return null;
  if (s.split("::").length > 2) return null;
  let groups: string[];
  if (s.includes("::")) {
    const [left, right] = s.split("::");
    const leftParts = left ? left.split(":") : [];
    const rightParts = right ? right.split(":") : [];
    const parts = [...leftParts, ...rightParts];
    if (parts.length >= 8) return null;
    for (const p of parts) {
      if (!/^[0-9a-f]{1,4}$/.test(p)) return null;
    }
    groups = [...leftParts, ...Array(8 - parts.length).fill("0"), ...rightParts];
  } else {
    groups = s.split(":");
    if (groups.length !== 8) return null;
    for (const p of groups) {
      if (!/^[0-9a-f]{1,4}$/.test(p)) return null;
    }
  }
  let v = 0n;
  for (const g of groups) {
    v = (v << 16n) | BigInt(parseInt(g, 16));
  }
  return v;
};

const ipv6ToBinary = (int: bigint): string => {
  const segs: string[] = [];
  for (let i = 7; i >= 0; i--) {
    segs.push(((int >> BigInt(i * 16)) & 0xffffn).toString(2).padStart(16, "0"));
  }
  return segs.join(" ");
};

const convert = () => {
  const raw = input.value.trim();
  if (!raw) {
    message.error("请输入内容");
    result.value = null;
    return;
  }
  let int: bigint | null = null;
  let isV6 = false;
  switch (format.value) {
    case "ipv4": {
      int = ipv4ToInt(raw);
      if (int === null) message.error("IPv4 格式非法（应为 4 段 0-255）");
      break;
    }
    case "decimal": {
      if (!/^\d{1,10}$/.test(raw)) {
        message.error("十进制格式非法");
        break;
      }
      const v = BigInt(raw);
      if (v > 0xffffffffn) {
        message.error("十进制超出范围（0 ~ 4294967295）");
        break;
      }
      int = v;
      break;
    }
    case "hex": {
      const h = raw.replace(/^0x/i, "");
      if (!/^[0-9a-fA-F]{1,8}$/.test(h)) {
        message.error("十六进制格式非法（0x 前缀，8 位以内）");
        break;
      }
      int = BigInt("0x" + h);
      break;
    }
    case "binary": {
      const b = raw.replace(/^0b/i, "");
      if (!/^[01]{1,32}$/.test(b)) {
        message.error("二进制格式非法（32 位以内 0/1）");
        break;
      }
      int = BigInt("0b" + b);
      break;
    }
    case "octal": {
      const o = raw.startsWith("0") ? raw.slice(1) : raw;
      if (o === "") {
        int = 0n;
        break;
      }
      if (!/^[0-7]{1,11}$/.test(o)) {
        message.error("八进制格式非法（0 前缀 + 八进制数字）");
        break;
      }
      int = BigInt("0o" + o);
      break;
    }
    case "ipv6": {
      int = ipv6ToInt(raw);
      isV6 = true;
      if (int === null) message.error("IPv6 格式非法（8 组 4 位 hex，支持 :: 压缩）");
      break;
    }
  }
  if (int === null) {
    result.value = null;
    return;
  }
  const bitLen = isV6 ? 128 : 32;
  const hexDigits = isV6 ? 32 : 8;
  result.value = {
    ipv4: isV6 ? "—" : intToIpv4(int),
    decimal: int.toString(),
    hex: "0x" + int.toString(16).padStart(hexDigits, "0"),
    binary: int.toString(2).padStart(bitLen, "0"),
    octal: "0" + int.toString(8),
    ipv6Binary: isV6 ? ipv6ToBinary(int) : "",
    isV6,
  };
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item" style="flex: 1; min-width: 240px">
          <n-input
            v-model:value="input"
            clearable
            class="tb-mono"
            :placeholder="placeholderText"
            @keyup.enter="convert"
          />
        </div>
        <div class="tb-config-item">
          <n-select v-model:value="format" :options="formatOptions" style="width: 130px" />
        </div>
        <div class="tb-config-item">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" @click="convert">
                <template #icon>
                  <n-icon><ArrowsHorizontal /></n-icon>
                </template>
              </n-button>
            </template>
            转换
          </n-tooltip>
        </div>
      </div>

      <template v-if="result">
        <n-divider />
        <n-descriptions bordered :column="1" label-placement="left" size="medium">
          <n-descriptions-item v-for="item in resultItems" :key="item.label" :label="item.label">
            <div class="result-row">
              <span class="mono-value">{{ item.value }}</span>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary circle @click="copy(item.value)">
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
