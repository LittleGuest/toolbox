<script setup lang="ts">
import { computed, ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";

const message = useMessage();

const ipv4 = ref("");
const ipv6 = ref("");

interface IpNumberResult {
  binary: string;
  octal: string;
  decimal: string;
  hex: string;
}

const emptyResult = (): IpNumberResult => ({
  binary: "",
  octal: "",
  decimal: "",
  hex: "",
});

const ipv4Result = ref<IpNumberResult>(emptyResult());
const ipv6Result = ref<IpNumberResult>(emptyResult());

const ipv4Rows = computed(() => resultRows(ipv4Result.value));
const ipv6Rows = computed(() => resultRows(ipv6Result.value));

const resultRows = (value: IpNumberResult) => [
  { label: "二进制", value: value.binary },
  { label: "八进制", value: value.octal },
  { label: "十进制", value: value.decimal },
  { label: "十六进制", value: value.hex },
];

const convertIp = async (type: "v4" | "v6", ip: string) => {
  if (!ip.trim()) {
    message.warning("请输入 IP 地址");
    return emptyResult();
  }

  try {
    return await invoke<IpNumberResult>("ip_to_number", {
      t: type,
      ip,
    });
  } catch (error) {
    message.error(`${error}`);
    return emptyResult();
  }
};

const convertIpv4 = async () => {
  ipv4Result.value = await convertIp("v4", ipv4.value);
};

const convertIpv6 = async () => {
  ipv6Result.value = await convertIp("v6", ipv6.value);
};

const pasteIpv4 = async () => {
  ipv4.value = await readText();
  await convertIpv4();
};

const pasteIpv6 = async () => {
  ipv6.value = await readText();
  await convertIpv6();
};

const copy = async (value: string) => {
  if (!value) {
    return;
  }
  await writeText(value);
  message.success("复制成功");
};
</script>

<template>
  <n-space vertical size="large">
    <n-card title="IPv4 转换">
      <n-space vertical>
        <n-input-group>
          <n-input v-model:value="ipv4" placeholder="例如 192.168.1.1" @keydown.enter="convertIpv4" />
          <n-button type="primary" @click="convertIpv4">转换</n-button>
          <n-button @click="pasteIpv4">粘贴</n-button>
        </n-input-group>
        <n-list bordered>
          <n-list-item v-for="row in ipv4Rows" :key="row.label">
            <template #prefix>
              <strong class="result-label">{{ row.label }}</strong>
            </template>
            <n-code :code="row.value || '-'" word-wrap />
            <template #suffix>
              <n-button text :disabled="!row.value" @click="copy(row.value)">复制</n-button>
            </template>
          </n-list-item>
        </n-list>
      </n-space>
    </n-card>

    <n-card title="IPv6 转换">
      <n-space vertical>
        <n-input-group>
          <n-input v-model:value="ipv6" placeholder="例如 2001:db8::1" @keydown.enter="convertIpv6" />
          <n-button type="primary" @click="convertIpv6">转换</n-button>
          <n-button @click="pasteIpv6">粘贴</n-button>
        </n-input-group>
        <n-list bordered>
          <n-list-item v-for="row in ipv6Rows" :key="row.label">
            <template #prefix>
              <strong class="result-label">{{ row.label }}</strong>
            </template>
            <n-code :code="row.value || '-'" word-wrap />
            <template #suffix>
              <n-button text :disabled="!row.value" @click="copy(row.value)">复制</n-button>
            </template>
          </n-list-item>
        </n-list>
      </n-space>
    </n-card>
  </n-space>
</template>

<style scoped>
.result-label {
  display: inline-block;
  min-width: 72px;
}
</style>
