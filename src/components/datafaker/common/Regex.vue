<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { QuestionCircleOutlined } from "@vicons/antd";

// 正则表达式生成器默认值
const defaultValue = {
  pattern: "[A-Za-z0-9]{10}", // 正则表达式
  rawDataMode: false, // 原始数据模式
  previewValue: "", // 预览值

  includeDefault: false, // 包含默认值
  defaultValue: "", // 默认值
  defaultPercentage: 5, // 默认值百分比
  includeNull: false, // 包含空值
  nullPercentage: 5, // 空值百分比
  unique: false, // 唯一值
  forbiddenLinks: false, // 禁用字段之间的数据链接
};

// 表单数据
let form = reactive({
  ...defaultValue,
});

// 重置属性
const reset = () => {
  form.pattern = defaultValue.pattern;
  form.rawDataMode = defaultValue.rawDataMode;
  form.previewValue = defaultValue.previewValue;
  form.includeDefault = defaultValue.includeDefault;
  form.defaultValue = defaultValue.defaultValue;
  form.defaultPercentage = defaultValue.defaultPercentage;
  form.includeNull = defaultValue.includeNull;
  form.nullPercentage = defaultValue.nullPercentage;
  form.unique = defaultValue.unique;
  form.forbiddenLinks = defaultValue.forbiddenLinks;
};

// 预览正则表达式
const previewApi = async (pattern) => {
  return await invoke("preview_regex", { pattern })
    .then((res) => {
      return res;
    })
    .catch((err) => {
      message.error(err);
    });
};

// 生成预览数据
const preview = async () => {
  form.previewValue = await previewApi(form.pattern);
};
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <n-form-item path="pattern" label="正则表达式">
      <n-input
        v-model:value="form.pattern"
        type="textarea"
        :rows="6"
        placeholder="例如: [A-Za-z0-9]{10}"
      />
    </n-form-item>
    <n-form-item path="rawDataMode" label="原始数据模式">
      <n-checkbox
        v-model:checked="form.rawDataMode"
        style="margin-right: 5px"
      />
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-icon size="20">
            <QuestionCircleOutlined />
          </n-icon>
        </template>
        原始数据模式下，生成的数据将直接使用正则表达式生成的结果。
      </n-tooltip>
    </n-form-item>
    <n-form-item path="previewValue" label="预览">
      <n-input v-model:value="form.previewValue" readonly placeholder="" />
      <n-button @click="preview">刷新</n-button>
    </n-form-item>
    <n-form-item path="includeDefault" label="包含默认值">
      <n-checkbox v-model:checked="form.includeDefault" />
    </n-form-item>
    <n-form-item path="defaultValue" label=" ">
      <n-input
        placeholder="请输入默认值"
        :disabled="!form.includeDefault"
        v-model:value="form.defaultValue"
        clearable
      />
    </n-form-item>
    <n-form-item path="defaultPercentage" label=" ">
      <n-input-number
        placeholder="百分比"
        :disabled="!form.includeDefault"
        v-model:value="form.defaultPercentage"
        :min="0"
        :max="100"
        :step="1"
        clearable
      >
        <template #suffix> % </template>
      </n-input-number>
    </n-form-item>
    <n-form-item path="includeNull" label="包含NULL值">
      <n-checkbox v-model:checked="form.includeNull" />
    </n-form-item>
    <n-form-item path="nullPercentage" label=" ">
      <n-input-number
        class="percentage-input"
        placeholder="百分比"
        :disabled="!form.includeNull"
        v-model:value="form.nullPercentage"
        :min="0"
        :max="100"
        :step="1"
      />
    </n-form-item>
    <n-form-item path="previewValue" label="设置唯一">
      <n-checkbox v-model:checked="form.unique" />
    </n-form-item>
    <n-form-item path="previewValue" label="禁用字段之间数据链接">
      <n-checkbox v-model:checked="form.forbiddenLinks" />
    </n-form-item>
    <n-form-item label=" ">
      <n-button @click="reset"> 重置属性 </n-button>
    </n-form-item>
  </n-form>
</template>

<style scoped></style>
