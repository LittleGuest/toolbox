<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";

const message = useMessage();

const props = defineProps({
  datasource: {
    type: Array,
    required: true,
  },
  showDrawer: {
    type: Boolean,
    required: true,
  },
});
const emits = defineEmits(["closeDrawer"]);

const show = computed({
  get: () => props.showDrawer,
  set: (value) => {
    if (!value) {
      emits("closeDrawer");
    }
  },
});

const form = ref({
  datasourceKey: "",
  language: "java",
  tableNames: [] as string[],
  fileTypes: [
    "entity.java",
    "mapper.java",
    "mapper.xml",
    "service.java",
    "serviceImpl.java",
    "controller.java",
  ],
  packageNames: {
    "entity.java": "com.example.entity",
    "mapper.java": "com.example.mapper",
    "service.java": "com.example.service",
    "serviceImpl.java": "com.example.service.impl",
    "controller.java": "com.example.controller",
  },
});

const tableOptions = ref<{ label: string; value: string; key: string }[]>([]);
const loadingTables = ref(false);
const generating = ref(false);
const generatedCodes = ref<Record<string, string>>({});
const activeTab = ref("");
const tableLoadMessage = ref("");
const tableLoadSeq = ref(0);

const datasourceKey = (item, index) =>
  item.id !== null && item.id !== undefined
    ? `id:${item.id}`
    : `fallback:${index}:${item.driver || ""}:${item.host || ""}:${item.database || ""}:${item.name || ""}`;

const datasourceOptions = computed(() =>
  props.datasource.map((item, index) => ({
    label: `${item.database || ""}#${item.name || ""}`,
    value: datasourceKey(item, index),
  })),
);

const languageOptions = [
  { label: "Java", value: "java" },
  { label: "Rust", value: "rust" },
];

const javaFileOptions = [
  { label: "Entity", value: "entity.java" },
  { label: "Mapper", value: "mapper.java" },
  { label: "Mapper XML", value: "mapper.xml" },
  { label: "Service", value: "service.java" },
  { label: "Service Impl", value: "serviceImpl.java" },
  { label: "Controller", value: "controller.java" },
];

const rustFileOptions = [{ label: "Model", value: "model.rs" }];

const fileOptions = computed(() =>
  form.value.language === "rust" ? rustFileOptions : javaFileOptions,
);

const getFileLanguage = (file) => {
  if (file.endsWith(".java")) return "java";
  if (file.endsWith(".xml")) return "xml";
  if (file.endsWith(".rs")) return "rust";
  return "text";
};

const selectedDatasource = computed(() =>
  props.datasource.find((item, index) => datasourceKey(item, index) === form.value.datasourceKey),
);

const packageFileTypes = computed(() =>
  form.value.fileTypes.filter((file) => file.endsWith(".java")),
);

const ensureDatasourceSelected = () => {
  const list = props.datasource;
  if (!list.length) {
    form.value.datasourceKey = "";
    tableOptions.value = [];
    tableLoadMessage.value = "暂无数据源，请先新建连接";
    return false;
  }

  const currentExists = list.some(
    (item, index) => datasourceKey(item, index) === form.value.datasourceKey,
  );
  if (!currentExists) {
    form.value.datasourceKey = datasourceKey(list[0], 0);
  }
  return true;
};

const reloadTables = async () => {
  if (!props.showDrawer || !ensureDatasourceSelected()) {
    return;
  }
  form.value.tableNames = [];
  generatedCodes.value = {};
  await loadTables();
};

watch(
  () => props.datasource,
  async () => {
    await reloadTables();
  },
  { immediate: true, deep: true },
);

watch(
  () => form.value.datasourceKey,
  async () => {
    await reloadTables();
  },
);

watch(
  () => props.showDrawer,
  async (visible) => {
    if (visible) {
      ensureDatasourceSelected();
      await loadTables();
    }
  },
);

watch(
  () => form.value.language,
  (language) => {
    form.value.fileTypes =
      language === "rust"
        ? ["model.rs"]
        : [
            "entity.java",
            "mapper.java",
            "mapper.xml",
            "service.java",
            "serviceImpl.java",
            "controller.java",
          ];
    generatedCodes.value = {};
  },
);

const loadTables = async () => {
  const seq = tableLoadSeq.value + 1;
  tableLoadSeq.value = seq;
  const datasourceInfo = selectedDatasource.value;
  if (!datasourceInfo) {
    tableOptions.value = [];
    tableLoadMessage.value = "请先选择数据源";
    return;
  }

  loadingTables.value = true;
  tableLoadMessage.value = "";
  try {
    const tables = await invoke("database_table_tree", {
      datasourceInfo,
    });
    if (seq !== tableLoadSeq.value) {
      return;
    }
    tableOptions.value = tables.map((table) => {
      const value = table.schema ? `${table.schema}.${table.tableName}` : table.tableName;
      const comment = table.tableComment ? `（${table.tableComment}）` : "";
      return {
        label: `${value}${comment}`,
        key: value,
        value,
      };
    });
    tableLoadMessage.value = tableOptions.value.length
      ? ""
      : "当前数据源未读取到表，请检查连接和数据库权限";
  } catch (error) {
    if (seq !== tableLoadSeq.value) {
      return;
    }
    tableOptions.value = [];
    tableLoadMessage.value = `加载表失败: ${error}`;
    message.error(`加载表失败: ${error}`);
  } finally {
    if (seq === tableLoadSeq.value) {
      loadingTables.value = false;
    }
  }
};

const handleGenerate = async () => {
  const datasourceInfo = selectedDatasource.value;
  if (!datasourceInfo) {
    message.warning("请选择数据源");
    return;
  }
  if (form.value.fileTypes.length === 0) {
    message.warning("请选择要生成的文件");
    return;
  }

  generating.value = true;
  try {
    const res = await invoke("generate_code_from_db", {
      request: {
        datasourceInfo,
        language: form.value.language,
        tableNames: form.value.tableNames,
        fileTypes: form.value.fileTypes,
        packageNames: form.value.packageNames,
      },
    });
    if (res.success) {
      generatedCodes.value = res.data;
      activeTab.value = Object.keys(res.data)[0] || "";
      message.success("代码生成成功");
    } else {
      message.error(res.message || "代码生成失败");
    }
  } catch (error) {
    message.error(`代码生成出错: ${error}`);
  } finally {
    generating.value = false;
  }
};

const copyCode = async (code) => {
  await writeText(code);
  message.success("已复制");
};
</script>

<template>
  <n-drawer v-model:show="show" placement="right" resizable default-width="70%" :default-height="600">
    <n-drawer-content title="数据库代码生成" closable>
      <n-form label-placement="left" label-width="90" require-mark-placement="right-hanging">
        <n-form-item label="数据源">
          <n-select placeholder="请选择数据源" v-model:value="form.datasourceKey" :options="datasourceOptions" />
        </n-form-item>
        <n-form-item label="语言">
          <n-radio-group v-model:value="form.language">
            <n-radio-button v-for="item in languageOptions" :key="item.value" :value="item.value">
              {{ item.label }}
            </n-radio-button>
          </n-radio-group>
        </n-form-item>
        <n-form-item label="表">
          <n-select v-model:value="form.tableNames" multiple filterable clearable
            placeholder="请选择表，不选择则生成全部表" :loading="loadingTables" :options="tableOptions"
            :disabled="!selectedDatasource || loadingTables" />
          <n-text v-if="tableLoadMessage" class="table-load-message" depth="3">
            {{ tableLoadMessage }}
          </n-text>
        </n-form-item>
        <n-form-item label="生成文件">
          <n-checkbox-group v-model:value="form.fileTypes">
            <n-space>
              <n-checkbox v-for="item in fileOptions" :key="item.value" :value="item.value">
                {{ item.label }}
              </n-checkbox>
            </n-space>
          </n-checkbox-group>
        </n-form-item>
        <template v-for="file in packageFileTypes" :key="file">
          <n-form-item :label="`${file} 包名`">
            <n-input v-model:value="form.packageNames[file]" placeholder="请输入包名，例如 com.example.demo" />
          </n-form-item>
        </template>
      </n-form>

      <n-empty v-if="Object.keys(generatedCodes).length === 0" description="暂无生成结果" />
      <n-tabs v-else v-model:value="activeTab" type="line" animated>
        <n-tab-pane v-for="(code, file) in generatedCodes" :key="file" :name="file" :tab="file">
          <div class="code-actions">
            <n-tag size="small">{{ getFileLanguage(file) }}</n-tag>
            <n-button size="small" @click="copyCode(code)">复制</n-button>
          </div>
          <pre class="code-block"><code>{{ code }}</code></pre>
        </n-tab-pane>
      </n-tabs>

      <template #footer>
        <n-button @click="emits('closeDrawer')">取消</n-button>
        <n-button type="primary" :loading="generating" @click="handleGenerate">生成代码</n-button>
      </template>
    </n-drawer-content>
  </n-drawer>
</template>

<style lang="scss" scoped>
.m-p {
  margin-top: 0 !important;
}

.block {
  border: 1px solid #e8eaec;
  box-sizing: border-box;
  margin-bottom: 30px;
  padding: 0px 20px;
  border-radius: 3px;
}

.block-child {
  box-sizing: border-box;
  margin-left: 40px;
  padding: 0px 20px;
}

h3 {
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

h3>.m-span:last-child {
  color: #515a6e;
}

h3 .mark1,
h3 .mark2 {
  display: inline-block;
  font-size: 15px;
  color: #ffffff;
  box-sizing: border-box;
  padding: 2px 4px;
  margin-right: 4px;
  border-radius: 4px;
}

h3 .mark1 {
  background-color: #2b85e4;
}

h3 .mark2 {
  background-color: #5cadff;
  margin-left: 4px;
}

.block-content .m-p {
  color: #515a6e;
  font-size: 15px;
}

.block-content .m-p .m-span {
  display: inline-block;
  color: #17233d;
  box-sizing: border-box;
  padding-right: 5px;
}

.code-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-bottom: 8px;
}

.code-block {
  max-height: calc(100vh - 360px);
  overflow: auto;
  padding: 16px;
  background: #0f172a;
  color: #e2e8f0;
  border-radius: 6px;
  font-size: 12px;
  line-height: 1.6;
}

.table-load-message {
  margin-left: 12px;
  white-space: nowrap;
}
</style>
