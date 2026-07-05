<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from "vue";
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
import ForeignKey from "./common/ForeignKey.vue";
import SimplePreview from "./common/SimplePreview.vue";

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
  referenceOptions: {
    type: Array,
    default: () => [],
  },
});
// 定义事件
const emit = defineEmits(["update:show"]);

// 数据生成器配置
const datafakerValue = ref(null);
const configComponent = ref();
const generatorGroups = [
  {
    label: "日期时间",
    children: [
      { label: "日期", value: "date" },
      { label: "日期时间", value: "datetime" },
      { label: "时间", value: "time" },
      { label: "时间戳", value: "timestamp" },
      { label: "时区名称", value: "timezone" },
    ],
  },
  {
    label: "地理位置",
    children: [
      { label: "省份和城市", value: "province_city" },
      { label: "国家或地区", value: "country_region" },
      { label: "经纬度", value: "latitude_longitude" },
      { label: "邮编", value: "zip_code" },
      { label: "地址", value: "address" },
      { label: "城市", value: "city" },
      { label: "省/州", value: "state" },
      { label: "街道地址", value: "street_address" },
      { label: "固话区号", value: "phone_area_code" },
      { label: "固定电话", value: "phone" },
    ],
  },
  {
    label: "教育",
    children: [
      { label: "学历", value: "degree" },
      { label: "小学名称", value: "primary_school" },
      { label: "小学年级", value: "primary_school_grade" },
      { label: "中学名称", value: "high_school" },
      { label: "中学年级", value: "high_school_grade" },
      { label: "班级", value: "school_class" },
      { label: "大学", value: "college" },
      { label: "专业", value: "major" },
    ],
  },
  {
    label: "金融",
    children: [
      { label: "金额", value: "money" },
      { label: "股票名称+股票代码", value: "stock" },
      { label: "日K线数据", value: "stock_kline" },
      { label: "基金名称+基金代码", value: "fund" },
      { label: "货币信息", value: "currency" },
      { label: "银行卡号", value: "bank_card" },
      { label: "付款方式", value: "payment_method" },
      { label: "信用卡类型", value: "credit_card_type" },
      { label: "信用卡卡号", value: "credit_card_number" },
      { label: "信用卡日期", value: "credit_card_date" },
    ],
  },
  {
    label: "个人",
    children: [
      { label: "姓名", value: "name" },
      { label: "性别", value: "gender" },
      { label: "手机号", value: "mobile" },
      { label: "身份证号", value: "id_card" },
      { label: "用户名", value: "username" },
      { label: "密码", value: "password" },
      { label: "QQ号", value: "qq" },
      { label: "昵称", value: "nickname" },
      { label: "民族", value: "ethnicity" },
      { label: "职位", value: "job_title" },
    ],
  },
  {
    label: "商业",
    children: [
      { label: "公司", value: "company" },
      { label: "部门", value: "department" },
      { label: "行业", value: "industry" },
    ],
  },
  {
    label: "产品",
    children: [
      { label: "产品名称", value: "product_name" },
      { label: "产品类别", value: "product_category" },
      { label: "颜色", value: "color" },
      { label: "尺寸", value: "size" },
      { label: "重量单位", value: "weight_unit" },
      { label: "条码", value: "barcode" },
      { label: "SKU", value: "sku" },
    ],
  },
  {
    label: "互联网",
    children: [
      { label: "邮箱", value: "email" },
      { label: "主机名", value: "hostname" },
      { label: "IP地址", value: "ip" },
      { label: "MAC地址", value: "mac" },
      { label: "网址", value: "website" },
      { label: "文件扩展名", value: "file_extension" },
      { label: "文件名", value: "file_name" },
      { label: "文件路径", value: "file_path" },
      { label: "App Bundle ID", value: "app_bundle_id" },
      { label: "应用名", value: "app_name" },
      { label: "应用版本", value: "app_version" },
      { label: "User-Agent", value: "user_agent" },
      { label: "端口", value: "port" },
    ],
  },
  {
    label: "其它",
    children: [
      { label: "数字", value: "number" },
      { label: "布尔值", value: "boolean" },
      { label: "JSON", value: "json" },
      { label: "汉字", value: "chinese_char" },
      { label: "成语", value: "idiom" },
      { label: "车牌号", value: "license_plate" },
      { label: "热门手机型号", value: "mobile_model" },
      { label: "统一社会信用代码", value: "unified_social_credit_code" },
      { label: "数据生成工具", value: "data_tool" },
      { label: "序列", value: "sequence" },
      { label: "枚举", value: "enum" },
      { label: "文本", value: "text" },
      { label: "图像或二进制", value: "binary" },
      { label: "外键", value: "foreign_key" },
      { label: "UUID", value: "uuid" },
      { label: "正则表达式", value: "regex" },
    ],
  },
];

const flatDatafakerOptions = generatorGroups.flatMap((group) => group.children);
const datafakerOptions = generatorGroups.map((group) => ({
  type: "group",
  label: group.label,
  key: group.label,
  children: group.children,
}));

const simplePreviewGenerators = new Set([
  "timestamp",
  "timezone",
  "province_city",
  "country_region",
  "latitude_longitude",
  "phone_area_code",
  "mobile",
  "phone",
  "id_card",
  "gender",
  "username",
  "password",
  "qq",
  "nickname",
  "ethnicity",
  "money",
  "stock",
  "stock_kline",
  "fund",
  "currency",
  "bank_card",
  "payment_method",
  "credit_card_type",
  "credit_card_number",
  "credit_card_date",
  "boolean",
  "json",
  "address",
  "city",
  "state",
  "street_address",
  "zip_code",
  "company",
  "job_title",
  "department",
  "industry",
  "degree",
  "primary_school",
  "primary_school_grade",
  "high_school",
  "high_school_grade",
  "school_class",
  "college",
  "major",
  "product_name",
  "product_category",
  "size",
  "weight_unit",
  "barcode",
  "sku",
  "app_name",
  "app_bundle_id",
  "app_version",
  "user_agent",
  "port",
  "color",
  "chinese_char",
  "idiom",
  "license_plate",
  "mobile_model",
  "unified_social_credit_code",
  "data_tool",
  "binary",
]);

const isSimplePreviewGenerator = computed(() =>
  simplePreviewGenerators.has(String(datafakerValue.value))
);

// 关闭抽屉
const close = () => {
  emit("update:show", false);
};

const handleShowUpdate = (value: boolean) => {
  emit("update:show", value);
};

const selectedOption = () =>
  flatDatafakerOptions.find((item) => item.value === datafakerValue.value);

const hydrateConfig = async () => {
  await nextTick();
  configComponent.value?.setConfig?.(props.data?.config || {});
};

const saveChanges = () => {
  const config = configComponent.value?.getConfig?.() || {};
  const option = selectedOption();
  Object.assign(props.data, {
    datafaker: datafakerValue.value,
    datafakerName: option?.label || datafakerValue.value,
    config,
  });
  close();
};

onMounted(() => {
  datafakerValue.value = props.data.datafaker;
  hydrateConfig();
});

watch(datafakerValue, hydrateConfig);
</script>

<template>
  <n-drawer
    v-if="show"
    :show="show"
    width="30%"
    @update:show="handleShowUpdate"
    :on-esc="close"
    :on-mask-click="close"
  >
    <n-drawer-content closable @close="close">
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
      <Date v-if="datafakerValue === 'date'" ref="configComponent" />
      <DateTime v-if="datafakerValue === 'datetime'" ref="configComponent" />
      <Time v-if="datafakerValue === 'time'" ref="configComponent" />
      <Email v-if="datafakerValue === 'email'" ref="configComponent" />
      <Enum v-if="datafakerValue === 'enum'" ref="configComponent" />
      <FileExtension v-if="datafakerValue === 'file_extension'" ref="configComponent" />
      <FileName v-if="datafakerValue === 'file_name'" ref="configComponent" />
      <FilePath v-if="datafakerValue === 'file_path'" ref="configComponent" />
      <ForeignKey
        v-if="datafakerValue === 'foreign_key'"
        ref="configComponent"
        :reference-options="referenceOptions"
      />
      <Hostname v-if="datafakerValue === 'hostname'" ref="configComponent" />
      <IP v-if="datafakerValue === 'ip'" ref="configComponent" />
      <Mac v-if="datafakerValue === 'mac'" ref="configComponent" />
      <Name v-if="datafakerValue === 'name'" ref="configComponent" />
      <Number v-if="datafakerValue === 'number'" ref="configComponent" />
      <Regex v-if="datafakerValue === 'regex'" ref="configComponent" />
      <Sequence v-if="datafakerValue === 'sequence'" ref="configComponent" />
      <Text v-if="datafakerValue === 'text'" ref="configComponent" />
      <Uuid v-if="datafakerValue === 'uuid'" ref="configComponent" />
      <Website v-if="datafakerValue === 'website'" ref="configComponent" />
      <SimplePreview
        v-if="isSimplePreviewGenerator"
        ref="configComponent"
        :generator="datafakerValue"
      />
      <template #footer>
        <n-space>
          <n-button @click="close">取消</n-button>
          <n-button type="primary" @click="saveChanges">保存</n-button>
        </n-space>
      </template>
    </n-drawer-content>
  </n-drawer>
</template>
