<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";

const greetMsg = ref("");
const name = ref("");

async function greet() {
  // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
  greetMsg.value = await invoke("greet", { name: name.value });
}
</script>

<template>
  <a-form>
    <a-form-item label="Uppercase">
      <a-switch />
    </a-form-item>

    <a-form-item label="Output Type">
      <a-select defaultValue="Hex" :style="{ width: '120px' }">
        <a-option>Hex</a-option>
      </a-select>
    </a-form-item>

    <a-form-item label="HMAC Mode">
      <a-switch />
    </a-form-item>

    <a-form-item label="Input">
      <a-textarea allow-clear show-word-limit />
    </a-form-item>

    <a-form-item label="MD5">
      <div>aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</div>
      <a-button size="small"> <icon-copy /> </a-button>
    </a-form-item>

    <a-form-item label="SHA1">
      <div>aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</div>
      <a-button size="small"> <icon-copy /> </a-button>
    </a-form-item>

    <a-form-item label="SHA256">
      <div>aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</div>
      <a-button size="small"> <icon-copy /> </a-button>
    </a-form-item>

    <a-form-item label="SHA512">
      <div>aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa</div>
      <a-button size="small"> <icon-copy /> </a-button>
    </a-form-item>
  </a-form>
</template>
