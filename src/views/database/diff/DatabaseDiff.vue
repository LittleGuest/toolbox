<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, useMessage } from "naive-ui";
import {
  datasourceInfosApi,
  saveDatasourceInfoApi,
  updateDatasourceInfoApi,
  deleteDatasourceInfoApi,
} from "@/store/db.js";
import DatabaseDiffReport from "./DatabaseDiffReport.vue";
import { QuestionCircleOutlined } from "@vicons/antd";
import DatabaseDiffSql from "./DatabaseDiffSql.vue";
import DatabaseStandardCheck from "./DatabaseStandardCheck.vue";
import DatabaseGeneratorCode from "./DatabaseGeneratorCode.vue";

const message = useMessage();

const connects = ref([]);

onMounted(async () => {
  connects.value = await datasourceInfosApi();

  sourceTables.value = connects.value.map((c) => {
    return {
      label: c.database + "#" + c.name,
      value: c.database + "#" + c.name,
    };
  });

  targetTables.value = connects.value.map((c) => {
    return {
      label: c.database + "#" + c.name,
      value: c.database + "#" + c.name,
    };
  });

  standardCheckCodeList.value = await databaseStandardCheckApi();
});

const columns = [
  {
    title: "连接名称",
    key: "name",
  },
  {
    title: "主机",
    key: "host",
  },
  {
    title: "端口",
    key: "port",
  },
  {
    title: "数据库",
    key: "database",
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
              ping(row);
            },
          },
          { default: () => "测试" },
        ),
        h(
          NButton,
          {
            onClick: () => {
              addDrawer.value = false;
              showAddaDrawer.value = true;
              model.value = row;
            },
          },
          { default: () => "编辑" },
        ),
        h(
          NButton,
          {
            onClick: async () => {
              await deleteDatasourceInfoApi(row.id);
              message.success("删除成功");
              connects.value = await datasourceInfosApi();
            },
          },
          { default: () => "删除" },
        ),
      ]);
    },
  },
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
      message: "请选择类型",
    },
  ],
  name: [
    {
      required: true,
      message: "请输入连接名称",
    },
  ],
  host: [
    {
      required: true,
      message: "请输入主机地址",
    },
  ],
  database: [
    {
      required: true,
      message: "请输入数据库",
    },
  ],
};
const driverOptions = [
  {
    label: "PostgreSql",
    value: "postgresql",
  },
  {
    label: "MySQL",
    value: "mysql",
  },
  {
    label: "SQLite",
    value: "sqlite",
  },
];

const handleAddDrawer = () => {
  addDrawer.value = true;
  showAddaDrawer.value = true;
  modelRef.value = {};
};

const pingApi = async (info) => {
  await invoke("database_ping", { datasourceInfo: info })
    .then((res) => {
      message.success("连接成功");
    })
    .catch((err) => {
      message.error("连接失败");
    });
};

const databaseStandardCheckApi = async () => {
  return await invoke("database_standard_check_codes", {})
    .then((res) => {
      return res;
    })
    .catch((error) => message.error(error));
};

const ping = async (info) => {
  await pingApi(info);
};

// 保存连接信息
const saveConnect = (e) => {
  e.preventDefault();
  formRef.value?.validate(async (errors) => {
    if (!errors) {
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

const sourceTables = ref();
const targetTables = ref([]);
const reportSourceTable = ref();
const reportTargetTable = ref();
const sourceDatasourceInfo = ref();
const targetDatasourceInfo = ref();
const showDiffReportDrawer = ref(false);
const showDiffSqlDrawer = ref(false);
const sqlSourceTable = ref();
const sqlTargetTable = ref();
const standardCheckTable = ref();
const showStandardCheckDrawer = ref(false);
const showCustomCheckDrawer = ref(false);
const customStandardChecked = ref([]);
const standardCheckCodeList = ref([]);
const standardCheckCodes = ref([]);
const showGeneratorCodeDrawer = ref(false);

const generateReport = () => {
  if (!reportSourceTable.value || !reportTargetTable.value) {
    message.error("请选择基准库和变动库");
    return;
  }
  sourceDatasourceInfo.value = connects.value.find(
    (info) => info.database + "#" + info.name === reportSourceTable.value,
  );
  targetDatasourceInfo.value = connects.value.find(
    (info) => info.database + "#" + info.name === reportTargetTable.value,
  );
  showDiffReportDrawer.value = true;
};

const closeDrawer = () => {
  showDiffReportDrawer.value = false;
  showDiffSqlDrawer.value = false;
  showStandardCheckDrawer.value = false;
  showCustomCheckDrawer.value = false;
};

const generateSql = (type) => {
  if (!sqlSourceTable.value || !sqlTargetTable.value) {
    message.error("请选择基准库和变动库");
    return;
  }
  sourceDatasourceInfo.value = connects.value.find(
    (info) => info.database + "#" + info.name === sqlSourceTable.value,
  );
  targetDatasourceInfo.value = connects.value.find(
    (info) => info.database + "#" + info.name === sqlTargetTable.value,
  );
  showDiffSqlDrawer.value = true;
};

const generateCheck = (custom) => {
  if (!standardCheckTable.value) {
    message.error("请选择基准库");
    return;
  }
  sourceDatasourceInfo.value = connects.value.find(
    (info) => info.database + "#" + info.name === standardCheckTable.value,
  );
  if (custom) {
    standardCheckCodes.value = customStandardChecked.value;
  } else {
    standardCheckCodes.value = standardCheckCodeList.value.map(c => c.code);
  }
  showCustomCheckDrawer.value = false;
  showStandardCheckDrawer.value = true;
};

const generateCode = () => {
  showGeneratorCodeDrawer.value = true;
};

const showCheck = () => {
  if (!standardCheckTable.value) {
    message.error("请选择基准库");
    return;
  }
  sourceDatasourceInfo.value = connects.value.find(
    (info) => info.database + "#" + info.name === standardCheckTable.value,
  );
  showCustomCheckDrawer.value = true;
};
</script>

<template>
  <n-form inline :label-width="80" label-placement="left" class="opt">
    <n-form-item>
      差异报告
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-icon size="18">
            <QuestionCircleOutlined />
          </n-icon>
        </template>
        对比两个数据库之间的差异变化，用于评审检查数据库的变动
      </n-tooltip>
    </n-form-item>
    <n-form-item label="基准表">
      <n-select placeholder="请选择基准表" v-model:value="reportSourceTable" :options="sourceTables" />
    </n-form-item>
    <n-form-item label="变动表">
      <n-select placeholder="请选择目标表" v-model:value="reportTargetTable" :options="targetTables" />
    </n-form-item>
    <n-form-item>
      <n-button @click="generateReport">生成</n-button>
    </n-form-item>
  </n-form>
  <n-form inline :label-width="80" label-placement="left" class="opt">
    <n-form-item>
      差异SQL
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-icon size="18">
            <QuestionCircleOutlined />
          </n-icon>
        </template>
        对比基准库之后,生成的差异sql，在变化库上执行即可补齐差异。
        （注意：sql语句仅供参考，执行前应当检查一下sql，出现数据丢失一概不负责）
      </n-tooltip>
    </n-form-item>
    <n-form-item label="基准表">
      <n-select placeholder="请选择基准表" v-model:value="sqlSourceTable" :options="sourceTables" />
    </n-form-item>
    <n-form-item label="变动表">
      <n-select placeholder="请选择目标表" v-model:value="sqlTargetTable" :options="targetTables" />
    </n-form-item>
    <n-form-item>
      <n-button @click="generateSql('struct')">结构差异</n-button>
      <!-- <n-button @click="generateSql('data')">数据差异</n-button> -->
    </n-form-item>
  </n-form>
  <n-form inline :label-width="80" label-placement="left" class="opt">
    <n-form-item>
      规范检查
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-icon size="18">
            <QuestionCircleOutlined />
          </n-icon>
        </template>
        对基准库的数据库设计进行规范检查
      </n-tooltip>
    </n-form-item>
    <n-form-item label="基准表">
      <n-select placeholder="请选择基准表" v-model:value="standardCheckTable" :options="sourceTables" />
    </n-form-item>
    <n-form-item>
      <n-button @click="generateCheck(false)">检查</n-button>
      <n-button @click="showCheck">自定义检查</n-button>
    </n-form-item>
  </n-form>
  <n-form inline :label-width="80" label-placement="left">
    <n-form-item>
      逆向生成
      <n-tooltip trigger="hover">
        <template #trigger>
          <n-icon size="18">
            <QuestionCircleOutlined />
          </n-icon>
        </template>
        一键生成entity.java，mapper.java，mapper.xml，service.java，serviceImpl.java，controller.java文件
      </n-tooltip>
    </n-form-item>
    <n-form-item>
      <n-button @click="generateCode()">生成</n-button>
    </n-form-item>
  </n-form>

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
          <n-input-number placeholder="请输入端口" v-model:value="model.port" clearable min="0" max="65535" />
        </n-form-item>
        <n-form-item path="username" label="用户名">
          <n-input placeholder="请输入用户名" v-model:value="model.username" clearable />
        </n-form-item>
        <n-form-item path="password" label="密码">
          <n-input placeholder="请输入密码" v-model:value="model.password" type="password" clearable
            @input="handlePasswordInput" @keydown.enter.prevent />
        </n-form-item>
        <n-form-item path="database" label="数据库">
          <n-input placeholder="请输入数据库" v-model:value="model.database" clearable />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-button @click="ping(model)">测试连接</n-button>
        <n-button @click="saveConnect">保存</n-button>
      </template>
    </n-drawer-content>
  </n-drawer>

  <n-drawer v-model:show="showCustomCheckDrawer" placement="bottom" resizable :default-width="502"
    :default-height="600">
    <n-drawer-content title="自定义规范检查" closable>
      <n-form label-placement="left" label-width="auto" require-mark-placement="right-hanging">
        <n-form-item label="">
          <n-checkbox-group v-model:value="customStandardChecked">
            <n-space style="flex-flow: column;">
              <n-checkbox v-for="code in standardCheckCodeList" :value="code.code"> {{ code.desc }} </n-checkbox>
            </n-space>
          </n-checkbox-group>
        </n-form-item>
      </n-form>
      <template #footer>
        <n-button @click="closeDrawer()">取消</n-button>
        <n-button @click="generateCheck(true)">保存</n-button>
      </template>
    </n-drawer-content>
  </n-drawer>

  <DatabaseDiffReport v-if="showDiffReportDrawer" :source="sourceDatasourceInfo" :target="targetDatasourceInfo"
    :showDrawer="showDiffReportDrawer" @closeDrawer="closeDrawer" />
  <DatabaseDiffSql v-if="showDiffSqlDrawer" :source="sourceDatasourceInfo" :target="targetDatasourceInfo"
    :showDrawer="showDiffSqlDrawer" @closeDrawer="closeDrawer" />
  <DatabaseStandardCheck v-if="showStandardCheckDrawer" :source="sourceDatasourceInfo" :checkCodes="standardCheckCodes"
    :showDrawer="showStandardCheckDrawer" @closeDrawer="closeDrawer" />
  <DatabaseGeneratorCode v-if="showGeneratorCodeDrawer" :datasource="connects" :showDrawer="showStandardCheckDrawer"
    @closeDrawer="closeDrawer" />
</template>

<style lang="scss" scoped>
.opt {
  .n-form-item {
    .n-select {
      width: 250px;
    }
  }
}
</style>
