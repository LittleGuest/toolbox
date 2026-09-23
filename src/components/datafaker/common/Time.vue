<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";
import { useMessage } from "naive-ui";

const message = useMessage();

const defaultValue = {
  wholeDay: true,
  startTime: null,
  endTime: null,

  includeDefault: false, // 包含默认值
  defaultValue: "", // 默认值
  defaultPercentage: 5, // 默认值百分比
  includeNull: false, // 包含空值
  nullPercentage: 5, // 空值百分比
  unique: false, // 唯一值
  forbiddenLinks: false, // 禁用字段之间的数据链接
};

const form = reactive({
  ...defaultValue,
});

const reset = () => {
  form.wholeDay = defaultValue.wholeDay;
  form.startTime = defaultValue.startTime;
  form.endTime = defaultValue.endTime;
  form.includeDefault = defaultValue.includeDefault;
  form.defaultValue = defaultValue.defaultValue;
  form.defaultPercentage = defaultValue.defaultPercentage;
  form.includeNull = defaultValue.includeNull;
  form.nullPercentage = defaultValue.nullPercentage;
  form.unique = defaultValue.unique;
  form.forbiddenLinks = defaultValue.forbiddenLinks;
  previewValue.value = "";
};

const previewValue = ref("");
const previewApi = async (config) => {
  return await invoke("preview_time", { config })
    .then((res) => {
      return res;
    })
    .catch((err) => {
      message.error(err);
    });
};
const preview = async () => {
  previewValue.value = await previewApi({
    startDate: form.startDate,
    endDate: form.endDate,
    wholeDay: form.wholeDay,
    startTime: form.startTime,
    endTime: form.endTime,
    weekType: form.weekType,
    customWeeks: form.customWeeks,
  });
};
defineExpose({
  getConfig: () => ({ ...form }),
  setConfig: (config = {}) => Object.assign(form, config),
});
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <n-form-item label="一整天">
      <n-checkbox v-model:checked="form.wholeDay" />
    </n-form-item>

    <n-form-item label="开始时间">
      <n-time-picker
        :disabled="form.wholeDay"
        v-model:value="form.startTime"
        placeholder="选择开始时间"
      />
    </n-form-item>
    <n-form-item label="结束时间">
      <n-time-picker
        :disabled="form.wholeDay"
        v-model:value="form.endTime"
        placeholder="选择结束时间"
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

    <n-form-item path="unique" label="设置唯一">
      <n-checkbox v-model:checked="form.unique" />
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
