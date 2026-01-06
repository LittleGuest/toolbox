<script setup>
import { ref, h } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Close, ArrowUp } from "@vicons/carbon";

const message = useMessage();

const input = ref("");
const results = ref([]);
const loading = ref(false);

// 表格列配置
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


// 乱码恢复函数
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

// 复制结果
const copyResult = (result) => {
  writeText(result.recoveredText);
  message.success("已复制到剪贴板");
};

// 清空
const clear = () => {
  input.value = "";
  results.value = [];
};

// 提取恢复后的文本（现在直接使用结构化数据）
const extractText = (result) => {
  return result.recoveredText;
};
</script>

<template>
  <!-- 输入区域 -->
  <div class="input-section">
    <n-input v-model:value="input" :rows="6" type="textarea" placeholder="请输入乱码文本，例如：锘挎槬鐪犱笉瑙夋檽锛屽澶勯椈鍟奸笩。"
      style="width: 100%" />
    <div class="hint">
      <span>说明：并非所有乱码都可以被完美恢复，乱码中的问号说明该字符已经丢失，是无法恢复的。</span>
      <div class="buttons">
        <n-button @click="recover" :loading="loading" type="primary">
          恢复
        </n-button>
        <n-button @click="clear">
          清空
        </n-button>
      </div>
    </div>
  </div>

  <!-- 结果区域 -->
  <div class="results-section">
    <n-data-table v-if="results && results.length > 0" :columns="tableColumns" :data="results" :pagination="false"
      :bordered="true" max-height="calc(100vh - 350px)">
      <template #empty>
        <n-empty description="暂无数据" />
      </template>
    </n-data-table>
  </div>
</template>

<style scoped>
.buttons {
  display: flex;
  gap: 8px;
}

.hint {
  margin-top: 8px;
  font-size: 12px;
  color: #999;
  line-height: 1.5;
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.results-section {
  margin-top: 24px;
}
</style>