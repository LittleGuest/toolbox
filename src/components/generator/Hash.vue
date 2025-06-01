<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { Copy, Paste, Close } from "@vicons/carbon";

const uppercase = ref(false);
const outputType = ref();
const hmacMode = ref(false);
const input = ref("");
const hash = ref({});

const api = async () => {
  return await invoke("hash", {
    uppercase: uppercase.value,
    outputType: outputType.value,
    hmacMode: hmacMode.value,
    input: input.value,
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};

const change = async (value) => {
  hash.value = await api();
};

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  writeText(value);
};

const clear = () => {
  input.value = "";
  hash.value = {};
};
</script>

<template>
  <n-form label-placement="left">
    <!-- <n-form-item label="大写"> -->
    <!--   <n-switch v-model:value="uppercase" @update.value="change" checked="Y" unchecked="N" /> -->
    <!-- </n-form-item> -->
    <n-form-item label="操作">
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
    </n-form-item>
    <n-form-item label="输入">
      <n-input placeholder="请输入" clearable @update:value="change" @clear="clear" v-model:value="input" :rows="5"
        type="textarea" />
    </n-form-item>
    <n-form-item label="MD5">
      <n-input placeholder="" disabled v-model:value="hash.md5" />
      <n-button @click="copy(hash.md5)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA1">
      <n-input placeholder="" disabled v-model:value="hash.sha1" />
      <n-button @click="copy(hash.sha1)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA256">
      <n-input placeholder="" disabled v-model:value="hash.sha256" />
      <n-button @click="copy(hash.sha256)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA512">
      <n-input placeholder="" disabled v-model:value="hash.sha512" />
      <n-button @click="copy(hash.sha512)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA3 256">
      <n-input placeholder="" disabled v-model:value="hash.sha3_256" />
      <n-button @click="copy(hash.sha3_256)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA3 512">
      <n-input placeholder="" disabled v-model:value="hash.sha3_512" />
      <n-button @click="copy(hash.sha3_512)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
  </n-form>
</template>
