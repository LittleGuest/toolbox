<script setup>
import { ref, computed, onMounted } from "vue";
import { MdEditor } from "md-editor-v3";
import "md-editor-v3/lib/style.css";
import {
  fetchCodeSnippetsApi,
  updateCodeSnippetApi,
  saveCodeSnippetApi,
  fetchTagsApi,
  deleteCodeSnippetApi,
} from "@/store/codeSnippet.js";
import { Delete, Edit } from "@vicons/carbon";
import { useMessage } from "naive-ui";

// 引入消息提示
const message = useMessage();

// 表单引用
const formRef = ref(null);

// 搜索词
const search = ref("");
// 选中的标签
const selectedSnippetId = ref(null);
// 选中的标签
const selectedTags = ref([]);
// 代码片段列表
const snippets = ref([]);
// 标签列表
const tags = ref([]);

// 当前片段
const currentSnippet = ref({});

// 过滤后的代码片段
const filteredSnippets = computed(() => {
  let result = snippets.value;
  // 根据标签过滤
  if (selectedTags.value.length > 0) {
    result = result.filter((snippet) =>
      selectedTags.value.every((tag) => snippet.tags.includes(tag))
    );
  }
  // 根据搜索词过滤
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

// 选择代码片段
const selectSnippet = (snippet) => {
  selectedSnippetId.value = snippet.id;
  currentSnippet.value = { ...snippet };
};

// 切换标签选择状态
const toggleTag = (tag) => {
  if (selectedTags.value.includes(tag)) {
    selectedTags.value = selectedTags.value.filter((t) => t !== tag);
  } else {
    selectedTags.value = [...selectedTags.value, tag];
  }
};

// 添加重置标签方法
const resetSelectedTags = () => {
  selectedTags.value = [];
};

// 添加弹窗
const showAddDialog = ref(false);
// 添加片段
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

// 提交表单
const saveSnippet = async () => {
  try {
    // 验证表单
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
    // 验证失败，不执行保存操作
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

// 获取代码片段
const getCodeSnippets = async () => {
  const codeSnippets = await fetchCodeSnippetsApi();
  snippets.value = codeSnippets.map((cs) => {
    cs.tags = cs.tags.split(",");
    return cs;
  });
};

// 获取标签
const getTags = async () => {
  const tagList = (await fetchTagsApi()) || [];
  const tagSet = tagList.map((tag) => tag.tags.split(",")).flat();
  tags.value = [...new Set(tagSet)];
};

// 删除代码片段
const deleteSnippet = async (id) => {
  await deleteCodeSnippetApi(id);
  await getCodeSnippets();
  await getTags();
};

// 导入功能
const importSnippets = () => {
  // 实现导入逻辑
  alert("导入功能待实现");
};

// 导出功能
const exportSnippets = () => {
  // 实现导出逻辑
  alert("导出功能待实现");
};

onMounted(() => {
  getCodeSnippets();
  getTags();
});
</script>

<template>
  <div class="code-snippet-container">
    <!-- 顶部工具栏 -->
    <div class="toolbar">
      <n-button type="primary" @click="addSnippets">新建</n-button>
      <n-button @click="importSnippets">导入</n-button>
      <n-button @click="exportSnippets">导出</n-button>
      <n-input v-model:value="search" placeholder="搜索片段..." />
      <n-button>搜索</n-button>
    </div>

    <!-- 主内容区域 -->
    <div class="main-content">
      <!-- 左侧标签栏 -->
      <div class="tag-sidebar" v-if="tags.length > 0">
        <n-scrollbar>
          <n-card title="标签" size="small">
            <template #header-extra v-if="selectedTags.length > 0">
              <n-button type="text" size="small" @click="resetSelectedTags">重置选择</n-button>
            </template>
            <div class="tag-list">
              <n-tag v-for="tag in tags" :key="tag" :class="{ active: selectedTags.includes(tag) }"
                :bordered="!selectedTags.includes(tag)" @click="toggleTag(tag)" class="tag-item">
                {{ tag }}
              </n-tag>
            </div>
          </n-card>
        </n-scrollbar>
      </div>

      <!-- 中间代码片段列表 -->
      <div class="snippet-content">
        <div class="snippet-list-container">
          <n-scrollbar>
            <n-list class="snippet-list" v-if="filteredSnippets.length > 0">
              <n-list-item v-for="snippet in filteredSnippets" :key="snippet.id"
                :class="{ active: selectedSnippetId === snippet.id }">
                <template #suffix>
                  <n-button-group>
                    <n-button size="small" @click="editSnippets(snippet)">
                      <template #icon>
                        <n-icon>
                          <Edit />
                        </n-icon>
                      </template>
                    </n-button>
                    <n-popconfirm positive-text="确认" negative-text="取消" @positive-click="deleteSnippet(snippet.id)">
                      <template #trigger>
                        <n-button size="small">
                          <template #icon>
                            <n-icon>
                              <Delete />
                            </n-icon>
                          </template>
                        </n-button>
                      </template>
                      是否确认删除？
                    </n-popconfirm>
                  </n-button-group>
                </template>
                <n-thing :title="snippet.title" />
              </n-list-item>
            </n-list>
            <n-empty v-else description="无数据" style="margin-top: 50px;" />
          </n-scrollbar>
        </div>
      </div>
    </div>
  </div>

  <!-- 添加/编辑代码片段弹窗 -->
  <n-drawer v-model:show="showAddDialog" placement="bottom" resizable :default-width="502" :default-height="'100%'"
    :height="'100%'" @update:show="handleClose">
    <n-drawer-content :title="form.id ? '编辑' : '添加'" closable>
      <n-form ref="formRef" :model="form" :rules="rules" label-placement="left" label-width="auto"
        require-mark-placement="right-hanging" style="display: flex; flex-direction: column; flex: 1;">
        <n-form-item path="title" label="一句话">
          <n-input placeholder="一句话描述" v-model:value="form.title" clearable />
        </n-form-item>
        <!-- <n-form-item path="language" label="语言" v-if="false">
          <n-input placeholder="请输入语言" v-model:value="form.language" clearable />
        </n-form-item> -->
        <n-form-item path="tags" label="标签">
          <n-select v-model:value="form.tags" multiple filterable tag
            :options="tags.map(tag => ({ label: tag, value: tag }))" placeholder="输入或选择标签" />
        </n-form-item>
        <n-form-item path="code" :show-labels="false">
          <MdEditor v-model="form.code" style="height: calc(100vh - 290px);" />
        </n-form-item>
      </n-form>
      <template #footer>
        <n-button @click="showAddDialog = false">取消</n-button>
        <n-button @click="saveSnippet">保存</n-button>
      </template>
    </n-drawer-content>
  </n-drawer>


</template>

<style lang="scss" scoped>
.code-snippet-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;

  .toolbar {
    display: flex;
    align-items: center;
    padding: 10px;

    .search-container {
      display: inline-flex;
      gap: 10px;
      width: 50%;
    }

    .action-buttons {
      display: flex;
      gap: 10px;
    }
  }

  .main-content {
    flex: 1;
    display: flex;
    overflow: hidden;

    .tag-sidebar {
      width: 200px;
      height: 100%;
      border-right: 1px solid #eee;
      padding: 10px;

      .tag-list {
        display: flex;
        flex-wrap: wrap;
        gap: 8px;

        .tag-item {
          cursor: pointer;

          &.active {
            background-color: #f0f7ff;
            color: #1890ff;
          }
        }
      }
    }

    .snippet-content {
      flex: 1;
      display: flex;
      justify-content: center;
      align-items: flex-start;
      padding: 20px;
      overflow-y: auto;

      .snippet-list-container {
        width: 100%;
        background-color: #fff;
        border-radius: 8px;
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        padding: 16px;

        .snippet-list {
          .n-list-item {
            cursor: pointer;
            margin-bottom: 8px;
            border-radius: 6px;
            transition: all 0.3s ease;

            &:hover {
              background-color: #f9f9f9;
            }

            &.active {
              background-color: #f0f7ff;
              border-left: 3px solid #1890ff;
            }
          }
        }
      }
    }
  }

  .code-preview {
    margin-top: 16px;
    max-height: 400px;
    overflow-y: auto;
  }
}
</style>
