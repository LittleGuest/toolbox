<script setup lang="ts">
import { reactive } from "vue";

const props = defineProps({
  referenceOptions: {
    type: Array,
    default: () => [],
  },
});

// 生成器默认值
const defaultValue = {
  referenceKey: null,
  referenceSchema: null,
  referenceTable: null,
  referenceColumn: null,
  // 生成模式（random/unique/repeat）
  generateMode: "random",
  // 重复范围
  repeatFrom: 1,
  repeatTo: 3,

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
  form.referenceKey = defaultValue.referenceKey;
  form.referenceSchema = defaultValue.referenceSchema;
  form.referenceTable = defaultValue.referenceTable;
  form.referenceColumn = defaultValue.referenceColumn;
  form.generateMode = defaultValue.generateMode;
  form.repeatFrom = defaultValue.repeatFrom;
  form.repeatTo = defaultValue.repeatTo;
  form.includeDefault = defaultValue.includeDefault;
  form.defaultValue = defaultValue.defaultValue;
  form.defaultPercentage = defaultValue.defaultPercentage;
  form.includeNull = defaultValue.includeNull;
  form.nullPercentage = defaultValue.nullPercentage;
  form.unique = defaultValue.unique;
  form.forbiddenLinks = defaultValue.forbiddenLinks;
};

const syncReference = () => {
  const option = props.referenceOptions.find(
    (item) => item.value === form.referenceKey
  );
  form.referenceSchema = option?.schema || null;
  form.referenceTable = option?.tableName || null;
  form.referenceColumn = option?.column || null;
};

defineExpose({
  getConfig: () => {
    syncReference();
    return { ...form };
  },
  setConfig: (config = {}) => {
    Object.assign(form, config);
    if (!form.referenceKey && form.referenceSchema && form.referenceTable && form.referenceColumn) {
      form.referenceKey = `${form.referenceSchema}#${form.referenceTable}#${form.referenceColumn}`;
    }
    syncReference();
  },
});
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <n-form-item path="referenceKey" label="引用字段">
      <n-select
        placeholder="选择画布中已配置的字段"
        v-model:value="form.referenceKey"
        :options="referenceOptions"
        @update:value="syncReference"
        clearable
        filterable
      />
    </n-form-item>

    <!-- 生成模式 -->
    <n-form-item label="生成模式">
      <n-radio-group v-model:value="form.generateMode">
        <n-space direction="vertical">
          <n-radio value="random">随机</n-radio>
          <n-radio value="unique">不重复</n-radio>
          <n-radio value="repeat">
            <n-space>
              <span>重复每个值</span>
              <n-input-number
                :disabled="form.generateMode !== 'repeat'"
                v-model:value="form.repeatFrom"
                :min="1"
                style="width: 80px"
                placeholder="从"
              />
              <span>到</span>
              <n-input-number
                :disabled="form.generateMode !== 'repeat'"
                v-model:value="form.repeatTo"
                :min="form.repeatFrom"
                style="width: 80px"
                placeholder="到"
              />
              <span>次</span>
            </n-space>
          </n-radio>
        </n-space>
      </n-radio-group>
    </n-form-item>

    <n-form-item label=" ">
      <n-button @click="reset"> 重置属性 </n-button>
    </n-form-item>
  </n-form>
</template>

<style scoped></style>
