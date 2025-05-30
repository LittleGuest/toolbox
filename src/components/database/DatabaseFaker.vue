<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, createDiscreteApi } from "naive-ui";
import { datasourceInfosApi, saveDatasourceInfoApi, updateDatasourceInfoApi, deleteDatasourceInfoApi } from '../../db.js';
import DataGenerator from './DataGenerator.vue';

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi(["message", "dialog", "notification", "loadingBar", "modal"]);

const connects = ref([]);

onMounted(async () => {
  connects.value = await datasourceInfosApi();
});

const columns = [
  {
    title: "连接名称",
    key: "name"
  },
  {
    title: "主机",
    key: "host"
  },
  {
    title: "端口",
    key: "port"
  },
  {
    title: "数据库",
    key: "database"
  },
  {
    title: "操作",
    key: "oper",
    width: 220,
    fixed: "right",
    render(row) {
      return h(NButtonGroup, [
        h(
          NButton,
          {
            onClick: () => {

            }
          },
          { default: () => "数据生成" }
        ),
        h(
          NButton,
          {
            onClick: () => {
              addDrawer.value = false;
              showAddaDrawer.value = true;
              model.value = row;
            }
          },
          { default: () => "编辑" }
        ),
        h(
          NButton,
          {
            onClick: async () => {
              await deleteDatasourceInfoApi(row.id);
              message.success("删除成功");
              connects.value = await datasourceInfosApi();
            }
          },
          { default: () => "删除" }
        )
      ]);
    }
  }
];

// 展示添加/编辑抽屉
const showAddaDrawer = ref(false);
const addDrawer = ref(true);
const formRef = ref(null);
const modelRef = ref({
  id: null,
  driver: null,
  name: null,
  host: null,
  port: null,
  database: null,
  username: null,
  password: null,
});
const model = modelRef;
const rules = {
  driver: [
    {
      required: true,
      message: "请选择类型"
    }
  ],
  name: [
    {
      required: true,
      message: "请输入连接名称"
    }
  ],
  host: [
    {
      required: true,
      message: "请输入主机地址"
    }
  ],
};
const driverOptions = [
  {
    label: "PostgreSql",
    value: "postgresql"
  },
  {
    label: "MySQL",
    value: "mysql"
  },
  {
    label: "SQLite",
    value: "sqlite"
  },
];

const handleAddDrawer = () => {
  addDrawer.value = true;
  showAddaDrawer.value = true;
  modelRef.value = {};
};


const pingApi = async (info) => {
  await invoke("database_ping", { datasourceInfo: info }).then(res => {
    message.success("连接成功")
  }).catch(err => {
    message.error("连接失败")
  });
};

const ping = async () => {
  await pingApi(model.value);
}

// 保存连接信息
const saveConnect = (e) => {
  e.preventDefault();
  formRef.value?.validate(async (errors) => {
    if (!errors) {
      console.log(addDrawer, '==-=-=-=-=')
      if (addDrawer.value) {
        message.success("添加成功");
        await saveDatasourceInfoApi(model.value);
      } else {
        message.success("编辑成功");
        await updateDatasourceInfoApi(model.value);
      }
      addDrawer.value = false;
      showAddaDrawer.value = false;
      connects.value = await datasourceInfosApi();
    }
  });
};

</script>

<template>
  <n-button @click="handleAddDrawer">新建连接</n-button>
  <n-data-table :columns="columns" :data="connects" :bordered="false" :scroll-x="1800" :max-height="550" />

  <DataGenerator />

  <n-drawer v-model:show="showAddaDrawer" placement="bottom" resizable :default-width="502" :default-height="600">
    <n-drawer-content :title="addDrawer ? '添加' : '编辑'" closable>
      <n-form ref="formRef" :model="model" :rules="rules" label-placement="left" label-width="auto"
        require-mark-placement="right-hanging">
        <n-form-item path="driver" label="类型">
          <n-select placeholder="请选择类型" v-model:value="model.driver" :options="driverOptions" />
        </n-form-item>
        <n-form-item path="name" label="连接名称">
          <n-input placeholder="请输入连接名称" v-model:value="model.name" clearable />
        </n-form-item>
        <n-form-item path="host" label="主机">
          <n-input placeholder="请输入主机名" v-model:value="model.host" clearable />
        </n-form-item>
        <n-form-item path="port" label="端口">
          <n-input-number placeholder="请输入端口" v-model:value="model.port" clearable />
        </n-form-item>
        <n-form-item path="database" label="数据库">
          <n-input placeholder="请输入数据库" v-model:value="model.database" clearable />
        </n-form-item>
        <n-form-item path="username" label="用户名">
          <n-input placeholder="请输入用户名" v-model:value="model.username" clearable />
        </n-form-item>
        <n-form-item path="password" label="密码">
          <n-input placeholder="请输入密码" v-model:value="model.password" type="password" clearable
            @input="handlePasswordInput" @keydown.enter.prevent />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-button @click="ping">测试连接</n-button>
        <n-button @click="saveConnect">保存</n-button>
      </template>
    </n-drawer-content>
  </n-drawer>


</template>

<style lang="scss" scoped>
.light-green {
  height: 100%;
  background-color: rgba(0, 128, 0, 0.12);
}

.green {
  height: 100%;
  background-color: rgba(0, 128, 0, 0.24);
}
</style>
