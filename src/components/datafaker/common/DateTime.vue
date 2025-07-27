<script setup>
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";
import { useMessage } from "naive-ui";

// 消息提示
const message = useMessage();

// 生成器默认值
const defaultValue = {
  // 开始日期
  startDate: "2000-01-01",
  // 结束日期
  endDate: new Date().toISOString().split("T")[0],
  // 全天选项
  wholeDay: true,
  // 开始时间
  startTime: null,
  // 结束时间
  endTime: null,
  // 星期选择
  weekType: "all", // 'all', 'workday', 'custom'
  // 自定义星期
  customWeeks: ["1", "2", "3", "4", "5"], // 默认选中工作日

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
  form.startDate = defaultValue.startDate;
  form.endDate = defaultValue.endDate;
  form.wholeDay = defaultValue.wholeDay;
  form.startTime = defaultValue.startTime;
  form.endTime = defaultValue.endTime;
  form.weekType = defaultValue.weekType;
  form.customWeeks = defaultValue.customWeeks;
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
  return await invoke("preview_datetime", { config })
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
    startDate: form.startDate,
    endDate: form.endDate,
    wholeDay: form.wholeDay,
    startTime: form.startTime,
    endTime: form.endTime,
    weekType: form.weekType,
    customWeeks: form.customWeeks,
  });
};
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <!-- 日期范围设置 -->
    <n-form-item label="开始日期">
      <n-date-picker
        v-model:value="form.startDate"
        type="date"
        placeholder="选择开始日期"
      />
    </n-form-item>
    <n-form-item label="结束日期:">
      <n-date-picker
        v-model:value="form.endDate"
        type="date"
        placeholder="选择结束日期"
      />
    </n-form-item>

    <!-- 全天选项 -->
    <n-form-item label="一整天">
      <n-checkbox v-model:checked="form.wholeDay" />
    </n-form-item>

    <!-- 时间范围设置 -->
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

    <!-- 星期选择 -->
    <n-form-item label="星期">
      <div style="display: flex; flex-direction: column">
        <n-radio-group v-model:value="form.weekType">
          <n-radio value="all">全部</n-radio>
          <n-radio value="workday">工作日</n-radio>
          <n-radio value="custom">自定义</n-radio>
        </n-radio-group>
        <n-checkbox-group
          :disabled="form.weekType !== 'custom'"
          v-model:value="form.customWeeks"
          style="margin-top: 10px"
        >
          <n-checkbox value="1">星期一</n-checkbox>
          <n-checkbox value="2">星期二</n-checkbox>
          <n-checkbox value="3">星期三</n-checkbox>
          <n-checkbox value="4">星期四</n-checkbox>
          <n-checkbox value="5">星期五</n-checkbox>
          <n-checkbox value="6">星期六</n-checkbox>
          <n-checkbox value="7">星期日</n-checkbox>
        </n-checkbox-group>
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
