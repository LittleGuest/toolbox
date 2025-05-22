<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste } from "@vicons/carbon";

const inputType = ref("Decimal");
const input = ref("");
const binary = ref("");
const octal = ref("");
const decimal = ref("");
const hex = ref("");
const typeOptions = [
  {
    label: "二进制",
    value: "binary"
  },
  {
    label: "八进制",
    value: "octal"
  },
  {
    label: "十进制",
    value: "decimal"
  },
  {
    label: "十六进制",
    value: "hex"
  }
];

const api = async () => {
  const res = await invoke("number_base", {
    inputType: inputType.value,
    input: input.value,
  });
  binary.value = res.binary;
  octal.value = res.octal;
  decimal.value = res.decimal;
  hex.value = res.hex;
};

const change = (value) => {
  api();
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <n-form label-placement="left">
    <n-form-item label="输入类型">
      <n-select placeholder="请选择类型" :options="typeOptions" v-model:value="inputType" />
    </n-form-item>
    <n-form-item label="输入">
      <n-button-group>
        <n-button @click="paste">
          <template #icon>
            <n-icon>
              <Paste />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="copy(input)">
          <template #icon>
            <n-icon>
              <Copy />
            </n-icon>
          </template>
        </n-button>
      </n-button-group>
      <n-input placeholder="请输入" v-model:value="input" @input="change" maxlength="19" />
    </n-form-item>
    <n-form-item label="二进制">
      <n-input placeholder="" disabled v-model:value="binary" />
      <n-button @click="copy(binary)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="八进制">
      <n-input placeholder="" disabled v-model:value="octal" />
      <n-button @click="copy(octal)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="十进制">
      <n-input placeholder="" disabled v-model:value="decimal" />
      <n-button @click="copy(decimal)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="十六进制">
      <n-input placeholder="" disabled v-model:value="hex" />
      <n-button @click="copy(hex)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
  </n-form>
</template>
