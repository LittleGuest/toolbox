<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { Document, CopyDocument } from "@element-plus/icons-vue";

const hyphens = ref();
const uppercase = ref();
const uuidVersion = ref();
const number = ref(5);
const uuids = ref("");

const paste = (value) => {
  console.log("paste", value);
};

const copy = (value) => {
  console.log("copy", value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px" style="max-width: 460px">
    <el-form-item label="Hyphens">
      <el-switch
        v-model="hyphens"
        class="ml-2"
        inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
        active-text="Y"
        inactive-text="N"
        active-value="1"
        inactive-value="0"
      />
    </el-form-item>

    <el-form-item label="Uppercase">
      <el-switch
        v-model="uppercase"
        class="ml-2"
        inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949"
        active-text="Y"
        inactive-text="N"
        active-value="1"
        inactive-value="0"
      />
    </el-form-item>

    <el-form-item label="UUID version">
      <el-select v-model="uuidVersion" class="m-2" size="large">
        <el-option key="4(GUID)" label="4(GUID)" value="4(GUID)" />
        <el-option key="5" label="5" value="5" />
      </el-select>
    </el-form-item>

    <el-form-item label="Generate">
      <el-text class="mx-1" type="primary">Generate UUID(s) x </el-text>
      <el-input-number
        v-model="number"
        :min="1"
        :max="999999"
        controls-position="right"
      />
    </el-form-item>

    <el-form-item label="UUID(s)">
      <el-button-group class="ml-4">
        <el-button
          type="primary"
          :icon="CopyDocument"
          v-on:click="copy(uuids)"
        />
      </el-button-group>
      <el-input v-model="uuids" :rows="10" type="textarea" />
    </el-form-item>
  </el-form>
</template>
