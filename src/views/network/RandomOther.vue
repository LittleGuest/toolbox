<script setup lang="ts">
import { ref } from "vue";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Play, Copy, Close } from "@vicons/carbon";

const message = useMessage();

const pad2 = (n: number) => String(n).padStart(2, "0");

const ipType = ref<"v4" | "v6">("v4");
const ipCount = ref(10);
const ipOut = ref("");

const genIp = () => {
  const lines: string[] = [];
  for (let i = 0; i < ipCount.value; i++) {
    if (ipType.value === "v4") {
      const parts: string[] = [];
      for (let j = 0; j < 4; j++) parts.push(String(Math.floor(Math.random() * 256)));
      lines.push(parts.join("."));
    } else {
      const groups: string[] = [];
      for (let j = 0; j < 8; j++) groups.push(Math.floor(Math.random() * 0x10000).toString(16));
      lines.push(groups.join(":"));
    }
  }
  ipOut.value = lines.join("\n");
};

const macFormat = ref("colon");
const MAC_FORMAT_OPTIONS = [
  { label: "冒号分隔 (AA:BB:CC:DD:EE:FF)", value: "colon" },
  { label: "短横分隔 (AA-BB-CC-DD-EE-FF)", value: "dash" },
  { label: "无分隔", value: "none" },
];
const macUnicast = ref(false);
const macCount = ref(10);
const macOut = ref("");

const randomHexByte = () =>
  Math.floor(Math.random() * 256).toString(16).padStart(2, "0").toUpperCase();

const genMac = () => {
  const lines: string[] = [];
  for (let i = 0; i < macCount.value; i++) {
    const bytes: string[] = [];
    for (let j = 0; j < 6; j++) {
      if (j === 1 && macUnicast.value) {
        const hi = ["0", "2", "4", "6", "8", "A", "C", "E"][Math.floor(Math.random() * 8)];
        const lo = Math.floor(Math.random() * 16).toString(16).toUpperCase();
        bytes.push(hi + lo);
      } else {
        bytes.push(randomHexByte());
      }
    }
    if (macFormat.value === "colon") lines.push(bytes.join(":"));
    else if (macFormat.value === "dash") lines.push(bytes.join("-"));
    else lines.push(bytes.join(""));
  }
  macOut.value = lines.join("\n");
};

const timeFormat = ref<"24" | "12">("24");
const timeCount = ref(10);
const timeOut = ref("");

const genTime = () => {
  const lines: string[] = [];
  for (let i = 0; i < timeCount.value; i++) {
    const h = Math.floor(Math.random() * 24);
    const m = Math.floor(Math.random() * 60);
    const s = Math.floor(Math.random() * 60);
    if (timeFormat.value === "24") {
      lines.push(`${pad2(h)}:${pad2(m)}:${pad2(s)}`);
    } else {
      const h12 = h % 12 || 12;
      const ampm = h < 12 ? "AM" : "PM";
      lines.push(`${pad2(h12)}:${pad2(m)}:${pad2(s)} ${ampm}`);
    }
  }
  timeOut.value = lines.join("\n");
};

const dateStart = ref<number | null>(null);
const dateEnd = ref<number | null>(null);
const dateCount = ref(10);
const dateOut = ref("");

const formatDate = (ts: number) => {
  const d = new Date(ts);
  return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}`;
};

const genDate = () => {
  if (dateStart.value == null || dateEnd.value == null) {
    message.warning("请选择开始和结束日期");
    return;
  }
  let lo = dateStart.value;
  let hi = dateEnd.value;
  if (lo > hi) [lo, hi] = [hi, lo];
  const lines: string[] = [];
  for (let i = 0; i < dateCount.value; i++) {
    const ts = lo + Math.floor(Math.random() * (hi - lo + 1));
    lines.push(formatDate(ts));
  }
  dateOut.value = lines.join("\n");
};

const outputs = {
  ip: ipOut,
  mac: macOut,
  time: timeOut,
  date: dateOut,
};

const copyOut = (key: keyof typeof outputs) => {
  const v = outputs[key].value;
  if (!v) {
    message.warning("暂无内容可复制");
    return;
  }
  writeText(v);
  message.success("复制成功");
};

const clearOut = (key: keyof typeof outputs) => {
  outputs[key].value = "";
};
</script>

<template>
  <div class="tb-page">
    <section class="tb-card">
      <n-tabs type="line" animated>
        <n-tab-pane name="ip" tab="IP 地址">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <n-radio-group v-model:value="ipType">
                <n-radio-button value="v4">IPv4</n-radio-button>
                <n-radio-button value="v6">IPv6</n-radio-button>
              </n-radio-group>
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="ipCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genIp">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="ipOut"
              type="textarea"
              :rows="10"
              placeholder="随机 IP 地址（每行一个）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('ip')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('ip')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="mac" tab="MAC 地址">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">格式</span>
              <n-select v-model:value="macFormat" :options="MAC_FORMAT_OPTIONS" style="width: 260px" />
            </div>
            <div class="tb-config-item">
              <n-checkbox v-model:checked="macUnicast">仅单播（第二段首字符为偶数）</n-checkbox>
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="macCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genMac">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="macOut"
              type="textarea"
              :rows="10"
              placeholder="随机 MAC 地址（每行一个）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('mac')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('mac')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="time" tab="时间">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <n-radio-group v-model:value="timeFormat">
                <n-radio-button value="24">24 小时制 HH:MM:SS</n-radio-button>
                <n-radio-button value="12">12 小时制 hh:mm:ss AM|PM</n-radio-button>
              </n-radio-group>
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="timeCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genTime">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="timeOut"
              type="textarea"
              :rows="10"
              placeholder="随机时间（每行一个）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('time')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('time')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>

        <n-tab-pane name="date" tab="日期">
          <div class="tb-config-row">
            <div class="tb-config-item">
              <span class="tb-config-label">开始日期</span>
              <n-date-picker v-model:value="dateStart" type="date" style="width: 160px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">结束日期</span>
              <n-date-picker v-model:value="dateEnd" type="date" style="width: 160px" />
            </div>
            <div class="tb-config-item">
              <span class="tb-config-label">数量</span>
              <n-input-number v-model:value="dateCount" :min="1" :max="10000" style="width: 120px" />
            </div>
            <div class="tb-config-item">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button type="primary" @click="genDate">
                    <template #icon>
                      <n-icon><Play /></n-icon>
                    </template>
                  </n-button>
                </template>
                生成
              </n-tooltip>
            </div>
          </div>
          <div class="tb-editor tb-mono">
            <span class="tb-editor-label">输出</span>
            <n-input
              v-model:value="dateOut"
              type="textarea"
              :rows="10"
              placeholder="区间内随机日期 YYYY-MM-DD（每行一个）"
            />
            <div class="tb-toolbar" style="margin-top: 8px">
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="copyOut('date')">
                    <template #icon><n-icon><Copy /></n-icon></template>
                  </n-button>
                </template>
                复制
              </n-tooltip>
              <n-tooltip trigger="hover">
                <template #trigger>
                  <n-button size="small" quaternary @click="clearOut('date')">
                    <template #icon><n-icon><Close /></n-icon></template>
                  </n-button>
                </template>
                清除
              </n-tooltip>
            </div>
          </div>
        </n-tab-pane>
      </n-tabs>
    </section>
  </div>
</template>
