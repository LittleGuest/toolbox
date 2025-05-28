<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Database from '@tauri-apps/plugin-sql';
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { NButton, NButtonGroup, createDiscreteApi } from "naive-ui";
import { ArrowUp, ArrowDown, Copy, Paste, Close } from "@vicons/carbon";
import { repeat } from "seemly";

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi(["message", "dialog", "notification", "loadingBar", "modal"]);


const connects = ref([]);

const loadDatabase = async () => {
  return await Database.load('sqlite:toolbox.db');
};


const connectsApi = async () => {
  const db = await loadDatabase();
  return await db.select('select * from database_connect');
};

const saveApi = async (connect) => {
  const db = await loadDatabase();
  await db.execute(
    "insert into database_connect (driver, name, host, port, database, username, password) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    [connect.driver, connect.name, connect.host, connect.port, connect.database, connect.username, connect.password],
  );
};

const updateApi = async (connect) => {
  const db = await loadDatabase();
  await db.execute(
    "update database_connect set driver=$1, name=$2, host=$3, port=$4, database=$5, username=$6, password=$7 where id=$8",
    [connect.driver, connect.name, connect.host, connect.port, connect.database, connect.username, connect.password, connect.id],
  );
};

const deleteApi = async (id) => {
  const db = await loadDatabase();
  await db.execute(
    "delete from database_connect where id=$1",
    [id],
  );
};



onMounted(async () => {
  connects.value = await connectsApi();
});


const keyword = ref("");
const showIrrelevantNodes = ref(false);

const defaultExpandedKeys = ref(["40", "4030", "403020"]);
const defaultCheckedKeys = ref(["40302010"]);
const updateCheckedKeys = (keys, options, meta) => {
  console.log("updateCheckedKeys", keys, options, meta);
};



function createLabel(level) {
  if (level === 4)
    return "道生一";
  if (level === 3)
    return "一生二";
  if (level === 2)
    return "二生三";
  if (level === 1)
    return "三生万物";
  return "";
}

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
              await deleteApi(row.id);
              message.success("删除成功");
              connects.value = await connectsApi();
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

// 保存连接信息
const saveConnect = (e) => {
  e.preventDefault();
  formRef.value?.validate(async (errors) => {
    if (!errors) {
      console.log(addDrawer, '==-=-=-=-=')
      if (addDrawer.value) {
        message.success("添加成功");
        await saveApi(model.value);
      } else {
        message.success("编辑成功");
        await updateApi(model.value);
      }
      addDrawer.value = false;
      showAddaDrawer.value = false;
      connects.value = await connectsApi();
    }
  });
};

</script>

<template>
  <n-button @click="handleAddDrawer">新建连接</n-button>
  <n-data-table :columns="columns" :data="connects" :bordered="false" :scroll-x="1800" :max-height="550" />

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
        <n-button @click="saveConnect">保存</n-button>
      </template>
    </n-drawer-content>
  </n-drawer>


  <!-- <n-flex justify="space-between"> -->
  <!--   <n-flex> -->
  <!--     <n-input v-model:value="keyword" placeholder="搜索" /> -->
  <!--     <n-scrollbar style="max-height: 90%"> -->
  <!--       <n-tree block-line cascade checkable key-field="key" label-field="label" :selectable="false" :data="treeData" -->
  <!--         :default-expanded-keys="defaultExpandedKeys" :default-checked-keys="defaultCheckedKeys" -->
  <!--         :show-irrelevant-nodes="showIrrelevantNodes" :pattern="keyword" @update:checked-keys="updateCheckedKeys" /> -->
  <!--     </n-scrollbar> -->
  <!--   </n-flex> -->
  <!---->
  <!--   <div> -->
  <!---->
  <!--     {{ value }} -->
  <!--     <n-button>Oops!</n-button> -->
  <!--     <n-button>Oops!</n-button> -->
  <!--     <n-button>Oops!</n-button> -->
  <!--   </div> -->
  <!-- </n-flex> -->
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
