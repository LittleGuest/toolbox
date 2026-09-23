<script setup lang="ts">
import { ref, h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Close, ArrowUp, Play, Erase } from "@vicons/carbon";

const message = useMessage();

const input = ref("");
const results = ref([]);
const loading = ref(false);

const tableColumns = [
  {
    title: "序号",
    key: "index",
    width: 80,
    render: (row, index) => index + 1
  },
  {
    title: "源编码（假设）",
    key: "sourceCharset",
    width: 120
  },
  {
    title: "目标编码（假设）",
    key: "targetCharset",
    width: 120
  },
  {
    title: "恢复后的文本",
    key: "recoveredText"
  },
  {
    title: "得分",
    key: "score",
    width: 100,
    render: (row) => {
      const score = row.score || 0;
      const percentage = (score * 100).toFixed(2);
      const color = score >= 0.9 ? '#52c41a' : score >= 0.7 ? '#faad14' : '#ff4d4f';
      return h(
        'span',
        {
          style: {
            color: color,
            fontWeight: 'bold'
          }
        },
        `${percentage}`
      );
    }
  },
  {
    title: "操作",
    key: "actions",
    width: 80,
    render: (row) => {
      return h(
        "button",
        {
          onClick: () => copyResult(row),
          style: {
            padding: "4px 8px",
            backgroundColor: "#1890ff",
            color: "white",
            border: "none",
            borderRadius: "4px",
            cursor: "pointer",
            fontSize: "12px"
          }
        },
        "复制"
      );
    }
  }
];

const recover = async () => {
  if (!input.value.trim()) {
    message.warning("请输入乱码文本");
    return;
  }

  loading.value = true;
  try {
    const response = await invoke("recover_garbled_code", {
      input: input.value
    });

    results.value = response || [];

    if (results.value.length === 0) {
      message.info("未找到可恢复的文本");
    }
  } catch (error) {
    console.error("错误:", error);
    message.error(JSON.stringify(error));
  } finally {
    loading.value = false;
  }
};

const copyResult = (result) => {
  writeText(result.recoveredText);
  message.success("已复制到剪贴板");
};

const clear = () => {
  input.value = "";
  results.value = [];
};

const extractText = (result) => {
  return result.recoveredText;
};
</script>

<template>
  <div>
      <div class="tb-editor tb-mono">
        <span class="tb-editor-label">输入</span>
        <n-input v-model:value="input" :rows="6" type="textarea"
          placeholder="请输入乱码文本，例如：锘挎槬鐪犱笉瑙夋檽锛屽澶勯椈鍟奸笩。" />
        <div class="tb-toolbar">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" :loading="loading" @click="recover">
                <template #icon><n-icon><Play /></n-icon></template>
              </n-button>
            </template>
            恢复
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="clear">
                <template #icon><n-icon><Erase /></n-icon></template>
              </n-button>
            </template>
            清空
          </n-tooltip>
        </div>
        <p class="tb-hint">说明：并非所有乱码都可以被完美恢复，乱码中的问号说明该字符已经丢失，是无法恢复的。</p>
      </div>

      <div class="tb-editor">
        <span class="tb-editor-label">结果</span>
        <n-data-table v-if="results && results.length > 0" :columns="tableColumns" :data="results"
          :pagination="false" :bordered="true" max-height="calc(100vh - 350px)">
          <template #empty>
            <n-empty description="暂无数据" />
          </template>
        </n-data-table>
      </div>
  </div>
</template>
