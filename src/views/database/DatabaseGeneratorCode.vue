<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, createDiscreteApi } from "naive-ui";
import { Download } from "@vicons/carbon";
import html2canvas from "html2canvas";

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi([
  "message",
  "dialog",
  "notification",
  "loadingBar",
  "modal",
]);

const props = defineProps({
  datasource: {
    type: Array,
    required: true,
  },
  showDrawer: {
    type: Object,
    required: true,
  },
});
const emits = defineEmits(["closeDrawer"]);

const form = ref({
  datasourceInfo: null,
  language: '',
  path: '',
  tableNames: '',
  ignoreTables: '',
  ignoreTablePrefix: '',
  genEntity: true,
  genMapper: true,
  genMapperXml: true,
  genService: true,
  genServiceImpl: true,
  genController: true,
  entityPackageName: '',
  mapperPackageName: '',
  mapperXmlPackageName: '',
  servicePackageName: '',
  serviceImplPackageName: '',
  controllerPackageName: '',
});
const datasourceOptions = ref([]);
const tableOptions = ref([]);

const databaseStandardCheckApi = async () => {
  return await invoke("database_standard_check", {
    source: props.source,
    checkCodes: props.checkCodes.map((c) => Number(c)),
  })
    .then((res) => {
      return res;
    })
    .catch((error) => message.error(error));
};

// const postImg = ref(null);
// const downloadImg = () => {
//   html2canvas(postImg.value, {
//     allowTaint: true,
//     taintTest: false,
//     useCORS: true,
//     scrollY: 0,
//     scrollX: 0,
//     width: postImg.value.clientWidth,
//     height: postImg.value.clientHeight,
//     scale: 2.5,
//   }).then((canvas) => {
//     // 转成图片，生成图片地址
//     let imgUrl = canvas.toDataURL("image/png");
//     const eleLink = document.createElement("a");
//     eleLink.href = imgUrl; // 转换后的图片地址
//     eleLink.download = "数据库差异报告";
//     // 触发点击
//     document.body.appendChild(eleLink);
//     eleLink.click();
//     // 然后移除
//     document.body.removeChild(eleLink);
//   });
// };

const onClose = () => {
  emits("closeDrawer");
};

onMounted(async () => {
  console.log('props', props);
  // checkReports.value = await databaseStandardCheckApi();

  datasourceOptions.value = props.datasource.map((c) => {
    return {
      label: c.database + "#" + c.name,
      value: c.database + "#" + c.name,
    };
  });
});



// 数据库选项
const databaseOptions = ref([
  { value: 'mysql', label: 'MySQL' },
  { value: 'oracle', label: 'Oracle' },
  { value: 'postgresql', label: 'PostgreSQL' },
  { value: 'sqlserver', label: 'SQL Server' }
])

// 表单状态
const formState = reactive({
  database: '',
  tableNames: '',
  selectedFiles: [],
  packageNames: {}
})

// 生成的代码
const generatedCodes = ref({})
// 当前激活的标签页
const activeTab = ref('')
// 代码展示区域引用
const postImg = ref(null)

// 格式化文件名用于显示
const formatFileName = (file) => {
  return file.split('.')[0].charAt(0).toUpperCase() + file.split('.')[0].slice(1) + `.${file.split('.')[1]}`
}

// 根据文件类型获取代码语言
const getFileLanguage = (file) => {
  if (file.endsWith('.java')) return 'java'
  if (file.endsWith('.xml')) return 'xml'
  return 'text'
}

// 监听选中文件变化，初始化包名
watch(() => formState.selectedFiles, (newFiles) => {
  newFiles.forEach(file => {
    if (!formState.packageNames[file]) {
      formState.packageNames[file] = ''
    }
  })

  // 移除未选中文件的包名
  Object.keys(formState.packageNames).forEach(file => {
    if (!newFiles.includes(file)) {
      delete formState.packageNames[file]
    }
  })
}, { deep: true })

// 处理代码生成
const handleGenerate = async () => {
  try {
    if (!formState.database) {
      message.warning('请选择数据库类型')
      return
    }
    if (!formState.tableNames) {
      message.warning('请输入表名')
      return
    }
    if (formState.selectedFiles.length === 0) {
      message.warning('请选择要生成的文件类型')
      return
    }
    if (Object.values(formState.packageNames).some(pkg => !pkg)) {
      message.warning('请为每个选中的文件输入包名')
      return
    }

    const res = await invoke('generate_code_from_db', {
      database: formState.database,
      tableNames: formState.tableNames.split(',').map(t => t.trim()).filter(t => t),
      fileTypes: formState.selectedFiles,
      packageNames: formState.packageNames
    })

    if (res.success) {
      generatedCodes.value = res.data
      if (formState.selectedFiles.length > 0) {
        activeTab.value = formState.selectedFiles[0]
      }
      message.success('代码生成成功')
    } else {
      message.error(res.message || '代码生成失败')
    }
  } catch (error) {
    console.error('代码生成出错:', error)
    message.error('代码生成出错，请检查控制台日志')
  }
}
</script>

<template>
  <n-drawer v-model:show="props.showDrawer" placement="right" resizable default-width="50%" :default-height="600"
    @update:show="onClose()">
    <n-drawer-content title="数据库代码生成" closable>
      <n-form label-placement="left" label-width="auto" require-mark-placement="right-hanging">
        <n-form @submit.prevent="handleGenerate">
          <n-form-item label="数据源" path="datasourceInfo">
            <n-select placeholder="请选择数据源" v-model:value="form.datasourceInfo" :options="datasourceOptions" />
          </n-form-item>

          <n-form-item label="输入表名" path="tableNames">
            <n-select placeholder="请选择要生成的表名，不选择则生成所有表" v-model:value="form.tableNames" :options="tableOptions" />
          </n-form-item>

          <!-- 文件类型选择 -->
          <n-form-item label="选择生成文件" path="selectedFiles">
            <n-checkbox-group v-model:value="formState.selectedFiles">
              <n-checkbox value="entity.java">Entity.java</n-checkbox>
              <n-checkbox value="mapp.java">Mapp.java</n-checkbox>
              <n-checkbox value="mapper.xml">Mapper.xml</n-checkbox>
              <n-checkbox value="service.java">Service.java</n-checkbox>
              <n-checkbox value="serviceimpl.java">ServiceImpl.java</n-checkbox>
              <n-checkbox value="controller.java">Controller.java</n-checkbox>
            </n-checkbox-group>
          </n-form-item>

          <!-- 每个文件的包名输入 -->
          <template v-for="file in formState.selectedFiles" :key="file">
            <n-form-item :label="`${formatFileName(file)} 包名`" :path="`packageNames.${file}`">
              <n-input v-model:value="formState.packageNames[file]" placeholder="请输入包名（例如：com.example.demo）" />
            </n-form-item>
          </template>

        </n-form>

        <!-- 代码展示区域 -->
        <div ref="postImg" class="code-display" v-if="Object.keys(generatedCodes).length">
          <n-tabs v-model:active-key="activeTab">
            <n-tab-pane v-for="(code, file) in generatedCodes" :key="file" :name="file" :title="formatFileName(file)">
              <!-- <n-code-block :language="getFileLanguage(file)" :code="code" show-line-number copyable /> -->
            </n-tab-pane>
          </n-tabs>
        </div>
      </n-form>

      <template #footer>
        <n-button-group>
          <n-button @click="downloadImg()">
            <template #icon>
              <n-icon>
                <Download />
              </n-icon>
            </template>
          </n-button>
          <n-button native-type="submit">
            生成代码
          </n-button>
        </n-button-group>
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
</style>
