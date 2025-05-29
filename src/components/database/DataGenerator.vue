<script setup>
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, createDiscreteApi } from "naive-ui";
import { ArrowUp, ArrowDown, ArrowLeft, ArrowRight, Copy, Paste, Close } from "@vicons/carbon";
import { datasourceInfosApi, saveDatasourceInfoApi, updateDatasourceInfoApi, deleteDatasourceInfoApi } from '../../db.js';

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi(["message", "dialog", "notification", "loadingBar", "modal"]);


onMounted(async () => {
  datasourceInfos.value = await datasourceInfosApi();
});


const databaseSchemasApi = async (info) => {
  return await invoke("database_schemas", { datasourceInfo: info });
};

const databaseTableTreeApi = async (info) => {
  return await invoke("database_table_tree", { datasourceInfo: info });
};


const datasourceInfos = ref([]);
const datasourceInfo = ref({
  driver: null,
  name: null,
  host: null,
  port: null,
  database: null,
  username: null,
  password: null,
});
const schemas = ref([]);
const tableTreeData = ref();


const current = ref(1);
const currentStatus = ref("process");
const keyword = ref("");
const showIrrelevantNodes = ref(false);
const defaultExpandedKeys = ref(["40", "4030", "403020"]);
const defaultCheckedKeys = ref(["40302010"]);

const updateCheckedKeys = (keys, options, meta) => {
  console.log("updateCheckedKeys", keys, options, meta);
};

const next = () => {
  if (current.value === null)
    current.value = 1;
  else if (current.value >= 3)
    current.value = null;
  else current.value++;
};

const prev = () => {
  if (current.value === 0)
    current.value = null;
  else if (current.value === null)
    current.value = 3;
  else current.value--;
};

const handleSelectDatasource = async () => {
  const info = datasourceInfos.value.find(info => info.name === datasourceInfo.value.name);
  schemas.value = await databaseSchemasApi(info);
};

const handleSelectTable = async () => {
  const info = datasourceInfos.value.find(info => info.name === datasourceInfo.value.name);
  info.database = datasourceInfo.value.database;
  const data = await databaseTableTreeApi(info);
  tableTreeData.value = {
    key: "root",
    label: "root",
    children: data,
  };
  console.log(tableTreeData.value, '--==')

  //   await invoke("fetch_api_data", {
  //   url: url.value
  // }).then((res) => {
  //   apiData.value = JSON.parse(res);
  // }).catch((error) => message.error(error));

};

</script>

<template>
  <n-dialog maskClosable="false" :showIcon="false" @positive-click="handlePositiveClick"
    @negative-click="handleNegativeClick">

    <n-flex vertical>
      <n-steps :current="current" :status="currentStatus" style="display: flex; justify-content: space-around;">
        <n-step title="选择库" />
        <n-step title="配置" />
        <n-step title="预览" />
      </n-steps>

      <template v-if="current === 1">
        <n-form-item label="连接">
          <n-select placeholder="请选择连接信息" label-field="name" value-field="name" v-model:value="datasourceInfo.name"
            :options="datasourceInfos" @update:value="handleSelectDatasource" />
        </n-form-item>
        <n-form-item label="数据库">
          <n-select placeholder="请选择数据库" label-field="name" value-field="name" v-model:value="datasourceInfo.database"
            :options="schemas" @update:value="handleSelectTable" />
        </n-form-item>
      </template>
      <template v-if="current === 2">
        <n-input v-model:value="keyword" placeholder="搜索" />
        <n-scrollbar style="max-height: 90%">
          <!-- <n-tree block-line cascade checkable key-field="key" label-field="label" :selectable="false" -->
          <!--   :data="tableTreeData" :default-expanded-keys="defaultExpandedKeys" -->
          <!--   :default-checked-keys="defaultCheckedKeys" :show-irrelevant-nodes="showIrrelevantNodes" :pattern="keyword" -->
          <!--   @update:checked-keys="updateCheckedKeys" /> -->

          <n-tree block-line cascade checkable :selectable="false" :data="tableTreeData" :pattern="keyword"
            @update:checked-keys="updateCheckedKeys" />
        </n-scrollbar>
      </template>
    </n-flex>


    <template #action>
      <n-button-group>
        <n-button v-if="current > 1" @click="prev">
          <template #icon>
            <n-icon>
              <ArrowLeft />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="next">
          <template #icon>
            <n-icon>
              <ArrowRight />
            </n-icon>
          </template>
        </n-button>
      </n-button-group>
    </template>
  </n-dialog>
</template>

<style lang="scss" scoped></style>
