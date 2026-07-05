<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { reactive, ref } from "vue";
import { useMessage } from "naive-ui";

const props = defineProps({
  generator: {
    type: String,
    required: true,
  },
});

const message = useMessage();

const defaultValue = {
  locale: "zh_cn",
  includeDefault: false,
  defaultValue: "",
  defaultPercentage: 5,
  includeNull: false,
  nullPercentage: 5,
  unique: false,
  forbiddenLinks: false,
};

const form = reactive({ ...defaultValue });
const previewValue = ref("");

const reset = () => {
  Object.assign(form, defaultValue);
  previewValue.value = "";
};

const preview = async () => {
  previewValue.value = await invoke<string>("preview_generator", {
    generator: props.generator,
    config: { locale: form.locale },
  }).catch((err) => {
    message.error(err);
    return "";
  });
};

defineExpose({
  getConfig: () => ({ ...form }),
  setConfig: (config = {}) => Object.assign(form, config),
});

const localeOptions = [
  { label: "简体中文", value: "zh_cn" },
  { label: "繁体中文", value: "zh_traditional" },
  { label: "中文拼音", value: "zh_pinyin" },
  { label: "English", value: "en_us" },
];
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <n-form-item label="语言">
      <n-select
        v-model:value="form.locale"
        :options="localeOptions"
        placeholder="选择语言"
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
        v-model:value="form.defaultValue"
        placeholder="请输入默认值"
        :disabled="!form.includeDefault"
        clearable
      />
    </n-form-item>
    <n-form-item path="defaultPercentage" label=" ">
      <n-input-number
        v-model:value="form.defaultPercentage"
        placeholder="百分比"
        :disabled="!form.includeDefault"
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
        v-model:value="form.nullPercentage"
        placeholder="百分比"
        :disabled="!form.includeNull"
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
