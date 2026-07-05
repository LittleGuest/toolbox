<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";
import { useMessage } from "naive-ui";

// 消息提示
const message = useMessage();

// 生成器默认值
const defaultValue = {
  format: "full_name", // 格式类型
  locale: "zh_cn", // 语言

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
  form.format = defaultValue.format;
  form.locale = defaultValue.locale;
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
  return await invoke("preview_name", { config })
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
    format: form.format,
    locale: form.locale,
  });
};

// 格式选项
const formatOptions = [
  { label: "全名", value: "full_name" },
  { label: "名", value: "first_name" },
  { label: "姓", value: "last_name" },
  // { label: "前缀", value: "prefix" },
  // { label: "后缀", value: "suffix" },
  // { label: "头衔", value: "title" },
  // { label: "用户名", value: "username" },
];
const localeOptions = [
  { label: "简体中文", value: "zh_cn" },
  { label: "繁体中文", value: "zh_traditional" },
  { label: "中文拼音", value: "zh_pinyin" },
  { label: "English", value: "en_us" },
];
defineExpose({
  getConfig: () => ({ ...form }),
  setConfig: (config = {}) => {
    Object.assign(form, config);
    if (!form.locale && Array.isArray(form.locales)) {
      form.locale = form.locales[0] || defaultValue.locale;
    }
  },
});
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <!-- 格式类型 -->
    <n-form-item label="格式类型">
      <n-select
        v-model:value="form.format"
        :options="formatOptions"
        placeholder="选择格式类型"
      />
    </n-form-item>

    <!-- 语言选择 -->
    <n-form-item label="语言">
      <n-select
        v-model:value="form.locale"
        :options="localeOptions"
        placeholder="选择语言"
      />
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

    <!-- 唯一值 -->
    <n-form-item path="unique" label="设置唯一">
      <n-checkbox v-model:checked="form.unique" />
    </n-form-item>

    <!-- 禁用字段之间数据链接 -->
    <n-form-item path="forbiddenLinks" label="禁用字段之间数据链接">
      <n-checkbox v-model:checked="form.forbiddenLinks" />
    </n-form-item>

    <n-form-item label=" ">
      <n-button @click="reset"> 重置属性 </n-button>
    </n-form-item>
  </n-form>
</template>

<style scoped></style>
