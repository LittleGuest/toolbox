<script setup>
import { ref, onMounted } from "vue";
import Regex from "./common/Regex.vue";
import Name from "./common/Name.vue";
import Text from "./common/Text.vue";

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
];

onMounted(() => {
  console.log(props);
});
</script>

<template>
  <n-drawer v-if="show" :show="show" width="50%">
    <n-drawer-content closable>
      <template #header>
        {{ data?.columnName }}-{{ data?.columnType }} 生成器配置
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

      <!-- <Regex /> -->
      <!-- <Name /> -->
      <Text />

      <template #footer>
        <n-space>
          <n-button @click="show = false">取消</n-button>
          <n-button type="primary" @click="saveChanges">保存</n-button>
        </n-space>
      </template>
    </n-drawer-content>
  </n-drawer>
</template>
