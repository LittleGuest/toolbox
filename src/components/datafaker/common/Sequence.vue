<script setup>
import { ref, reactive, computed } from "vue";
import { useMessage } from "naive-ui";

// 消息提示
const message = useMessage();

// 生成器默认值
const defaultValue = {
  start: 1, // 开始值
  step: 1, // 递增值
  minEnabled: false, // 最小值是否启用
  min: 1, // 最小值
  maxEnabled: false, // 最大值是否启用
  max: 2147483647, // 最大值
  cycle: false, // 循环
};
// 表单数据
const form = reactive({
  ...defaultValue,
});

// 重置属性
const reset = () => {
  form.start = defaultValue.start;
  form.step = defaultValue.step;
  form.minEnabled = defaultValue.minEnabled;
  form.min = defaultValue.min;
  form.maxEnabled = defaultValue.maxEnabled;
  form.max = defaultValue.max;
  form.cycle = defaultValue.cycle;
  previewValue.value = "";
};

// 预览数据
const previewValue = ref("");
// 预览API
const previewApi = async (config) => {
  return await invoke("preview_sequence", { config })
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
    step: form.step,
    minEnabled: form.minEnabled,
    min: form.min,
    maxEnabled: form.maxEnabled,
    max: form.max,
    cycle: form.cycle,
  });
};
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <n-form-item path="start" label="开始值">
      <n-input-number v-model:value="form.start" placeholder="输入开始值" />
    </n-form-item>

    <n-form-item path="step" label="递增值">
      <n-input-number v-model:value="form.step" placeholder="输入递增值" />
    </n-form-item>

    <n-form-item path="min" label="最小">
      <n-flex align="center">
        <n-checkbox v-model:checked="form.minEnabled" />
        <n-input-number
          v-model:value="form.min"
          :disabled="!form.minEnabled"
          placeholder="输入最小值"
        />
      </n-flex>
    </n-form-item>
    <n-form-item path="max" label="最大">
      <n-flex align="center">
        <n-checkbox v-model:checked="form.maxEnabled" />
        <n-input-number
          v-model:value="form.max"
          :disabled="!form.maxEnabled"
          placeholder="输入最大值"
        />
      </n-flex>
    </n-form-item>
    <n-form-item path="cycle" label="循环">
      <n-checkbox v-model:checked="form.cycle" />
    </n-form-item>

    <!-- 预览 -->
    <n-form-item path="previewValue" label="预览">
      <n-input v-model:value="previewValue" readonly placeholder="" />
      <n-button @click="preview">刷新</n-button>
    </n-form-item>

    <n-form-item label=" ">
      <n-button @click="reset"> 重置属性 </n-button>
    </n-form-item>
  </n-form>
</template>

<style scoped>
.card-container {
  padding: 24px;
}
.form-item {
  margin-bottom: 16px;
}
</style>
