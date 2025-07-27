<script setup>
import { ref, onMounted } from "vue";
import Regex from "./common/Regex.vue";
import Name from "./common/Name.vue";
import Text from "./common/Text.vue";
import Number from "./common/Number.vue";
import Email from "./common/Email.vue";
import DateTime from "./common/DateTime.vue";
import Time from "./common/Time.vue";
import Date from "./common/Date.vue";
import FileExtension from "./common/FileExtension.vue";
import FileName from "./common/FileName.vue";
import Website from "./common/Website.vue";
import Sequence from "./common/Sequence.vue";
import Uuid from "./common/Uuid.vue";
import FilePath from "./common/FilePath.vue";
import Hostname from "./common/Hostname.vue";
import Mac from "./common/Mac.vue";
import IP from "./common/IP.vue";
import Enum from "./common/Enum.vue";

// 定义属性
const props = defineProps({
  show: {
    type: Boolean,
    required: true,
  },
  data: {
    type: Object,
    required: true,
  },
});
// 定义事件
const emit = defineEmits(["update:show"]);

// 数据生成器配置
const datafakerValue = ref(null);
// 数据生成器配置选项
const datafakerOptions = [
  {
    label: "地址",
    value: "address",
  },
  {
    label: "教育",
    value: "education",
  },
  {
    label: "emoji",
    value: "emoji",
  },
  {
    label: "文件",
    value: "file",
  },
  {
    label: "互联网",
    value: "internet",
  },
  {
    label: "姓名",
    value: "name",
  },
  {
    label: "数字",
    value: "number",
  },
  {
    label: "个人",
    value: "person",
  },
  {
    label: "正则表达式",
    value: "regex",
  },
  {
    label: "序列",
    value: "sequence",
  },
  {
    label: "文本",
    value: "text",
  },
  {
    label: "UUID",
    value: "uuid",
  },

  {
    label: "日期",
    value: "date",
  },
  {
    label: "时间",
    value: "time",
  },
  {
    label: "日期时间",
    value: "datetime",
  },
  {
    label: "网址",
    value: "website",
  },
  {
    label: "主机名",
    value: "hostname",
  },
  {
    label: "文件扩展名",
    value: "file_extension",
  },
  {
    label: "文件名",
    value: "file_name",
  },
  {
    label: "文件路径",
    value: "file_path",
  },
  {
    label: "MAC地址",
    value: "mac",
  },
  {
    label: "IP地址",
    value: "ip",
  },
  {
    label: "枚举",
    value: "enum",
  },
];

// 关闭抽屉
const close = () => {
  emit("update:show", false);
};

onMounted(() => {
  console.log("props", props);
  datafakerValue.value = props.data.datafaker;
});
</script>

<template>
  <n-drawer
    v-if="show"
    :show="show"
    width="50%"
    :on-esc="close"
    :on-mask-click="close"
  >
    <n-drawer-content closable>
      <template #header>
        <n-flex>
          <span>{{ data?.columnName }}</span>
          <span>{{ data?.type }}</span>
          <span>生成器配置</span>
        </n-flex>
      </template>
      <n-form-item
        path="percentage"
        label="生成器"
        label-placement="left"
        label-width="180"
      >
        <n-select
          v-model:value="datafakerValue"
          :options="datafakerOptions"
          placeholder="请选择生成器"
          filterable
        />
      </n-form-item>
      <Date v-if="datafakerValue === 'date'" />
      <DateTime v-if="datafakerValue === 'datetime'" />
      <Time v-if="datafakerValue === 'time'" />
      <Email v-if="datafakerValue === 'email'" />
      <Enum v-if="datafakerValue === 'enum'" />
      <FileExtension v-if="datafakerValue === 'file_extension'" />
      <FileName v-if="datafakerValue === 'file_name'" />
      <FilePath v-if="datafakerValue === 'file_path'" />
      <Hostname v-if="datafakerValue === 'hostname'" />
      <IP v-if="datafakerValue === 'ip'" />
      <Mac v-if="datafakerValue === 'mac'" />
      <Name v-if="datafakerValue === 'name'" />
      <Number v-if="datafakerValue === 'number'" />
      <Regex v-if="datafakerValue === 'regex'" />
      <Sequence v-if="datafakerValue === 'sequence'" />
      <Text v-if="datafakerValue === 'text'" />
      <Time v-if="datafakerValue === 'time'" />
      <Uuid v-if="datafakerValue === 'uuid'" />
      <Website v-if="datafakerValue === 'website'" />
      <template #footer>
        <n-space>
          <n-button @click="close">取消</n-button>
          <n-button type="primary" @click="saveChanges">保存</n-button>
        </n-space>
      </template>
    </n-drawer-content>
  </n-drawer>
</template>
