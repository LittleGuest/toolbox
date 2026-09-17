<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, computed } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import {
  Add,
  ArrowDown,
  ArrowUp,
  Close,
  CloudUpload,
  DocumentPdf,
  Image,
} from "@vicons/carbon";

interface Props {
  files: string[];
  title: string;
  /** 支持的扩展名清单（不含点），用于拖拽过滤与选择器 */
  extensions: string[];
  emptyHint: string;
  isImage?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  isImage: false,
});

const emit = defineEmits<{
  (e: "update", val: string[]): void;
}>();

const dragging = ref(false);

const filters = computed(() => [
  { name: "支持的文件", extensions: props.extensions },
  { name: "所有文件", extensions: ["*"] },
]);

// 文件同名去重 + 过滤扩展名（拖拽进来的才需要过滤，选择器已过滤）
const accept = (p: string) => {
  const ext = p.split(".").pop()?.toLowerCase() || "";
  return props.extensions.includes(ext);
};

const fileNames = computed(() =>
  props.files.map((f) => f.split(/[\\/]/).pop() || f)
);

// 通过系统对话框追加
const pick = async () => {
  const selected = await open({ multiple: true, filters: filters.value });
  if (!selected) return;
  const next = (Array.isArray(selected) ? selected : [selected]).filter(
    (p): p is string => typeof p === "string"
  );
  append(next);
};

const append = (paths: string[]) => {
  const existed = new Set(props.files);
  const fresh = paths.filter((p) => !existed.has(p));
  if (fresh.length) emit("update", [...props.files, ...fresh]);
};

const remove = (i: number) => {
  const next = props.files.slice();
  next.splice(i, 1);
  emit("update", next);
};

const move = (i: number, dir: number) => {
  const j = i + dir;
  if (j < 0 || j >= props.files.length) return;
  const next = props.files.slice();
  [next[i], next[j]] = [next[j], next[i]];
  emit("update", next);
};

const clear = () => emit("update", []);

// 真实拖拽（Tauri webview 返回本地路径）
let unlisten: (() => void) | null = null;
onMounted(async () => {
  try {
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      const { type, paths } = event.payload as { type: string; paths: string[] };
      if (type === "over") dragging.value = true;
      else if (type === "leave") dragging.value = false;
      else if (type === "drop") {
        dragging.value = false;
        append(paths.filter(accept));
      } else dragging.value = false;
    });
  } catch {
    /* 非 Tauri 环境忽略 */
  }
});
onBeforeUnmount(() => {
  unlisten?.();
});
</script>

<template>
  <div class="pfp">
    <!-- 标题 + 数量 + 清空 -->
    <div class="pfp-head">
      <span class="tb-editor-label">{{ title }}</span>
      <div class="pfp-head-right">
        <span v-if="files.length" class="pfp-count">{{ files.length }} 个文件</span>
        <n-button v-if="files.length" text size="small" type="error" @click="clear">
          <template #icon><n-icon><Close /></n-icon></template>
          清空
        </n-button>
      </div>
    </div>

    <!-- 拖拽区 -->
    <div
      class="pfp-drop"
      :class="{ 'pfp-drop--over': dragging }"
      @click="pick"
    >
      <div class="pfp-drop-icon">
        <n-icon><CloudUpload /></n-icon>
      </div>
      <div class="pfp-drop-main">
        <div class="pfp-drop-title">拖拽文件到这里</div>
        <div class="pfp-drop-sub">或点击选择文件 · {{ emptyHint }}</div>
      </div>
      <div class="pfp-drop-ext">.{{ extensions.slice(0, 4).join(" / .") }}</div>
    </div>

    <!-- 文件列表 -->
    <div v-if="files.length" class="pfp-list">
      <div v-for="(name, i) in fileNames" :key="i" class="pfp-item">
        <span class="pfp-item-index">
          <n-icon v-if="isImage"><Image :style="{ fontSize: '20px', color: 'var(--tb-accent)' }" /></n-icon>
          <n-icon v-else><DocumentPdf :style="{ fontSize: '20px', color: 'var(--tb-primary)' }" /></n-icon>
          <b>{{ i + 1 }}</b>
        </span>
        <span class="pfp-item-name" :title="props.files[i]">{{ name }}</span>
        <div class="pfp-item-ops">
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary :disabled="i === 0" @click="move(i, -1)">
                <template #icon><n-icon><ArrowUp /></n-icon></template>
              </n-button>
            </template>
            上移
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary :disabled="i === fileNames.length - 1" @click="move(i, 1)">
                <template #icon><n-icon><ArrowDown /></n-icon></template>
              </n-button>
            </template>
            下移
          </n-tooltip>
          <n-tooltip trigger="hover">
            <template #trigger>
              <n-button size="small" quaternary type="error" @click="remove(i)">
                <template #icon><n-icon><Close /></n-icon></template>
              </n-button>
            </template>
            移除
          </n-tooltip>
        </div>
      </div>
    </div>

    <!-- 追加按钮 -->
    <n-button v-if="files.length" dashed class="pfp-add" @click="pick">
      <template #icon><n-icon><Add /></n-icon></template>
      继续添加
    </n-button>
  </div>
</template>

<style scoped>
.pfp {
  display: flex;
  flex-direction: column;
  gap: 12px;
  min-height: 0;
}
.pfp-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
}
.pfp-head .tb-editor-label {
  margin-bottom: 0;
}
.pfp-head-right {
  display: flex;
  align-items: center;
  gap: 10px;
}
.pfp-count {
  font-size: 12px;
  color: var(--tb-primary);
  background: var(--tb-primary-weak);
  padding: 2px 8px;
  border-radius: 999px;
  font-weight: 600;
}
.pfp-drop {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px;
  border: 1.5px dashed var(--tb-border-strong);
  border-radius: var(--tb-radius-m);
  cursor: pointer;
  transition: border-color 0.15s ease, background 0.15s ease, transform 0.15s ease;
}
.pfp-drop:hover {
  border-color: var(--tb-primary);
  background: var(--tb-primary-weak);
}
.pfp-drop--over {
  border-color: var(--tb-primary);
  background: var(--tb-primary-weak);
  transform: scale(1.01);
}
.pfp-drop-icon {
  width: 40px;
  height: 40px;
  display: grid;
  place-items: center;
  border-radius: 10px;
  background: var(--tb-primary-weak);
  color: var(--tb-primary);
  font-size: 20px;
  flex: 0 0 auto;
}
.pfp-drop-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--tb-text);
}
.pfp-drop-sub {
  font-size: 12.5px;
  color: var(--tb-text-3);
  margin-top: 2px;
}
.pfp-drop-ext {
  margin-left: auto;
  font-size: 12px;
  color: var(--tb-text-3);
  font-family: var(--tb-font-mono);
  white-space: nowrap;
}
.pfp-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}
.pfp-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px 10px;
  border: 1px solid var(--tb-border);
  border-radius: var(--tb-radius-m);
  background: var(--tb-bg-app);
  transition: border-color 0.15s ease;
}
.pfp-item:hover {
  border-color: var(--tb-border-strong);
}
.pfp-item-index {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--tb-text-3);
  min-width: 70px;
}
.pfp-item-index b {
  color: var(--tb-text-2);
  font-variant-numeric: tabular-nums;
}
.pfp-item-name {
  flex: 1;
  font-size: 13px;
  color: var(--tb-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.pfp-item-ops {
  display: flex;
  align-items: center;
  gap: 2px;
}
.pfp-add {
  width: 100%;
}
</style>