<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref } from "vue";
import { useMessage } from "naive-ui";
import { fileExtensionTypeData } from "@/components/datafaker/constants";

const message = useMessage();

const defaultValue = {
  includeExtension: true,
  fileExtensionType: "",
  fileExtension: [],

  includeDefault: false, // 包含默认值
  defaultValue: "", // 默认值
  defaultPercentage: 5, // 默认值百分比
  includeNull: false, // 包含空值
  nullPercentage: 5, // 空值百分比
  forbiddenLinks: false, // 禁用字段之间的数据链接
};

const form = reactive({
  ...defaultValue,
});

const fileExtensionTypeOptions = fileExtensionTypeData.map((item) => ({
  label: item.label,
  value: item.value,
}));
watch(
  () => form.fileExtensionType,
  (newValue) => {
    form.fileExtension = fileExtensionTypeData.find(
      (item) => item.value === newValue
    ).data;
  }
);

const reset = () => {
  form.includeExtension = defaultValue.includeExtension;
  form.fileExtensionType = defaultValue.fileExtensionType;
  form.fileExtension = defaultValue.fileExtension;
  form.includeDefault = defaultValue.includeDefault;
  form.defaultValue = defaultValue.defaultValue;
  form.defaultPercentage = defaultValue.defaultPercentage;
  form.includeNull = defaultValue.includeNull;
  form.nullPercentage = defaultValue.nullPercentage;
  form.forbiddenLinks = defaultValue.forbiddenLinks;
  previewValue.value = "";
};

const previewValue = ref("");
const previewApi = async (config) => {
  return await invoke("preview_file_name", { config })
    .then((res) => {
      return res;
    })
    .catch((err) => {
      message.error(err);
    });
};
const preview = async () => {
  previewValue.value = await previewApi({
    includeExtension: form.includeExtension,
    fileExtensionType: form.fileExtensionType,
    fileExtension: form.fileExtension,
  });
};
defineExpose({
  getConfig: () => ({ ...form }),
  setConfig: (config = {}) => Object.assign(form, config),
});
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <n-form-item path="includeExtension" label="包含扩展名">
      <n-checkbox v-model:checked="form.includeExtension" />
    </n-form-item>
    <n-form-item
      path="fileExtensionType"
      label="扩展名类型"
      label-placement="left"
      label-width="180"
    >
      <n-select
        :disabled="!form.includeExtension"
        v-model:value="form.fileExtensionType"
        :options="fileExtensionTypeOptions"
        placeholder="请选择扩展名类型"
        filterable
      />
    </n-form-item>

    <n-form-item path="fileExtension" label="扩展名">
      <n-input
        :disabled="!form.includeExtension"
        v-model:value="form.fileExtension"
        type="textarea"
        :rows="6"
        placeholder="例如: .jpg, .jpeg, .png, .gif, .bmp, .svg, .webp, .tiff, .ico, .eps, .txt, .rtf, .pdf, .docx, .xlsx, .csv, .html, .zip"
      />
    </n-form-item>

    <n-form-item path="previewValue" label="预览">
      <n-input v-model:value="previewValue" readonly placeholder="" />
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
      >
        <template #suffix> % </template>
      </n-input-number>
    </n-form-item>

    <n-form-item path="includeNull" label="包含NULL值">
      <n-checkbox v-model:checked="form.includeNull" />
    </n-form-item>
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

    <n-form-item path="forbiddenLinks" label="禁用字段之间数据链接">
      <n-checkbox v-model:checked="form.forbiddenLinks" />
    </n-form-item>

    <n-form-item label=" ">
      <n-button @click="reset"> 重置属性 </n-button>
    </n-form-item>
  </n-form>
</template>

<style scoped></style>
