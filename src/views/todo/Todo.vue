<script setup>
import { ref, computed, onMounted } from "vue";
import { Delete, Edit, ChevronDown, ChevronRight, Add } from "@vicons/carbon";
import { useMessage } from "naive-ui";

const message = useMessage();
const newTodoText = ref("");
const todos = ref([]);
const filter = ref("all");
const editingTodoId = ref(null); // 当前正在编辑的待办事项ID
const editingTodoText = ref(""); // 编辑中的文本
const expandedTodos = ref(new Set()); // 存储展开的待办事项ID
const addingSubTodoForId = ref(null); // 正在添加子任务的父任务ID
const newSubTodoText = ref(""); // 新子任务的文本

// 从本地存储加载待办事项
const loadTodos = () => {
  const savedTodos = localStorage.getItem("todos");
  if (savedTodos) {
    todos.value = JSON.parse(savedTodos);
  }
};

// 保存待办事项到本地存储
const saveTodos = () => {
  localStorage.setItem("todos", JSON.stringify(todos.value));
};

// 添加新的待办事项
const addTodo = () => {
  if (!newTodoText.value.trim()) {
    message.warning("请输入待办事项内容");
    return;
  }

  const newTodo = {
    id: Date.now(),
    text: newTodoText.value.trim(),
    completed: false,
    // 添加创建时间戳
    createdAt: new Date().getTime(),
    // 添加子任务数组
    subTodos: [],
    // 添加父任务ID，null表示顶级任务
    parentId: null
  };

  // 将新待办事项添加到数组开头
  todos.value.unshift(newTodo);
  newTodoText.value = "";
  saveTodos();
};

// 删除待办事项
const deleteTodo = (id) => {
  // 删除任务及其所有子任务
  const deleteRecursive = (todoId) => {
    // 先删除所有子任务
    const subTodos = todos.value.filter(t => t.parentId === todoId);
    subTodos.forEach(subTodo => deleteRecursive(subTodo.id));

    // 然后删除任务本身
    todos.value = todos.value.filter((t) => t.id !== todoId);
  };

  deleteRecursive(id);
  saveTodos();
  message.success("待办事项已删除");
};

// 清除所有已完成的待办事项
const clearCompleted = () => {
  // 只清除顶级任务，子任务会随着父任务一起被清除
  const topCompletedIds = todos.value
    .filter(t => t.completed && t.parentId === null)
    .map(t => t.id);

  topCompletedIds.forEach(id => deleteTodo(id));

  saveTodos();
  message.success("已清除所有已完成的待办事项");
};

// 编辑待办事项
const editTodo = (id) => {
  const todo = todos.value.find((t) => t.id === id);
  if (todo) {
    editingTodoId.value = id;
    editingTodoText.value = todo.text;
  }
};

// 保存编辑后的待办事项
const saveEdit = () => {
  if (!editingTodoText.value.trim()) {
    message.warning("待办事项内容不能为空");
    return;
  }

  const todo = todos.value.find((t) => t.id === editingTodoId.value);
  if (todo) {
    todo.text = editingTodoText.value.trim();
    saveTodos();
    editingTodoId.value = null;
    editingTodoText.value = "";
    message.success("待办事项已更新");
  }
};

// 取消编辑
const cancelEdit = () => {
  editingTodoId.value = null;
  editingTodoText.value = "";
};

// 切换待办事项的展开/折叠状态
const toggleExpand = (id) => {
  if (expandedTodos.value.has(id)) {
    expandedTodos.value.delete(id);
  } else {
    expandedTodos.value.add(id);
  }
};

// 开始添加子任务
const startAddSubTodo = (id) => {
  addingSubTodoForId.value = id;
  newSubTodoText.value = "";
  // 自动展开父任务
  expandedTodos.value.add(id);
};

// 添加子任务
const addSubTodo = () => {
  if (!newSubTodoText.value.trim()) {
    message.warning("请输入子任务内容");
    return;
  }

  const newTodo = {
    id: Date.now(),
    text: newSubTodoText.value.trim(),
    completed: false,
    createdAt: new Date().getTime(),
    parentId: addingSubTodoForId.value,
    subTodos: [] // 子任务不能再有子任务
  };

  todos.value.unshift(newTodo);
  newSubTodoText.value = "";
  addingSubTodoForId.value = null;
  saveTodos();
  message.success("子任务已添加");
};

// 取消添加子任务
const cancelAddSubTodo = () => {
  addingSubTodoForId.value = null;
  newSubTodoText.value = "";
};

// 检查任务是否有子任务
const hasSubTodos = (id) => {
  return todos.value.some(t => t.parentId === id);
};

// 获取任务的子任务
const getSubTodos = (id) => {
  return todos.value.filter(t => t.parentId === id);
};

// 检查所有子任务是否都已完成
const areAllSubTodosCompleted = (id) => {
  const subTodos = getSubTodos(id);
  if (subTodos.length === 0) return false;
  return subTodos.every(t => t.completed);
};

// 当任务状态改变时，检查父任务状态
const updateTodoStatus = (todo) => {
  // 如果是子任务，检查父任务是否需要更新状态
  if (todo.parentId !== null) {
    const parentTodo = todos.value.find(t => t.id === todo.parentId);
    if (parentTodo) {
      // 如果所有子任务都完成，则标记父任务为完成
      if (areAllSubTodosCompleted(todo.parentId)) {
        parentTodo.completed = true;
      } else {
        // 否则标记父任务为未完成
        parentTodo.completed = false;
      }
    }
  }

  // 如果是父任务，且被标记为完成，则同时完成所有子任务
  if (todo.parentId === null && todo.completed) {
    const subTodos = getSubTodos(todo.id);
    subTodos.forEach(subTodo => {
      subTodo.completed = true;
    });
  }

  saveTodos();
};

// 筛选待办事项
const filteredTodos = computed(() => {
  let result;
  switch (filter.value) {
    case "active":
      result = todos.value.filter((t) => !t.completed);
      break;
    case "completed":
      result = todos.value.filter((t) => t.completed);
      break;
    default:
      result = todos.value;
      break;
  }

  // 排序：未完成的在前，已完成的在后；同状态下，最新添加的在前
  return result.sort((a, b) => {
    if (a.completed !== b.completed) {
      return a.completed ? 1 : -1;
    }
    // 按创建时间倒序排列
    return b.createdAt - a.createdAt;
  });
});

// 获取顶级任务（没有父任务的）
const topTodos = computed(() => {
  return filteredTodos.value.filter(t => t.parentId === null);
});

// 已完成的待办事项数量
const completedCount = computed(() => {
  return todos.value.filter((t) => t.completed).length;
});

// 初始加载待办事项
onMounted(() => {
  loadTodos();
});

// 添加watch监听todos变化并保存
import { watch } from "vue";

watch(
  () => todos.value,
  (newTodos) => {
    saveTodos();
  },
  { deep: true }
);
</script>

<template>
  <div class="todo-container">
    <!-- 添加待办事项 -->
    <div class="add-todo">
      <n-input v-model:value="newTodoText" placeholder="输入新的待办事项..." clearable @keyup.enter="addTodo" />
      <n-button type="primary" @click="addTodo" :disabled="!newTodoText.trim()">添加</n-button>
    </div>

    <!-- 筛选器 -->
    <div class="filter-container">
      <n-radio-group v-model:value="filter" button-style="solid">
        <n-radio-button value="all">全部</n-radio-button>
        <n-radio-button value="active">未完成</n-radio-button>
        <n-radio-button value="completed">已完成</n-radio-button>
      </n-radio-group>

      <n-button type="text" @click="clearCompleted" v-if="completedCount > 0">
        清除已完成 ({{ completedCount }})
      </n-button>
    </div>

    <!-- 待办事项列表 -->
    <div class="todo-list-container">
      <n-scrollbar>
        <n-list class="todo-list">
          <n-empty v-if="topTodos.length === 0">
            <template #description>
              {{
                filter === "all"
                  ? "暂无待办事项"
                  : filter === "active"
                    ? "暂无未完成的待办事项"
                    : "暂无已完成的待办事项"
              }}
            </template>
          </n-empty>

          <!-- 顶级任务列表 -->
          <template v-else>
            <n-list-item v-for="todo in topTodos" :key="todo.id" class="todo-item">
              <div class="todo-item-content">
                <!-- 展开/折叠按钮 -->
                <n-button v-if="hasSubTodos(todo.id)" text @click="toggleExpand(todo.id)" class="expand-btn">
                  <n-icon>
                    <ChevronDown v-if="expandedTodos.has(todo.id)" />
                    <ChevronRight v-else />
                  </n-icon>
                </n-button>
                <div v-else class="expand-placeholder"></div>

                <n-checkbox v-model:checked="todo.completed" @update:checked="updateTodoStatus(todo)" />

                <!-- 编辑模式 -->
                <div v-if="editingTodoId === todo.id" class="edit-mode">
                  <n-input v-model:value="editingTodoText" placeholder="编辑待办事项..." @keyup.enter="saveEdit"
                    @keyup.esc="cancelEdit" autofocus />
                  <n-button-group size="small">
                    <n-button type="primary" @click="saveEdit">保存</n-button>
                    <n-button @click="cancelEdit">取消</n-button>
                  </n-button-group>
                </div>

                <!-- 显示模式 -->
                <div v-else class="display-mode">
                  <div class="todo-text" :class="{ completed: todo.completed }">{{ todo.text }}</div>
                  <n-button-group size="small">
                    <n-button class="add-sub-btn" @click="startAddSubTodo(todo.id)">
                      <template #icon>
                        <n-icon>
                          <Add />
                        </n-icon>
                      </template>
                    </n-button>
                    <n-button class="edit-button" @click="editTodo(todo.id)">
                      <template #icon>
                        <n-icon>
                          <Edit />
                        </n-icon>
                      </template>
                    </n-button>
                    <n-popconfirm positive-text="确认" negative-text="取消" @positive-click="deleteTodo(todo.id)">
                      <template #trigger>
                        <n-button class="delete-button" type="error">
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
                </div>
              </div>

              <!-- 添加子任务的输入框 -->
              <div v-if="addingSubTodoForId === todo.id" class="add-sub-todo-container">
                <n-input v-model:value="newSubTodoText" placeholder="输入子任务内容..." @keyup.enter="addSubTodo"
                  @keyup.esc="cancelAddSubTodo" autofocus />
                <n-button-group size="small">
                  <n-button type="primary" @click="addSubTodo">添加</n-button>
                  <n-button @click="cancelAddSubTodo">取消</n-button>
                </n-button-group>
              </div>

              <!-- 子任务列表 -->
              <div v-if="expandedTodos.has(todo.id) && hasSubTodos(todo.id)" class="sub-todos-container">
                <div v-for="subTodo in getSubTodos(todo.id)" :key="subTodo.id" class="sub-todo-item">
                  <div class="sub-todo-content">
                    <div class="sub-todo-indent"></div>
                    <n-checkbox v-model:checked="subTodo.completed" @update:checked="updateTodoStatus(subTodo)" />

                    <!-- 子任务编辑模式 -->
                    <div v-if="editingTodoId === subTodo.id" class="edit-mode">
                      <n-input v-model:value="editingTodoText" placeholder="编辑子任务..." @keyup.enter="saveEdit"
                        @keyup.esc="cancelEdit" autofocus />
                      <n-button-group size="small">
                        <n-button type="primary" @click="saveEdit">保存</n-button>
                        <n-button @click="cancelEdit">取消</n-button>
                      </n-button-group>
                    </div>

                    <!-- 子任务显示模式 -->
                    <div v-else class="display-mode">
                      <div class="todo-text" :class="{ completed: subTodo.completed }">{{ subTodo.text }}</div>
                      <n-button-group size="small">
                        <n-button class="edit-button" @click="editTodo(subTodo.id)">
                          <template #icon>
                            <n-icon>
                              <Edit />
                            </n-icon>
                          </template>
                        </n-button>
                        <n-popconfirm positive-text="确认" negative-text="取消" @positive-click="deleteTodo(subTodo.id)">
                          <template #trigger>
                            <n-button class="delete-button" type="error">
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
                    </div>
                  </div>
                </div>
              </div>
            </n-list-item>
          </template>
        </n-list>
      </n-scrollbar>
    </div>
  </div>
</template>

<style lang="scss" scoped>
.todo-container {
  height: 100%;
  overflow: hidden;

  .add-todo {
    display: flex;
    gap: 10px;
  }

  .filter-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 20px;
  }

  .todo-list-container {
    height: calc(100% - 120px);

    .todo-list {
      .todo-item {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        width: 100%;
        padding: 8px 0;


        .todo-item-content {
          display: flex;
          align-items: center;
          width: 100%;

          .expand-btn {
            margin-right: 8px;
            width: 24px;
            height: 24px;
            display: flex;
            align-items: center;
            justify-content: center;
          }

          .expand-placeholder {
            width: 24px;
            height: 24px;
            margin-right: 8px;
          }

          .n-checkbox {
            margin-right: 12px;
          }

          .edit-mode,
          .display-mode {
            display: flex;
            align-items: center;
            width: 100%;
            margin-left: 12px;
          }

          .edit-mode {
            .n-input {
              flex: 1;
              margin-right: 10px;
            }

            .n-button-group {
              margin-left: auto;
              flex-shrink: 0;
            }
          }

          .display-mode {
            .todo-text {
              flex: 1;
              cursor: pointer;
              transition: all 0.2s;
              width: calc(100vh - 90px);
              word-wrap: break-word;
              overflow-wrap: break-word;
              white-space: pre-wrap;

              &.completed {
                text-decoration: line-through;
                color: #888;
              }
            }

            .n-button-group {
              margin-left: auto;
              flex-shrink: 0;
            }

            .add-sub-btn {
              margin-right: 8px;
            }
          }
        }

        .add-sub-todo-container {
          display: flex;
          align-items: center;
          margin-top: 10px;
          margin-left: 60px;
          width: calc(100% - 60px);

          .n-input {
            flex: 1;
            margin-right: 10px;
          }
        }

        .sub-todos-container {
          width: 100%;
          margin-top: 8px;

          .sub-todo-item {
            display: flex;
            align-items: center;
            width: 100%;
            padding: 6px 0;

            .sub-todo-content {
              display: flex;
              align-items: center;
              width: calc(100vh - 60px);
              margin-left: 60px;

              .sub-todo-indent {
                width: 20px;
                height: 20px;
                border-left: 2px dashed #ccc;
                margin-right: 10px;
              }

              .n-checkbox {
                margin-right: 12px;
              }

              .edit-mode,
              .display-mode {
                display: flex;
                align-items: center;
                width: 100%;
                margin-left: 12px;
                min-width: 0;
              }

              .edit-mode {
                .n-input {
                  flex: 1;
                  margin-right: 10px;
                }

                .n-button-group {
                  margin-left: auto;
                  flex-shrink: 0;
                }
              }

              .display-mode {
                .todo-text {
                  flex: 1;
                  cursor: pointer;
                  transition: all 0.2s;
                  width: calc(100vh - 100px);
                  word-wrap: break-word;
                  overflow-wrap: break-word;
                  white-space: pre-wrap;
                  min-width: 0;

                  &.completed {
                    text-decoration: line-through;
                    color: #888;
                  }
                }

                .n-button-group {
                  margin-left: auto;
                  flex-shrink: 0;
                }
              }
            }

            .delete-button,
            .edit-button {
              opacity: 0;
              transition: opacity 0.2s;
            }

            &:hover .delete-button,
            &:hover .edit-button {
              opacity: 1;
            }
          }
        }

        .delete-button,
        .edit-button,
        .add-sub-btn {
          opacity: 0;
          transition: opacity 0.2s;
        }

        &:hover .delete-button,
        &:hover .edit-button,
        &:hover .add-sub-btn {
          opacity: 1;
        }
      }
    }
  }
}
</style>
