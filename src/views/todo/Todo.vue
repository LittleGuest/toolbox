<script setup>
import { ref, computed, onMounted } from "vue";
import { Delete } from "@vicons/carbon";
import { useMessage } from "naive-ui";

const message = useMessage();
const newTodoText = ref("");
const todos = ref([]);
const filter = ref("all");
const showEditModal = ref(false);
const editingTodoId = ref(null);
const editingTodoText = ref("");

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
  };

  // 将新待办事项添加到数组开头
  todos.value.unshift(newTodo);
  newTodoText.value = "";
  saveTodos();
};

// 切换待办事项的完成状态
const toggleTodo = (id) => {
  const todo = todos.value.find((t) => t.id === id);
  if (todo) {
    todo.completed = !todo.completed;
    saveTodos();
  }
};

// 删除待办事项
const deleteTodo = (id) => {
  todos.value = todos.value.filter((t) => t.id !== id);
  saveTodos();
  message.success("待办事项已删除");
};

// 清除所有已完成的待办事项
const clearCompleted = () => {
  todos.value = todos.value.filter((t) => !t.completed);
  saveTodos();
  message.success("已清除所有已完成的待办事项");
};

// 编辑待办事项
const editTodo = (id) => {
  const todo = todos.value.find((t) => t.id === id);
  if (todo) {
    editingTodoId.value = id;
    editingTodoText.value = todo.text;
    showEditModal.value = true;
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
    showEditModal.value = false;
    message.success("待办事项已更新");
  }
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

// 已完成的待办事项数量
const completedCount = computed(() => {
  return todos.value.filter((t) => t.completed).length;
});

// 初始加载待办事项
onMounted(() => {
  loadTodos();
});
// 移除toggleTodo函数

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
      <n-input
        v-model:value="newTodoText"
        placeholder="输入新的待办事项..."
        clearable
        @keyup.enter="addTodo"
      />
      <n-button type="primary" @click="addTodo" :disabled="!newTodoText.trim()"
        >添加</n-button
      >
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
          <n-empty v-if="filteredTodos.length === 0">
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
          <n-list-item
            v-else
            v-for="todo in filteredTodos"
            :key="todo.id"
            class="todo-item"
          >
            <n-checkbox v-model:checked="todo.completed" />
            <span
              :class="{ completed: todo.completed }"
              @dblclick="editTodo(todo.id)"
              >{{ todo.text }}</span
            >
            <n-popconfirm
              positive-text="确认"
              negative-text="取消"
              @positive-click="deleteTodo(todo.id)"
            >
              <template #trigger>
                <n-button class="delete-button">
                  <n-icon><Delete /></n-icon>
                </n-button>
              </template>
              是否确认删除？
            </n-popconfirm>
          </n-list-item>
        </n-list>
      </n-scrollbar>
    </div>

    <!-- 编辑弹窗 -->
    <n-modal v-model:show="showEditModal" :title="'编辑待办事项'" :width="400">
      <n-input
        v-model:value="editingTodoText"
        placeholder="编辑待办事项..."
        @keyup.enter="saveEdit"
      />
      <template #footer>
        <n-button @click="showEditModal = false">取消</n-button>
        <n-button type="primary" @click="saveEdit">保存</n-button>
      </template>
    </n-modal>
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
        align-items: center;
        padding: 12px 16px;

        .n-checkbox {
          margin-right: 12px;
        }

        span {
          flex: 1;
          cursor: pointer;
          transition: all 0.2s;

          &.completed {
            text-decoration: line-through;
            color: #888;
          }
        }

        .delete-button {
          opacity: 0;
          transition: opacity 0.2s;

          &:hover {
            color: #f56c6c;
          }
        }

        &:hover .delete-button {
          opacity: 1;
        }
      }
    }
  }
}
</style>
