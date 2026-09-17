<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage, NButton, NIcon, NSpace, NTooltip } from "naive-ui";
import { Copy, Paste, Erase, ArrowsHorizontal } from "@vicons/carbon";

const message = useMessage();

// 当前时间
const current = ref("");
let updateTimer = null;

// 配置
const mode = ref<"ts_to_dt" | "dt_to_ts">("ts_to_dt"); // 时间戳→时间 / 时间→时间戳
const unit = ref<"s" | "ms">("s"); // 秒 / 毫秒（ts_to_dt 时有效，dt_to_ts 时输出两种）
const tzOffset = ref(28800); // 时区偏移（秒），默认 UTC+8

// 预设时区
const tzOptions = [
  { label: "UTC-12", value: -43200 },
  { label: "UTC-11", value: -39600 },
  { label: "UTC-10", value: -36000 },
  { label: "UTC-9", value: -32400 },
  { label: "UTC-8", value: -28800 },
  { label: "UTC-7", value: -25200 },
  { label: "UTC-6", value: -21600 },
  { label: "UTC-5", value: -18000 },
  { label: "UTC-4", value: -14400 },
  { label: "UTC-3", value: -10800 },
  { label: "UTC-2", value: -7200 },
  { label: "UTC-1", value: -3600 },
  { label: "UTC", value: 0 },
  { label: "UTC+1", value: 3600 },
  { label: "UTC+2", value: 7200 },
  { label: "UTC+3", value: 10800 },
  { label: "UTC+4", value: 14400 },
  { label: "UTC+5", value: 18000 },
  { label: "UTC+6", value: 21600 },
  { label: "UTC+7", value: 25200 },
  { label: "UTC+8", value: 28800 },
  { label: "UTC+9", value: 32400 },
  { label: "UTC+10", value: 36000 },
  { label: "UTC+11", value: 39600 },
  { label: "UTC+12", value: 43200 },
];

const input = ref("");
const convertResult = ref<
  { input: string; secondTs: number | null; milliTs: number | null; datetime: string }[] | null
>(null);
const loading = ref(false);

const updateCurrentTime = () => {
  current.value = Math.floor(Date.now() / 1000).toString();
};

const doConvert = async () => {
  const trimmed = input.value
    .split("\n")
    .filter((l) => l.trim())
    .map((l) => l.trim());
  if (trimmed.length === 0) {
    message.warning("请输入时间戳或时间");
    return;
  }
  loading.value = true;
  try {
    const res: {
      input: string;
      second_ts: number | null;
      milli_ts: number | null;
      datetime: string;
    }[] = await invoke("timestamp_convert", {
      mode: mode.value,
      unit: unit.value,
      tzOffsetSecs: tzOffset.value,
      values: trimmed,
    });
    convertResult.value = res.map((r) => ({
      input: r.input,
      secondTs: r.second_ts,
      milliTs: r.milli_ts,
      datetime: r.datetime,
    }));
  } catch (error) {
    message.error(String(error));
  } finally {
    loading.value = false;
  }
};

const fillInput = () => {
  const nums = [1700000000, "2023-01-01 00:00:00"];
  input.value = nums.join("\n");
};

const clear = () => {
  input.value = "";
  convertResult.value = null;
};

const actionBtns = (row) => {
  const mk = (val) =>
    h(
      NTooltip,
      null,
      {
        trigger: () =>
          h(
            NButton,
            { size: "small", quaternary: true, onClick: () => copy(val) },
            { icon: () => h(NIcon, null, { default: () => h(Copy) }) }
          ),
        default: () => val === null || val === undefined ? "值无效" : "复制",
      }
    );
  return h(NSpace, { size: 4 }, { default: () => [mk(row.secondTs), mk(row.milliTs), mk(row.datetime)] });
};

const columns = computed(() => [
  { title: "输入", key: "input", width: 200 },
  { title: "秒时间戳", key: "secondTs", width: 130, render: (row) => row.secondTs },
  { title: "毫秒时间戳", key: "milliTs", width: 150, render: (row) => row.milliTs },
  { title: "时间", key: "datetime" },
  { title: "操作", key: "actions", width: 140, render: (row) => actionBtns(row) },
]);

const paste = async () => {
  input.value = await readText();
};

const copy = (value) => {
  if (value === null || value === undefined || value === "") return;
  writeText(String(value));
  message.success("复制成功");
};

onMounted(() => {
  updateCurrentTime();
  updateTimer = setInterval(updateCurrentTime, 1000);
});

onUnmounted(() => {
  if (updateTimer) clearInterval(updateTimer);
});
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <div class="tb-config-row">
        <div class="tb-config-item">
          <span class="tb-config-label">当前秒级</span>
          <n-input :value="current" readonly style="width: 130px" class="tb-mono">
            <template #suffix>s</template>
          </n-input>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button quaternary circle size="small" @click="copy(current)">
                <template #icon><n-icon><Copy /></n-icon></template>
              </n-button>
            </template>
            复制当前时间戳
          </n-tooltip>
        </div>

        <div class="tb-config-item">
          <span class="tb-config-label">方向</span>
          <n-radio-group v-model:value="mode" size="small">
            <n-radio-button value="ts_to_dt">时间戳 → 时间</n-radio-button>
            <n-radio-button value="dt_to_ts">时间 → 时间戳</n-radio-button>
          </n-radio-group>
        </div>
        <div class="tb-config-item" v-if="mode === 'ts_to_dt'">
          <span class="tb-config-label">单位</span>
          <n-radio-group v-model:value="unit" size="small">
            <n-radio-button value="s">秒</n-radio-button>
            <n-radio-button value="ms">毫秒</n-radio-button>
          </n-radio-group>
        </div>
        <div class="tb-config-item">
          <span class="tb-config-label">时区</span>
          <n-select
            v-model:value="tzOffset"
            :options="tzOptions"
            size="small"
            style="width: 140px"
          />
        </div>
      </div>

      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输入</span>
        <n-input
          v-model:value="input"
          type="textarea"
          :autosize="{ minRows: 4, maxRows: 12 }"
          :placeholder="mode === 'ts_to_dt' ? '每行一个时间戳，可混合秒/毫秒，如：\n1700000000\n1700000000123' : '每行一个时间，如：\n2023-11-14 22:13:20\n2023-11-14 22:13:20.123'"
        />
      </div>

      <div class="tb-action-row">
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" :loading="loading" @click="doConvert">
              <template #icon><n-icon><ArrowsHorizontal /></n-icon></template>
            </n-button>
          </template>
          转换
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="fillInput">
              <template #icon><n-icon><Erase /></n-icon></template>
            </n-button>
          </template>
          填入示例
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="paste">
              <template #icon><n-icon><Paste /></n-icon></template>
            </n-button>
          </template>
          粘贴
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="clear">
              <template #icon><n-icon><Erase /></n-icon></template>
            </n-button>
          </template>
          清除
        </n-tooltip>
      </div>

      <template v-if="convertResult">
        <span class="tb-editor-label">结果</span>
        <n-data-table
          :columns="columns"
          :data="convertResult"
          :single-line="false"
          :bordered="true"
          size="small"
        />
      </template>
    </section>
  </div>
</template>