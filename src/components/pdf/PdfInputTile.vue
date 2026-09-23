<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount } from "vue";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { open } from "@tauri-apps/plugin-dialog";
import { Document } from "@vicons/carbon";

const props = withDefaults(
  defineProps<{
    modelValue: string;
    label?: string;
    placeholder?: string;
    extensions?: string[];
  }>(),
  {
    label: "选择 PDF",
    placeholder: "点击或拖拽 PDF 到此处",
    extensions: () => ["pdf"],
  }
);

const emit = defineEmits<{ (e: "update:modelValue", val: string): void }>();

const accept = (p: string) => {
  const ext = p.split(".").pop()?.toLowerCase() || "";
  return props.extensions.includes(ext);
};

const shortName = (p: string) => p.split(/[\\/]/).pop() || p;

const pick = async () => {
  const sel = await open({
    multiple: false,
    filters: [{ name: "支持的文件", extensions: props.extensions }],
  });
  if (sel && typeof sel === "string") emit("update:modelValue", sel);
};

let unlisten: (() => void) | null = null;
onMounted(async () => {
  try {
    unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      const { type, paths } = event.payload as { type: string; paths: string[] };
      const hit = paths.find(accept);
      if (type === "drop" && hit) emit("update:modelValue", hit);
    });
  } catch {
  }
});
onBeforeUnmount(() => unlisten?.());
</script>

<template>
  <div class="tb-file-tile" @click="pick">
    <span class="tb-file-tile__icon"><n-icon><Document /></n-icon></span>
    <span class="tb-file-tile__main">
      <span class="tb-file-tile__label">{{ modelValue ? label : "尚未选择" }}</span>
      <span
        class="tb-file-tile__value"
        :class="{ 'tb-file-tile__value--empty': !modelValue }"
        :title="modelValue"
      >{{ modelValue ? shortName(modelValue) : placeholder }}</span>
    </span>
    <span class="tb-file-tile__ext">.pdf</span>
  </div>
</template>

<style scoped>
.tb-file-tile__value--empty {
  color: var(--tb-text-3);
}
</style>
