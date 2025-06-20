<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { NButton, NButtonGroup, createDiscreteApi } from "naive-ui";
import { Download } from "@vicons/carbon";
import html2canvas from 'html2canvas';

const { message, notification, dialog, loadingBar, modal } = createDiscreteApi(["message", "dialog", "notification", "loadingBar", "modal"]);

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

const isDoCloseAll = ref(1);
const increList = ref([]);
const missList = ref([]);
const changeList = ref([]);

const databaseDiffReportApi = async () => {
  return await invoke("database_diff_report", {
    source: props.source,
    target: props.target,
  }).then((res) => {
    return res;
  }).catch((error) => message.error(error));
};


const columnColumns = ref([
  {
    title: "字段名称",
    key: "name",
  },
  {
    title: "类型",
    key: "columnType",
    render(row) {
      if (!row.fieldTypeChange) {
        return;
      }
      return getContent(row, 'fieldTypeChange');
    }
  },
  {
    title: "长度",
    key: "length",
    render(row) {
      if (!row.lengthChange) {
        return;
      }
      return getContent(row, 'lengthChange');
    }
  },
  {
    title: "小数位",
    key: "scale",
    render(row) {
      if (!row.scaleChange) {
        return;
      }
      return getContent(row, 'scaleChange');
    }
  },
  {
    title: "为空",
    key: "null",
    render(row) {
      if (!row.nullChange) {
        return;
      }
      return getContent(row, 'nullChange');
    }
  },
  {
    title: "无符号",
    key: "nonUnsigned",
    render(row) {
      if (!row.unsignedChange) {
        return;
      }
      return getContent(row, 'unsignedChange');
    }
  },
  {
    title: "默认值",
    key: "default",
    render(row) {
      if (!row.defaultChange) {
        return;
      }
      return getContent(row, 'defaultChange');
    }
  },
  {
    title: "注释",
    key: "comment",
    render(row) {
      if (!row.commentChange) {
        return;
      }
      return getContent(row, 'commentChange');
    }
  },
]);

const indexColumns = ref([
  {
    title: "索引名称",
    key: "name",
  },
  {
    title: "索引类型",
    key: "indexType",
    render(row) {
      if (!row.indexTypeChange) {
        return;
      }
      return getContent(row, 'indexTypeChange');
    }
  },
  {
    title: "能否重复",
    key: "nonUnique",
    render(row) {
      if (!row.nonUniqueChange) {
        return;
      }
      return getContent(row, 'nonUniqueChange');
    }
  },
  {
    title: "列名",
    key: "columnName",
    render(row) {
      if (!row.columnNameChange) {
        return;
      }
      return getContent(row, 'columnNameChange');
    }
  },
  {
    title: "注释",
    key: "indexComment",
    render(row) {
      if (!row.indexCommentChange) {
        return;
      }
      return getContent(row, 'indexCommentChange');
    }
  },
]);

const getContent = (row, type) => {
  let separate = " 变更为 ";
  if (type == 'fieldTypeChange') {
    return row.sourceFieldType + separate + row.targetFieldType;
  } else if (type == 'lengthChange') {
    return row.sourceLength + separate + row.targetLength;
  } else if (type == 'defaultChange') {
    return row.sourceDefault + separate + row.targetDefault;
  } else if (type == 'scaleChange') {
    return row.sourceScale + separate + row.targetScale;
  } else if (type == 'nullChange') {
    return row.sourceNull + separate + row.targetNull;
  } else if (type == 'commentChange') {
    return row.sourceComment + separate + row.targetComment;
  } else if (type == 'unsignedChange') {
    return row.sourceUnsigned + separate + row.targetUnsigned;
  } else if (type == 'nonUniqueChange') {
    return row.sourceNonUnique + separate + row.targetNonUnique;
  } else if (type == 'columnNameChange') {
    return row.sourceColumnName + separate + row.targetColumnName;
  } else if (type == 'indexTypeChange') {
    return row.sourceIndexType + separate + row.targetIndexType;
  } else if (type == 'indexCommentChange') {
    return row.sourceIndexComment + separate + row.targetIndexComment;
  }
  return "";
};

const closeColumn = (tableItem) => {
  if (tableItem) {
    tableItem.close = !tableItem.close;
  } else {
    for (let i = 0; i < changeList.value.length; i++) {
      changeList.value[i].close = isDoCloseAll.value;
    }
    isDoCloseAll.value = !isDoCloseAll.value
  }
};

const postImg = ref(null);

const downloadImg = () => {
  html2canvas(postImg.value, {
    allowTaint: true,
    taintTest: false,
    useCORS: true,
    scrollY: 0,
    scrollX: 0,
    width: postImg.value.clientWidth,
    height: postImg.value.clientHeight,
    scale: 2.5,
  }).then(canvas => {
    // 转成图片，生成图片地址
    let imgUrl = canvas.toDataURL('image/png');
    const eleLink = document.createElement('a');
    eleLink.href = imgUrl; // 转换后的图片地址
    eleLink.download = "数据库差异报告";
    // 触发点击
    document.body.appendChild(eleLink);
    eleLink.click();
    // 然后移除
    document.body.removeChild(eleLink);
  });
};

const onClose = () => {
  emits('closeDrawer');
};

onMounted(async () => {
  const res = await databaseDiffReportApi();
  changeList.value = res.changes;
  increList.value = res.incres;
  missList.value = res.misses;
});
</script>

<template>
  <n-drawer v-model:show="props.showDrawer" placement="right" resizable default-width="50%" :default-height="600"
    @update:show="onClose()">
    <n-drawer-content title="数据库差异报告" closable>
      <div class="container" id="postImg" ref="postImg" style="margin-top: 10px;">
        <div class="block-table extra" v-if="increList && increList.length > 0">
          <h3>增加的表</h3>
          <n-flex>
            <n-tag type="info" v-for="(item, index) in increList" :key="index">{{ item }}</n-tag>
          </n-flex>
        </div>
        <div class="block-table miss" v-if="missList && missList.length > 0">
          <h3>删除的表</h3>
          <n-flex>
            <n-tag type="error" v-for="(item, index) in missList" :key="index">{{ item }}</n-tag>
          </n-flex>
        </div>
        <div class="block-table change" v-if="changeList && changeList.length > 0">
          <h3>变动的表</h3>
          <div v-for="item in changeList">
            <div class="block-head-column">
              <div><n-tag type="warning">{{ item.tableName }}</n-tag></div>
              <div>
                <span class="m-btn-text" @click="closeColumn(item)">
                  <i :class="item.close ? 'n-icon-caret-bottom' : 'n-icon-caret-top'"></i>
                  {{ item.close ? '展开' : '收起' }}
                </span>
              </div>
            </div>
            <div class="block-column" v-show="!item.close">
              <div class="block-table miss" v-if="item.commentChange == 1">
                <h4>表注释变化</h4>
                <n-flex>
                  <n-tag type="info">基准库：{{ item.sourceComment }}</n-tag>
                  <n-tag type="info">变动库：{{ item.targetComment }}</n-tag>
                </n-flex>
              </div>
              <div class="block-table extra" v-if="item.increColumns && item.increColumns.length > 0">
                <h4>增加的字段</h4>
                <n-flex>
                  <n-tag v-for="(item1, index) in item.increColumns" :key="index" type="warning" size="medium">
                    {{ item1 }}
                  </n-tag>
                </n-flex>
              </div>
              <div class="block-table miss" v-if="item.missColumns && item.missColumns.length > 0">
                <h4>缺失的字段</h4>
                <n-flex>
                  <n-tag v-for="(item1, index) in item.missColumns" :key="index" type="warning" size="medium">
                    {{ item1 }}
                  </n-tag>
                </n-flex>
              </div>
              <div class="block-table" v-if="item.columns && item.columns.length > 0">
                <h4>变动的字段</h4>
                <n-data-table :columns="columnColumns" :data="item.columns" :pagination="false" :bordered="false" />
              </div>

              <div class="block-table extra" v-if="item.increIndexs && item.increIndexs.length > 0">
                <h4>增加的索引</h4>
                <n-flex>
                  <n-tag v-for="(item1, index) in item.increIndexs" :key="index" type="success" size="medium">
                    {{ item1 }}
                  </n-tag>
                </n-flex>
              </div>
              <div class="block-table miss" v-if="item.missIndexs && item.missIndexs.length > 0">
                <h4>缺失的索引</h4>
                <n-flex>
                  <n-tag v-for="(item1, index) in item.missIndexs" :key="index" type="success" size="medium">
                    {{ item1 }}
                  </n-tag>
                </n-flex>
              </div>
              <div class="block-table" v-if="item.indexs && item.indexs.length > 0">
                <h4>变动的索引</h4>
                <n-data-table :columns="indexColumns" :data="item.indexs" :pagination="false" :bordered="false" />
              </div>
            </div>
          </div>
        </div>
      </div>

      <template #footer>
        <n-button-group>
          <n-button @click="closeColumn()">
            展开/收起
          </n-button>
          <n-button @click="downloadImg()">
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

<style lang="scss" scoped>
.block-table,
.block-column {
  position: relative;
  width: 100%;
  min-height: 60px;
  border: 1px solid #dcdee2;
  border-radius: 5px;
  box-sizing: border-box;
  margin: 40px 0px;
  padding: 20px;
}

.block-column {
  border: 1px solid #409EFF;
  padding: 0px 20px 20px 20px;
  margin: 0px 0px 40px 0px;
}

.block-column>.block-table {
  box-sizing: border-box;
  margin: 20px 0px;
  border: 1px solid #e8eaec;
}

.block-head-column {
  border-bottom: 1px dashed #e8eaec;
  display: flex;
  width: 100%;
  box-sizing: border-box;
  margin: 20px 0px;
}

.block-head-column>div {
  display: flex;
  align-items: center;
  justify-content: flex-start;
}

.block-head-column>div:first-child {
  flex: 1;
}

.block-head-column>div:last-child {
  width: 0;
  flex: 0 50px;
}

h3,
h4 {
  display: inline-block;
  margin: 0 !important;
  padding: 0px 10px !important;
  position: absolute;
  top: -15px;
  left: 10px;
  background-color: #FFFFFF;
  box-sizing: border-box;
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.h3-right {
  position: absolute;
  top: -10px;
  right: 20px;
}

.m-btn-text {
  color: #409EFF;
  cursor: pointer;
}

h4 {
  top: -10px;
}

.miss .el-tag,
.extra .el-tag {
  margin-right: 10px;
  margin-bottom: 10px;
}

.block-table label {
  display: inline-block;
  padding: 5px 10px;
  color: white;
  border-radius: 3px;
  margin-right: 10px;
  margin-bottom: 10px;
  box-sizing: border-box;
  max-width: 100%;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.extra label {
  background: #19be6b;
}

.miss label {
  background: #ff9900;
}

.change label {
  background: #409EFF;
  margin-right: 0px;
  margin-bottom: 0px;
}

.block-column .extra label {
  color: #19be6b;
  background: none;
  border: 1px solid #19be6b;
  margin-right: 10px;
  margin-bottom: 10px;
}

.block-column .miss label {
  color: #ff9900;
  background: none;
  border: 1px solid #ff9900;
  margin-right: 10px;
  margin-bottom: 10px;
}
</style>
