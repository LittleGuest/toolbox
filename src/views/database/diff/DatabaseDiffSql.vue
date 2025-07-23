<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, useMessage } from "naive-ui";
import { Download } from "@vicons/carbon";
import CodeMirror from "vue-codemirror6";

const message = useMessage();

const props = defineProps({
  source: {
    type: Object,
    required: true,
  },
  target: {
    type: Object,
    required: true,
  },
  showDrawer: {
    type: Object,
    required: true,
  },
});
const emits = defineEmits(['closeDrawer']);

const sql = ref();
const sqls = ref([]);

const databaseDiffSqlApi = async () => {
  return await invoke("database_diff_sql", {
    source: props.source,
    target: props.target,
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};


const code = ref(null);
const download = () => {
  const blob = new Blob([sql.value], { type: "text/plain;charset=utf-8" });
  const objectURL = URL.createObjectURL(blob);
  const aTag = document.createElement('a');
  aTag.href = objectURL;
  aTag.download = "数据库差异SQL.sql";
  aTag.click();
  URL.revokeObjectURL(objectURL);
};

const onClose = () => {
  emits('closeDrawer');
};

onMounted(async () => {
  sqls.value = await databaseDiffSqlApi();
  sql.value = sqls.value.join("\n\n");
  // sql.value = format(sqls.value.join("\n"), {
  //   language: 'sql',
  //   tabWidth: 4,
  //   keywordCase: "upper",
  // });
});
</script>

<template>
  <n-drawer v-model:show="props.showDrawer" placement="right" resizable default-width="50%" :default-height="600"
    @update:show="onClose()">
    <n-drawer-content title="数据库差异SQL" closable>
      <div id="code" ref="code">
        <code-mirror basic v-model="sql" />
      </div>

      <template #footer>
        <n-button-group>
          <n-button @click="download()">
            <template #icon>
              <n-icon>
                <Download />
              </n-icon>
            </template>
          </n-button>
        </n-button-group>
      </template>
    </n-drawer-content>
  </n-drawer>
</template>

<style lang="scss" scoped></style>
