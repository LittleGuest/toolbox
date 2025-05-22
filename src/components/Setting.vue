<script setup>
import { ref } from "vue";

const language = ref("chinese");
const theme = ref();
const smartDetection = ref(false);
const font = ref();

const api = async () => {
  //const value = await invoke("uuid", { hyphens: hyphens.value, uppercase: uppercase.value, version: uuidVersion.value, number: number.value, });
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-form-item label="语言">
      <el-select v-model="language" class="m-2" size="large">
        <!-- <el-option key="1" label="系统默认" value="default" /> -->
        <el-option key="2" label="简体中文" value="chinese" />
        <el-option key="3" label="English" value="english" />
      </el-select>
    </el-form-item>

    <!--
    <el-form-item label="主题">
      <el-select v-model="theme" class="m-2" size="large">
      </el-select>
    </el-form-item>
    -->

    <!--
    <el-form-item label="Smart detection">
      <strong>Automatically detect the best tool based on the clippboard content</strong>
      <el-switch v-model="smartDetection" class="ml-2" inline-prompt
        style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949" active-text="Y" inactive-text="N" />
    </el-form-item>
    -->

    <!--
    <el-form-item label="Font">
      <el-select v-model="font" class="m-2" size="large">
      </el-select>
    </el-form-item>
    -->
  </el-form>
</template>
