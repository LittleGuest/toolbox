<script setup>
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";
import { useMessage } from "naive-ui";

// 消息提示
const message = useMessage();

// UUID版本
const versionOptions = [
  {
    label: "v1",
    value: 1,
  },
  {
    label: "v3",
    value: 3,
  },
  {
    label: "v4",
    value: 4,
  },
  {
    label: "v5",
    value: 5,
  },
  {
    label: "v6",
    value: 6,
  },
  {
    label: "v7",
    value: 7,
  },
  {
    label: "v8",
    value: 8,
  },
];

// 生成器默认值
const defaultValue = {
  uppercase: false, //是否大写
  uuidVersion: 4, //uuid版本
  includeHyphen: true, // 连字符

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
  form.uppercase = defaultValue.uppercase;
  form.uuidVersion = defaultValue.uuidVersion;
  form.includeHyphen = defaultValue.includeHyphen;
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
  return await invoke("preview_uuid", { config })
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
    uppercase: form.uppercase,
    uuidVersion: form.uuidVersion,
    includeHyphen: form.includeHyphen,
  });
};
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <n-form-item label="大写">
      <n-switch v-model:value="form.uppercase" checked="Y" unchecked="N" />
    </n-form-item>
    <n-form-item label="UUID版本">
      <n-select
        placeholder="请选择版本"
        :options="versionOptions"
        v-model:value="form.uuidVersion"
      />
    </n-form-item>
    <!-- 包含连字符 -->
    <n-form-item path="includeHyphen" label="包含连字符">
      <n-checkbox v-model:checked="form.includeHyphen" />
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
