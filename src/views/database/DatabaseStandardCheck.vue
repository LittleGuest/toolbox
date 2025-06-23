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
  source: {
    type: Object,
    required: true,
  },
  checkCodes: {
    type: Array,
    required: true,
  },
  showDrawer: {
    type: Object,
    required: true,
  },
});
const emits = defineEmits(["closeDrawer"]);
const checkReports = ref();
const checkSpellingTypeCode = 12;
const checkPluralizeTypeCode = 31;

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
  }).then((canvas) => {
    // 转成图片，生成图片地址
    let imgUrl = canvas.toDataURL("image/png");
    const eleLink = document.createElement("a");
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
  emits("closeDrawer");
};

onMounted(async () => {
  console.log('props', props.checkCodes);
  checkReports.value = await databaseStandardCheckApi();
});
</script>

<template>
  <n-drawer v-model:show="props.showDrawer" placement="right" resizable default-width="50%" :default-height="600"
    @update:show="onClose()">
    <n-drawer-content title="数据库规范检查" closable>
      <div class="container" id="postImg" ref="postImg" style="margin-top: 10px">
        <div class="block" v-for="item in checkReports">
          <h3>
            <n-tag>{{ item.name }}</n-tag>
          </h3>
          <div class="block-content">
            <p class="m-p" v-for="(item1, index1) in item.suggests">
              <span class="m-span">{{ index1 + 1 }}.</span class="m-span">
              <span>{{ item1.desc }}</span>
              <n-button v-if="item1.code == checkSpellingTypeCode || item1.code == checkPluralizeTypeCode"
                style="margin-left: 10px;" type="success" @click="showIgnoreConfirm(item1.code, item1.originWord)">
                <n-tooltip trigger="hover">
                  <template #trigger>
                    点击忽略
                  </template>
                  点击可忽略检查此单词，可在页面右边按钮中管理单词
                </n-tooltip>
              </n-button>
            </p>
          </div>
          <div class="block-child" v-for="item2 in item.children">
            <h3>
              <n-tag size="medium">{{ item2.name }}</n-tag>
            </h3>
            <div class="block-content">
              <p class="m-p" v-for="(item21, index21) in item2.suggests">
                <span class="m-span">{{ index21 + 1 }}.</span>
                <span>{{ item21.desc }}</span>
                <n-button v-if="
                  item21.code == checkSpellingTypeCode ||
                  item21.code == checkPluralizeTypeCode
                " style="margin-left: 10px" type="success" size="mini" plain
                  @click="showIgnoreConfirm(item21.code, item21.originWord)">
                  <n-tooltip trigger="hover">
                    <template #trigger>
                      点击忽略
                    </template>
                    点击可忽略检查此单词，可在页面右边按钮中管理单词
                  </n-tooltip>
                </n-button>
              </p>
            </div>
          </div>
        </div>
      </div>
      <template #footer>
        <n-button-group>
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
