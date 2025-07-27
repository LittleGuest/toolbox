<script setup>
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive, watch } from "vue";
import { useMessage } from "naive-ui";

// 消息提示
const message = useMessage();

// 生成器默认值
const defaultValue = {
  start: 0, // 开始值
  end: 1000, // 结束值
  numberType: "integer", // 数字类型
  decimalPlaces: 2, // 小数位数

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
  form.start = defaultValue.start;
  form.end = defaultValue.end;
  form.decimalPlaces = defaultValue.decimalPlaces;
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
  return await invoke("preview_number", { config })
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
    start: form.start,
    end: form.end,
    numberType: form.numberType,
    decimalPlaces: form.decimalPlaces,
  });
};

// 监听数字类型变化，控制小数位数可用性
watch(
  () => form.numberType,
  (newType) => {
    if (newType === "integer") {
      // 切换到整数类型时，确保开始/结束值为整数
      form.start = Math.floor(form.start);
      form.end = Math.ceil(form.end);
    }
  }
);
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <!-- 开始值 -->
    <n-form-item label="开始">
      <n-input-number
        v-model:value="form.start"
        :min="-999999999"
        :max="form.end - 1"
        placeholder="开始值"
      />
    </n-form-item>

    <!-- 结束值 -->
    <n-form-item label="结束">
      <n-input-number
        v-model:value="form.end"
        :min="form.start + 1"
        :max="999999999"
        placeholder="结束值"
      />
    </n-form-item>

    <!-- 数字类型选择 -->
    <n-form-item label="数字类型:">
      <n-radio-group v-model:value="form.numberType">
        <n-radio value="integer">整数</n-radio>
        <n-radio value="decimal">小数</n-radio>
      </n-radio-group>
    </n-form-item>
    <!-- 小数位数 (仅小数类型显示) -->
    <n-form-item label="小数位数">
      <n-input-number
        :disabled="form.numberType !== 'decimal'"
        v-model:value="form.decimalPlaces"
        :min="1"
        :max="10"
        :step="1"
        placeholder="小数位数"
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
