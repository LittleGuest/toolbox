<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { MdEditor } from "md-editor-v3";
import "md-editor-v3/lib/style.css";
import {
  fetchCodeSnippetsApi,
  updateCodeSnippetApi,
  saveCodeSnippetApi,
  fetchTagsApi,
  deleteCodeSnippetApi,
} from "@/store/codeSnippet";
import { Delete, Edit, Add, Upload, Download, Search, Reset, Close, Save } from "@vicons/carbon";
import { useMessage } from "naive-ui";

const message = useMessage();

const formRef = ref(null);

const search = ref("");
const selectedSnippetId = ref(null);
const selectedTags = ref([]);
const snippets = ref([]);
const tags = ref([]);
const importInputRef = ref(null);

const currentSnippet = ref({});

const filteredSnippets = computed(() => {
  let result = snippets.value;
  if (selectedTags.value.length > 0) {
    result = result.filter((snippet) =>
      selectedTags.value.every((tag) => snippet.tags.includes(tag))
    );
  }
  if (search.value) {
    const query = search.value.toLowerCase();
    result = result.filter(
      (snippet) =>
        snippet.title.toLowerCase().includes(query) ||
        snippet.code.toLowerCase().includes(query)
    );
  }
  return result;
});

const firstLine = (code: string) =>
  String(code || "").split(/\r?\n/).find((l) => l.trim()) ?? "";
const lineCount = (code: string) => (code ? String(code).split(/\r?\n/).length : 0);

const selectSnippet = (snippet) => {
  selectedSnippetId.value = snippet.id;
  currentSnippet.value = { ...snippet };
};

const toggleTag = (tag) => {
  if (selectedTags.value.includes(tag)) {
    selectedTags.value = selectedTags.value.filter((t) => t !== tag);
  } else {
    selectedTags.value = [...selectedTags.value, tag];
  }
};

const resetSelectedTags = () => {
  selectedTags.value = [];
};

const showAddDialog = ref(false);
const addSnippets = () => {
  form.value = {
    id: null,
    language: "",
    title: "",
    tags: [],
    code: "",
  };
  showAddDialog.value = true;
};
const form = ref({
  id: null,
  language: "",
  title: "",
  tags: [],
  code: "",
});
const rules = {
  title: [
    {
      required: true,
      message: "请一句话描述",
    },
  ],
  tags: [
    {
      required: true,
      message: "请选择或输入标签",
    },
  ],
  code: [
    {
      required: true,
      message: "请输入内容",
    },
  ],
};

const saveSnippet = async () => {
  try {
    await formRef.value?.validate();

    let params = { ...form.value };
    params.tags = params.tags.join(",");

    if (form.value.id) {
      await updateCodeSnippetApi(params);
    } else {
      await saveCodeSnippetApi(params);
    }
    message.success("保存成功");
    form.value = {
      id: null,
      language: "",
      title: "",
      tags: [],
      code: "",
    };
    showAddDialog.value = false;
    await getCodeSnippets();
    await getTags();
  } catch (error) {
    console.error("表单验证失败:", error);
  }
};

const editSnippets = (snippet) => {
  showAddDialog.value = true;
  form.value = { ...snippet };
};

const handleClose = () => {
  form.value = {
    id: null,
    language: "",
    title: "",
    tags: [],
    code: "",
  };
};

const getCodeSnippets = async () => {
  const codeSnippets = await fetchCodeSnippetsApi();
  snippets.value = codeSnippets.map((cs) => {
    cs.tags = cs.tags.split(",");
    return cs;
  });
};

const getTags = async () => {
  const tagList = (await fetchTagsApi()) || [];
  const tagSet = tagList.map((tag) => tag.tags.split(",")).flat();
  tags.value = [...new Set(tagSet)];
};

const deleteSnippet = async (id) => {
  await deleteCodeSnippetApi(id);
  await getCodeSnippets();
  await getTags();
};

const normalizeTags = (value) => {
  if (Array.isArray(value)) {
    return value.filter(Boolean).join(",");
  }
  return String(value || "")
    .split(",")
    .map((tag) => tag.trim())
    .filter(Boolean)
    .join(",");
};

const importSnippets = () => {
  importInputRef.value?.click();
};

const handleImportFile = async (event) => {
  const input = event.target;
  const file = input.files?.[0];
  if (!file) {
    return;
  }

  try {
    const text = await file.text();
    const data = JSON.parse(text);
    const list = Array.isArray(data) ? data : data.snippets;
    if (!Array.isArray(list)) {
      message.error("导入文件格式不正确");
      return;
    }

    let count = 0;
    for (const item of list) {
      if (!item?.title || !item?.code) {
        continue;
      }
      await saveCodeSnippetApi({
        language: item.language || "",
        title: item.title,
        tags: normalizeTags(item.tags),
        code: item.code,
      });
      count += 1;
    }

    await getCodeSnippets();
    await getTags();
    message.success(`导入完成，共导入 ${count} 条`);
  } catch (error) {
    message.error(`导入失败: ${error}`);
  } finally {
    input.value = "";
  }
};

const exportSnippets = () => {
  const data = filteredSnippets.value.map((snippet) => ({
    language: snippet.language || "",
    title: snippet.title,
    tags: snippet.tags || [],
    code: snippet.code,
  }));
  if (data.length === 0) {
    message.warning("没有可导出的代码片段");
    return;
  }

  const blob = new Blob([JSON.stringify({ version: 1, snippets: data }, null, 2)], {
    type: "application/json;charset=utf-8",
  });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `code-snippets-${Date.now()}.json`;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
  message.success(`已导出 ${data.length} 条代码片段`);
};

onMounted(() => {
  getCodeSnippets();
  getTags();
});
</script>

<template>
  <div class="tb-page">
    <div class="tb-card code-snippet-container">
      <div class="tb-card-header">
        <span class="tb-card-header-title">代码片段</span>
        <div class="tb-card-header-actions">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button type="primary" @click="addSnippets">
                <template #icon>
                  <n-icon><Add /></n-icon>
                </template>
              </n-button>
            </template>
            新建
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="importSnippets">
                <template #icon>
                  <n-icon><Upload /></n-icon>
                </template>
              </n-button>
            </template>
            导入
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button @click="exportSnippets">
                <template #icon>
                  <n-icon><Download /></n-icon>
                </template>
              </n-button>
            </template>
            导出
          </n-tooltip>
        </div>
        <input ref="importInputRef" type="file" accept="application/json,.json" style="display: none"
          @change="handleImportFile" />
      </div>

      <div class="snippet-toolbar">
        <n-input v-model:value="search" placeholder="搜索标题或代码内容…" clearable class="snippet-search">
          <template #prefix><n-icon><Search /></n-icon></template>
        </n-input>
        <span class="snippet-count">{{ filteredSnippets.length }} 条</span>
      </div>

      <div class="main-content">
        <aside class="tag-sidebar" v-if="tags.length > 0">
          <div class="tag-sidebar-title">
            <span>标签</span>
            <n-button v-if="selectedTags.length" text size="tiny" @click="resetSelectedTags">重置</n-button>
          </div>
          <div class="tag-list">
            <div
              v-for="tag in tags"
              :key="tag"
              class="tag-item"
              :class="{ active: selectedTags.includes(tag) }"
              @click="toggleTag(tag)"
            >
              <span class="tag-hash">#</span>{{ tag }}
            </div>
          </div>
        </aside>

        <section class="snippet-content">
          <n-scrollbar class="snippet-scroll" v-if="filteredSnippets.length > 0">
            <div class="snippet-list">
              <div
                v-for="snippet in filteredSnippets"
                :key="snippet.id"
                class="snippet-item"
                :class="{ active: selectedSnippetId === snippet.id }"
                @click="selectSnippet(snippet)"
              >
                <div class="snippet-item-main">
                  <div class="snippet-item-title">
                    {{ snippet.title }}
                    <span class="snippet-item-lines">{{ lineCount(snippet.code) }} 行</span>
                  </div>
                  <div class="snippet-item-code">{{ firstLine(snippet.code) || "（空内容）" }}</div>
                  <div class="snippet-item-tags" v-if="snippet.tags && snippet.tags.length">
                    <span class="snippet-tag" v-for="t in snippet.tags" :key="t">{{ t }}</span>
                  </div>
                </div>
                <div class="snippet-item-actions" @click.stop>
                  <n-tooltip trigger="hover">
                    <template #trigger>
                      <n-button size="small" quaternary circle @click="editSnippets(snippet)">
                        <template #icon><n-icon><Edit /></n-icon></template>
                      </n-button>
                    </template>
                    编辑
                  </n-tooltip>
                  <n-popconfirm positive-text="确认" negative-text="取消" @positive-click="deleteSnippet(snippet.id)">
                    <template #trigger>
                      <n-tooltip trigger="hover">
                        <template #trigger>
                          <n-button size="small" quaternary circle>
                            <template #icon><n-icon><Delete /></n-icon></template>
                          </n-button>
                        </template>
                        删除
                      </n-tooltip>
                    </template>
                    是否确认删除？
                  </n-popconfirm>
                </div>
              </div>
            </div>
          </n-scrollbar>
          <n-empty v-else class="snippet-empty" description="暂无代码片段" />
        </section>
      </div>
    </div>
  </div>

  <n-drawer v-model:show="showAddDialog" placement="bottom" resizable :default-width="502" :default-height="'100%'"
    :height="'100%'" @update:show="handleClose">
    <n-drawer-content :title="form.id ? '编辑' : '添加'" closable>
      <n-form ref="formRef" :model="form" :rules="rules" label-placement="left" label-width="auto"
        require-mark-placement="right-hanging" style="display: flex; flex-direction: column; flex: 1;">
        <n-form-item path="title" label="一句话">
          <n-input placeholder="一句话描述" v-model:value="form.title" clearable />
        </n-form-item>
        <n-form-item path="tags" label="标签">
          <n-select v-model:value="form.tags" multiple filterable tag
            :options="tags.map(tag => ({ label: tag, value: tag }))" placeholder="输入或选择标签" />
        </n-form-item>
        <n-form-item path="code" :show-labels="false">
          <MdEditor v-model="form.code" style="height: calc(100vh - 290px);" />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="showAddDialog = false">
              <template #icon>
                <n-icon><Close /></n-icon>
              </template>
            </n-button>
          </template>
          取消
        </n-tooltip>
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button @click="saveSnippet">
              <template #icon>
                <n-icon><Save /></n-icon>
              </template>
            </n-button>
          </template>
          保存
        </n-tooltip>
      </template>
    </n-drawer-content>
  </n-drawer>

</template>

<style lang="scss" scoped>
.code-snippet-container {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 20px;
  overflow: hidden;
  height: 100%;
}

.snippet-toolbar {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;
  padding-bottom: 12px;
  border-bottom: 1px solid var(--tb-border);

  .snippet-search {
    flex: 1;
    min-width: 200px;
  }

  .snippet-count {
    flex-shrink: 0;
    font-size: 12.5px;
    color: var(--tb-text-3);
    white-space: nowrap;
  }
}

.main-content {
  flex: 1;
  min-height: 0;
  display: flex;
  overflow: hidden;

  .tag-sidebar {
    flex-shrink: 0;
    width: 180px;
    padding-right: 16px;
    margin-right: 16px;
    border-right: 1px solid var(--tb-border);
    overflow-y: auto;

    .tag-sidebar-title {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 12px;
      font-weight: 600;
      color: var(--tb-text-3);
      letter-spacing: 0.05em;
      margin-bottom: 12px;
    }

    .tag-list {
      display: flex;
      flex-direction: column;
      gap: 4px;

      .tag-item {
        display: inline-flex;
        align-items: center;
        gap: 6px;
        padding: 6px 10px;
        border-radius: var(--tb-radius-m);
        font-size: 13px;
        color: var(--tb-text-2);
        cursor: pointer;
        border: 1px solid transparent;
        transition: background-color 0.15s ease, color 0.15s ease;

        .tag-hash {
          color: var(--tb-text-3);
          font-size: 12px;
        }

        &:hover {
          background: var(--tb-bg-app);
          color: var(--tb-text);
        }

        &.active {
          background: var(--tb-primary-weak);
          color: var(--tb-primary);
          font-weight: 600;

          .tag-hash {
            color: var(--tb-primary);
          }
        }
      }
    }
  }

  .snippet-content {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;

    .snippet-scroll {
      flex: 1;
      min-height: 0;
    }

    .snippet-empty {
      flex: 1;
      display: flex;
      align-items: center;
      justify-content: center;
    }
  }
}

.snippet-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 2px;
}

.snippet-item {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 12px 14px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  background: var(--tb-bg-elevated);
  cursor: pointer;
  transition: border-color 0.15s ease, box-shadow 0.15s ease;

  &:hover {
    border-color: var(--tb-border-strong);
    box-shadow: var(--tb-shadow-card);
  }

  &.active {
    border-color: var(--tb-primary);
    background: var(--tb-primary-weak);
    box-shadow: 0 2px 12px rgba(79, 110, 247, 0.12);
  }

  .snippet-item-main {
    flex: 1;
    min-width: 0;
  }

  .snippet-item-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    font-weight: 600;
    color: var(--tb-text);

    .snippet-item-lines {
      font-size: 11.5px;
      font-weight: 400;
      color: var(--tb-text-3);
    }
  }

  .snippet-item-code {
    margin-top: 4px;
    font-family: var(--tb-font-mono);
    font-size: 12px;
    color: var(--tb-text-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .snippet-item-tags {
    margin-top: 8px;
    display: flex;
    flex-wrap: wrap;
    gap: 6px;

    .snippet-tag {
      padding: 2px 8px;
      border-radius: 999px;
      font-size: 11.5px;
      background: var(--tb-bg-app);
      border: 1px solid var(--tb-border);
      color: var(--tb-text-2);
    }
  }

  &.active .snippet-tag {
    background: var(--tb-primary-weak-hover);
  }

  .snippet-item-actions {
    display: flex;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  &:hover .snippet-item-actions,
  &.active .snippet-item-actions {
    opacity: 1;
  }
}
</style>
