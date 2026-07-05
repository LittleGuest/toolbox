<script setup lang="ts">
import { computed, onMounted, ref } from "vue";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { Copy, Paste, Close, TrashCan } from "@vicons/carbon";

interface ClipboardHistory {
  id: number;
  content: string;
  createdAt: string;
}

const STORAGE_KEY = "clipboard-history";
const message = useMessage();

const input = ref("");
const keyword = ref("");
const clipboardHistory = ref<ClipboardHistory[]>([]);

const filteredHistory = computed(() => {
  const value = keyword.value.trim().toLowerCase();
  if (!value) {
    return clipboardHistory.value;
  }
  return clipboardHistory.value.filter((item) =>
    item.content.toLowerCase().includes(value),
  );
});

const saveHistory = () => {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(clipboardHistory.value));
};

const loadHistory = () => {
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return;
  }
  try {
    const data = JSON.parse(raw);
    if (Array.isArray(data)) {
      clipboardHistory.value = data.filter(
        (item) => typeof item?.content === "string",
      );
    }
  } catch {
    localStorage.removeItem(STORAGE_KEY);
  }
};

const addHistory = (content: string) => {
  const value = content.trim();
  if (!value) {
    message.warning("剪贴板内容为空");
    return;
  }
  clipboardHistory.value = clipboardHistory.value.filter(
    (item) => item.content !== value,
  );
  clipboardHistory.value.unshift({
    id: Date.now(),
    content: value,
    createdAt: new Date().toLocaleString(),
  });
  clipboardHistory.value = clipboardHistory.value.slice(0, 100);
  saveHistory();
};

const readCurrentClipboard = async () => {
  input.value = await readText();
  addHistory(input.value);
  message.success("已读取当前剪贴板");
};

const copy = async (value: string) => {
  await writeText(value);
  message.success("已复制到剪贴板");
};

const copyInput = async () => {
  if (!input.value.trim()) {
    message.warning("请输入要复制的内容");
    return;
  }
  await writeText(input.value);
  addHistory(input.value);
  message.success("已复制并加入历史");
};

const removeHistory = (id: number) => {
  clipboardHistory.value = clipboardHistory.value.filter((item) => item.id !== id);
  saveHistory();
};

const clearHistory = () => {
  clipboardHistory.value = [];
  saveHistory();
};

onMounted(loadHistory);
</script>

<template>
  <n-card title="剪贴板管理">
    <n-space vertical size="large">
      <n-input
        v-model:value="input"
        type="textarea"
        :autosize="{ minRows: 5, maxRows: 12 }"
        placeholder="输入内容后复制，或读取当前剪贴板加入历史"
      />
      <n-space>
        <n-button type="primary" @click="copyInput">
          <template #icon>
            <n-icon><Copy /></n-icon>
          </template>
          复制并记录
        </n-button>
        <n-button @click="readCurrentClipboard">
          <template #icon>
            <n-icon><Paste /></n-icon>
          </template>
          读取当前剪贴板
        </n-button>
        <n-popconfirm @positive-click="clearHistory">
          <template #trigger>
            <n-button secondary type="error">
              <template #icon>
                <n-icon><TrashCan /></n-icon>
              </template>
              清空历史
            </n-button>
          </template>
          确认清空剪贴板历史？
        </n-popconfirm>
      </n-space>
      <n-input v-model:value="keyword" clearable placeholder="搜索历史内容" />
      <n-empty v-if="filteredHistory.length === 0" description="暂无剪贴板历史" />
      <n-list v-else bordered>
        <n-list-item v-for="history in filteredHistory" :key="history.id">
          <template #suffix>
            <n-space>
              <n-button quaternary circle @click="copy(history.content)">
                <template #icon>
                  <n-icon><Copy /></n-icon>
                </template>
              </n-button>
              <n-button quaternary circle type="error" @click="removeHistory(history.id)">
                <template #icon>
                  <n-icon><Close /></n-icon>
                </template>
              </n-button>
            </n-space>
          </template>
          <n-thing :title="history.createdAt">
            <n-code :code="history.content" word-wrap />
          </n-thing>
        </n-list-item>
      </n-list>
    </n-space>
  </n-card>
</template>
