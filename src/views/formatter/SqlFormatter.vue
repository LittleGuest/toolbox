<script setup>
import { ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import CodeMirror from "vue-codemirror6";
import { format } from "sql-formatter";
import { Copy, Paste } from "@vicons/carbon";

const dialect = ref("mysql");
const indent = ref(2);
const upper = ref("upper");
const sql = ref("");
const indentOptions = [
  {
    label: 2,
    value: 2
  },
  {
    label: 4,
    value: 4,
  }
];
// bigquery,db2,db2i,hive,mariadb,mysql,n1ql,plsql,postgresql,redshift,singlestoredb,
// snowflake,spark,sql,sqlite,tidb,transactsql,trino,tsql
const dialectOptions = [
  {
    label: 'MySQl',
    value: 'mysql'
  },
  {
    label: 'SQLite',
    value: 'sqlite',
  },
  {
    label: 'PostgreSQL',
    value: 'postgresql',
  }
];

const formatSql = () => {
  sql.value = format(sql.value, {
    language: 'sql',
    tabWidth: indent.value,
    keywordCase: upper.value,
  });
};

const paste = async () => {
  const clip = await readText();
  sql.value = clip;
};

const copy = async () => {
  await writeText(sql.value);
};
</script>

<template>
  <n-form label-placement="left">
    <!-- <n-form-item label="方言"> -->
    <!--   <n-select placeholder="请选择方言" :options="dialectOptions" v-model:value="dialect" /> -->
    <!-- </n-form-item> -->
    <n-form-item label="缩进">
      <n-select placeholder="请选择缩进字符" :options="indentOptions" v-model:value="indent" />
    </n-form-item>
    <n-form-item label="关键字大写">
      <n-switch v-model:value="upper" checked-value="upper" unchecked-value="lower" />
    </n-form-item>
    <n-form-item label="">
      <n-button-group>
        <n-button @click="paste">
          <template #icon>
            <n-icon>
              <Paste />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="copy">
          <template #icon>
            <n-icon>
              <Copy />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="formatSql">格式化</n-button>
      </n-button-group>
    </n-form-item>

    <code-mirror basic v-model="sql" />
  </n-form>
</template>
