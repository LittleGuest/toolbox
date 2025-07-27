<script setup>
import { invoke } from "@tauri-apps/api/core";
import { ref, reactive } from "vue";
import { useMessage } from "naive-ui";

// 消息提示
const message = useMessage();

// 生成器默认值
const defaultValue = {
  // 库选择
  database: null,
  // 模式
  schema: null,
  // 表选择
  table: null,
  // 字段选择
  column: null,
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
  form.database = defaultValue.database;
  form.schema = defaultValue.schema;
  form.table = defaultValue.table;
  form.column = defaultValue.column;
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
  previewValue.value = "";
};

// 库
const databases = ref([]);
// 模式
const schemas = ref([]);
// 表
const tables = ref([]);
// 字段
const columns = ref([]);

// 获取库列表
const getDatabases = async () => {
  databases.value = await invoke("database_databases");
};
// 获取模式列表
const getSchemas = async () => {
  schemas.value = await invoke("database_schemas", { database: form.database });
};
// 获取表列表
const getTables = async () => {
  tables.value = await invoke("database_tables", {
    database: form.database,
    schema: form.schema,
  });
};
// 获取字段列表
const getColumns = async () => {
  columns.value = await invoke("database_columns", {
    database: form.database,
    schema: form.schema,
    table: form.table,
  });
};

onMounted(() => {
  getDatabases();
});
</script>

<template>
  <n-form :model="form" label-placement="left" label-width="180">
    <!-- 库选择 -->
    <n-form-item path="database" label="库">
      <n-select
        placeholder="选择库"
        v-model:value="form.database"
        :options="databases"
        clearable
        filterable
      />
    </n-form-item>
    <!-- 模式选择 -->
    <n-form-item path="schema" label="模式">
      <n-select
        placeholder="选择模式"
        v-model:value="form.schema"
        :options="schemas"
        clearable
        filterable
      />
    </n-form-item>

    <!-- 表选择 -->
    <n-form-item path="table" label="表">
      <n-select
        placeholder="选择表"
        v-model:value="form.table"
        :options="tables"
        clearable
        filterable
      />
    </n-form-item>

    <!-- 字段选择 -->
    <n-form-item path="column" label="字段">
      <n-select
        placeholder="选择字段"
        v-model:value="form.column"
        :options="columns"
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
