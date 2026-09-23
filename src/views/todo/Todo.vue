<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { Delete, Edit, ChevronDown, ChevronRight, Add, Save, Close, Erase } from "@vicons/carbon";
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

const loadTodos = () => {
  const savedTodos = localStorage.getItem("todos");
  if (savedTodos) {
    todos.value = JSON.parse(savedTodos);
  }
};

const saveTodos = () => {
  localStorage.setItem("todos", JSON.stringify(todos.value));
};

const addTodo = () => {
  if (!newTodoText.value.trim()) {
    message.warning("请输入待办事项内容");
    return;
  }

  const newTodo = {
    id: Date.now(),
    text: newTodoText.value.trim(),
    completed: false,
    createdAt: new Date().getTime(),
    subTodos: [],
    parentId: null
  };

  todos.value.unshift(newTodo);
  newTodoText.value = "";
  saveTodos();
};

const deleteTodo = (id) => {
  const deleteRecursive = (todoId) => {
    const subTodos = todos.value.filter(t => t.parentId === todoId);
    subTodos.forEach(subTodo => deleteRecursive(subTodo.id));

    todos.value = todos.value.filter((t) => t.id !== todoId);
  };

  deleteRecursive(id);
  saveTodos();
  message.success("待办事项已删除");
};

const clearCompleted = () => {
  const topCompletedIds = todos.value
    .filter(t => t.completed && t.parentId === null)
    .map(t => t.id);

  topCompletedIds.forEach(id => deleteTodo(id));

  saveTodos();
  message.success("已清除所有已完成的待办事项");
};

const editTodo = (id) => {
  const todo = todos.value.find((t) => t.id === id);
  if (todo) {
    editingTodoId.value = id;
    editingTodoText.value = todo.text;
  }
};

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

const cancelEdit = () => {
  editingTodoId.value = null;
  editingTodoText.value = "";
};

const toggleExpand = (id) => {
  if (expandedTodos.value.has(id)) {
    expandedTodos.value.delete(id);
  } else {
    expandedTodos.value.add(id);
  }
};

const startAddSubTodo = (id) => {
  addingSubTodoForId.value = id;
  newSubTodoText.value = "";
  expandedTodos.value.add(id);
};

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

const cancelAddSubTodo = () => {
  addingSubTodoForId.value = null;
  newSubTodoText.value = "";
};

const hasSubTodos = (id) => {
  return todos.value.some(t => t.parentId === id);
};

const getSubTodos = (id) => {
  return todos.value.filter(t => t.parentId === id);
};

const areAllSubTodosCompleted = (id) => {
  const subTodos = getSubTodos(id);
  if (subTodos.length === 0) return false;
  return subTodos.every(t => t.completed);
};

const updateTodoStatus = (todo) => {
  if (todo.parentId !== null) {
    const parentTodo = todos.value.find(t => t.id === todo.parentId);
    if (parentTodo) {
      if (areAllSubTodosCompleted(todo.parentId)) {
        parentTodo.completed = true;
      } else {
        parentTodo.completed = false;
      }
    }
  }

  if (todo.parentId === null && todo.completed) {
    const subTodos = getSubTodos(todo.id);
    subTodos.forEach(subTodo => {
      subTodo.completed = true;
    });
  }

  saveTodos();
};

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

  return result.sort((a, b) => {
    if (a.completed !== b.completed) {
      return a.completed ? 1 : -1;
    }
    return b.createdAt - a.createdAt;
  });
});

const topTodos = computed(() => {
  return filteredTodos.value.filter(t => t.parentId === null);
});

const completedCount = computed(() => {
  return todos.value.filter((t) => t.completed).length;
});

onMounted(() => {
  loadTodos();
});

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
  <div class="tb-page todo-container">
    <div class="tb-card">
      <div class="add-todo">
        <n-input v-model:value="newTodoText" placeholder="输入新的待办事项..." clearable @keyup.enter="addTodo" />
        <n-tooltip trigger="hover">
          <template #trigger>
            <n-button type="primary" @click="addTodo" :disabled="!newTodoText.trim()">
              <template #icon>
                <n-icon><Add /></n-icon>
              </template>
            </n-button>
          </template>
          添加
        </n-tooltip>
      </div>

      <div class="filter-container">
        <n-radio-group v-model:value="filter" button-style="solid">
          <n-radio-button value="all">全部</n-radio-button>
          <n-radio-button value="active">未完成</n-radio-button>
          <n-radio-button value="completed">已完成</n-radio-button>
        </n-radio-group>

        <n-tooltip trigger="hover" v-if="completedCount > 0">
          <template #trigger>
            <n-button type="text" @click="clearCompleted">
              <template #icon>
                <n-icon><Erase /></n-icon>
              </template>
            </n-button>
          </template>
          清除已完成
        </n-tooltip>
      </div>

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

          <template v-else>
            <n-list-item v-for="todo in topTodos" :key="todo.id" class="todo-item">
              <div class="todo-item-content">
                <n-tooltip v-if="hasSubTodos(todo.id)" trigger="hover">
                  <template #trigger>
                    <n-button text @click="toggleExpand(todo.id)" class="expand-btn">
                      <n-icon>
                        <ChevronDown v-if="expandedTodos.has(todo.id)" />
                        <ChevronRight v-else />
                      </n-icon>
                    </n-button>
                  </template>
                  展开/收起
                </n-tooltip>
                <div v-else class="expand-placeholder"></div>

                <n-checkbox v-model:checked="todo.completed" @update:checked="updateTodoStatus(todo)" />

                <div v-if="editingTodoId === todo.id" class="edit-mode">
                  <n-input v-model:value="editingTodoText" placeholder="编辑待办事项..." @keyup.enter="saveEdit"
                    @keyup.esc="cancelEdit" autofocus />
                  <n-space :size="8">
                    <n-tooltip trigger="hover">
                      <template #trigger>
                        <n-button type="primary" @click="saveEdit">
                          <template #icon>
                            <n-icon><Save /></n-icon>
                          </template>
                        </n-button>
                      </template>
                      保存
                    </n-tooltip>
                    <n-tooltip trigger="hover">
                      <template #trigger>
                        <n-button @click="cancelEdit">
                          <template #icon>
                            <n-icon><Close /></n-icon>
                          </template>
                        </n-button>
                      </template>
                      取消
                    </n-tooltip>
                  </n-space>
                </div>

                <div v-else class="display-mode">
                  <div class="todo-text" :class="{ completed: todo.completed }">{{ todo.text }}</div>
                  <n-space :size="8">
                    <n-tooltip trigger="hover">
                      <template #trigger>
                        <n-button class="add-sub-btn" @click="startAddSubTodo(todo.id)">
                          <template #icon>
                            <n-icon>
                              <Add />
                            </n-icon>
                          </template>
                        </n-button>
                      </template>
                      添加子任务
                    </n-tooltip>
                    <n-tooltip trigger="hover">
                      <template #trigger>
                        <n-button class="edit-button" @click="editTodo(todo.id)">
                          <template #icon>
                            <n-icon>
                              <Edit />
                            </n-icon>
                          </template>
                        </n-button>
                      </template>
                      编辑
                    </n-tooltip>
                    <n-popconfirm positive-text="确认" negative-text="取消" @positive-click="deleteTodo(todo.id)">
                      <template #trigger>
                        <n-tooltip trigger="hover">
                          <template #trigger>
                            <n-button class="delete-button" type="error">
                              <template #icon>
                                <n-icon>
                                  <Delete />
                                </n-icon>
                              </template>
                            </n-button>
                          </template>
                          删除
                        </n-tooltip>
                      </template>
                      是否确认删除？
                    </n-popconfirm>
                  </n-space>
                </div>
              </div>

              <div v-if="addingSubTodoForId === todo.id" class="add-sub-todo-container">
                <n-input v-model:value="newSubTodoText" placeholder="输入子任务内容..." @keyup.enter="addSubTodo"
                  @keyup.esc="cancelAddSubTodo" autofocus />
                <n-space :size="8">
                  <n-tooltip trigger="hover">
                    <template #trigger>
                      <n-button type="primary" @click="addSubTodo">
                        <template #icon>
                          <n-icon><Add /></n-icon>
                        </template>
                      </n-button>
                    </template>
                    添加
                  </n-tooltip>
                  <n-tooltip trigger="hover">
                    <template #trigger>
                      <n-button @click="cancelAddSubTodo">
                        <template #icon>
                          <n-icon><Close /></n-icon>
                        </template>
                      </n-button>
                    </template>
                    取消
                  </n-tooltip>
                </n-space>
              </div>

              <div v-if="expandedTodos.has(todo.id) && hasSubTodos(todo.id)" class="sub-todos-container">
                <div v-for="subTodo in getSubTodos(todo.id)" :key="subTodo.id" class="sub-todo-item">
                  <div class="sub-todo-content">
                    <div class="sub-todo-indent"></div>
                    <n-checkbox v-model:checked="subTodo.completed" @update:checked="updateTodoStatus(subTodo)" />

                    <div v-if="editingTodoId === subTodo.id" class="edit-mode">
                      <n-input v-model:value="editingTodoText" placeholder="编辑子任务..." @keyup.enter="saveEdit"
                        @keyup.esc="cancelEdit" autofocus />
                      <n-space :size="8">
                        <n-tooltip trigger="hover">
                          <template #trigger>
                            <n-button type="primary" @click="saveEdit">
                              <template #icon>
                                <n-icon><Save /></n-icon>
                              </template>
                            </n-button>
                          </template>
                          保存
                        </n-tooltip>
                        <n-tooltip trigger="hover">
                          <template #trigger>
                            <n-button @click="cancelEdit">
                              <template #icon>
                                <n-icon><Close /></n-icon>
                              </template>
                            </n-button>
                          </template>
                          取消
                        </n-tooltip>
                      </n-space>
                    </div>

                    <div v-else class="display-mode">
                      <div class="todo-text" :class="{ completed: subTodo.completed }">{{ subTodo.text }}</div>
                      <n-space :size="8">
                        <n-tooltip trigger="hover">
                          <template #trigger>
                            <n-button class="edit-button" @click="editTodo(subTodo.id)">
                              <template #icon>
                                <n-icon>
                                  <Edit />
                                </n-icon>
                              </template>
                            </n-button>
                          </template>
                          编辑
                        </n-tooltip>
                        <n-popconfirm positive-text="确认" negative-text="取消" @positive-click="deleteTodo(subTodo.id)">
                          <template #trigger>
                            <n-tooltip trigger="hover">
                              <template #trigger>
                                <n-button class="delete-button" type="error">
                                  <template #icon>
                                    <n-icon>
                                      <Delete />
                                    </n-icon>
                                  </template>
                                </n-button>
                              </template>
                              删除
                            </n-tooltip>
                          </template>
                          是否确认删除？
                        </n-popconfirm>
                      </n-space>
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
  </div>
</template>

<style lang="scss" scoped>
.todo-container {
  display: flex;
  flex-direction: column;
  gap: var(--tb-gap);

  .add-todo {
    display: flex;
    gap: 10px;
    margin-bottom: 16px;
  }

  .filter-container {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .todo-list-container {
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

            .n-space {
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

            .n-space {
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

                .n-space {
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

                .n-space {
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
