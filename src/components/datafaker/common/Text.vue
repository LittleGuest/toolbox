<script setup>
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";

// 生成器默认值
const defaultValue = {
  minLength: 100, // 最小字符数
  maxLength: 10000, // 最大字符数

  includeDefault: false, // 包含默认值
  defaultValue: "", // 默认值
  defaultPercentage: 5, // 默认值百分比
  includeNull: false, // 包含空值
  nullPercentage: 5, // 空值百分比
  unique: false, // 唯一值
  forbiddenLinks: false, // 禁用字段之间的数据链接
};
// 表单数据
const form = reactive({
  ...defaultValue,
});

// 重置属性
const reset = () => {
  form.minLength = defaultValue.minLength;
  form.maxLength = defaultValue.maxLength;
  form.includeDefault = defaultValue.includeDefault;
  form.defaultValue = defaultValue.defaultValue;
  form.defaultPercentage = defaultValue.defaultPercentage;
  form.includeNull = defaultValue.includeNull;
  form.nullPercentage = defaultValue.nullPercentage;
  form.unique = defaultValue.unique;
  form.forbiddenLinks = defaultValue.forbiddenLinks;
  previewValue.value = "";
};

// 预览数据
const previewValue = ref("");
// 预览API
const previewApi = async (config) => {
  return await invoke("preview_text", { config })
    .then((res) => {
      return res;
    })
    .catch((err) => {
      message.error(err);
    });
};
// 生成预览数据
const preview = async () => {
  previewValue.value = await previewApi({
    minLength: form.minLength,
    maxLength: form.maxLength,
  });
};
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <!-- 字符数范围 -->
    <n-form-item label="字符数">
      <div style="display: inline-flex; align-items: center">
        <n-input-number
          v-model:value="form.minLength"
          :min="1"
          :max="form.maxLength - 1"
          placeholder="最小值"
        />
        <span>—</span>
        <n-input-number
          v-model:value="form.maxLength"
          :min="form.minLength + 1"
          :max="4294967295"
          placeholder="最大值"
        />
      </div>
    </n-form-item>

    <!-- 预览 -->
    <n-form-item path="previewValue" label="预览">
      <n-input v-model:value="previewValue" readonly placeholder="" />
      <n-button @click="preview">刷新</n-button>
    </n-form-item>

    <!-- 其它配置选项 -->

    <!-- 包含默认值 -->
    <n-form-item path="includeDefault" label="包含默认值">
      <n-checkbox v-model:checked="form.includeDefault" />
    </n-form-item>
    <!-- 默认值 -->
    <n-form-item path="defaultValue" label=" ">
      <n-input
        placeholder="请输入默认值"
        :disabled="!form.includeDefault"
        v-model:value="form.defaultValue"
        clearable
      />
    </n-form-item>
    <!-- 默认值百分比 -->
    <n-form-item path="defaultPercentage" label=" ">
      <n-input-number
        placeholder="百分比"
        :disabled="!form.includeDefault"
        v-model:value="form.defaultPercentage"
        :min="0"
        :max="100"
        :step="1"
      >
        <template #suffix> % </template>
      </n-input-number>
    </n-form-item>

    <!-- 包含NULL值 -->
    <n-form-item path="includeNull" label="包含NULL值">
      <n-checkbox v-model:checked="form.includeNull" />
    </n-form-item>
    <!-- NULL值百分比 -->
    <n-form-item path="nullPercentage" label=" ">
      <n-input-number
        placeholder="百分比"
        :disabled="!form.includeNull"
        v-model:value="form.nullPercentage"
        :min="0"
        :max="100"
        :step="1"
      >
        <template #suffix> % </template>
      </n-input-number>
    </n-form-item>

    <n-form-item label=" ">
      <n-button @click="reset"> 重置属性 </n-button>
    </n-form-item>
  </n-form>
</template>

<style scoped></style>
