<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useMessage, useLoadingBar } from "naive-ui";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { ArchiveOutlined } from "@vicons/material";
import { Copy, Paste } from "@vicons/carbon";

const message = useMessage();
const loadingBar = useLoadingBar();

// 校验值
const checksum = ref("");
// 校验值对比
const validValue = ref("");
// 校验算法
const checksumAlgorithm = ref("md5sum");
// 校验算法
const checksumOptions = ref([
  {
    label: "md5sum",
    value: "md5sum",
  },
  {
    label: "sha1sum",
    value: "sha1sum",
  },
  {
    label: "sha2_224sum",
    value: "sha2_224sum",
  },
  {
    label: "sha2_256sum",
    value: "sha2_256sum",
  },
  {
    label: "sha2_384sum",
    value: "sha2_384sum",
  },
  {
    label: "sha2_512sum",
    value: "sha2_512sum",
  },
  {
    label: "sha3_256sum",
    value: "sha3_256sum",
  },
  {
    label: "sha3_384sum",
    value: "sha3_384sum",
  },
  {
    label: "sha3_512sum",
    value: "sha3_512sum",
  },
]);

// checksum API
const api = async (type, filePath) => {
  return await invoke("checksum", { type, filePath })
    .then((res) => {
      return res;
    })
    .catch((error) => message.error(error));
};

// 文件路径
const filePath = ref("");
// 上传
const handleUpload = async () => {
  try {
    // 使用 Tauri 文件对话框
    const selected = await open({
      // 单选文件
      multiple: false,
      filters: [
        {
          name: "All Files",
          extensions: ["*"],
        },
      ],
    });

    if (selected) {
      // 选择的文件路径
      filePath.value = selected;
    }
  } catch (error) {
    console.error("文件选择错误:", error);
    loadingBar.error();
  }
};

// 上传
const upload = async () => {
  checksum.value = "";
  loadingBar.start();

  // 文件校验
  checksum.value = await api(checksumAlgorithm.value, filePath.value);
  loadingBar.finish();
};

// 校验值对比
const checksumMatched = computed(() => {
  return checksum.value === validValue.value;
});

// 复制
const copy = (value) => {
  if (!value) {
    return;
  }
  writeText(value);
  message.success("复制成功");
};
// 粘贴
const paste = async () => {
  validValue.value = await readText();
};
</script>

<template>
  <n-form label-placement="left" label-width="180">
    <n-form-item label=" ">
      <n-flex align="center">
        <n-button size="large" type="success" @click="handleUpload"
          >上传</n-button
        >
        <span>{{ filePath }}</span>
      </n-flex>
    </n-form-item>

    <!-- 校验算法 -->
    <n-form-item label="校验算法">
      <n-select v-model:value="checksumAlgorithm" :options="checksumOptions" />
      <n-button type="success" @click="upload"> 计算 </n-button>
    </n-form-item>

    <!-- 校验值 -->
    <n-form-item label="校验值">
      <n-button @click="paste">
        <template #icon>
          <n-icon>
            <Paste />
          </n-icon>
        </template>
      </n-button>
      <n-button @click="copy(checksum)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label=" ">
      <n-input
        placeholder=""
        type="textarea"
        :autosize="{
          minRows: 3,
        }"
        readonly
        v-model:value="checksum"
      />
    </n-form-item>

    <n-form-item label="对比值">
      <n-button-group>
        <n-button @click="paste">
          <template #icon>
            <n-icon>
              <Paste />
            </n-icon>
          </template>
        </n-button>
        <n-button @click="copy(validValue)">
          <template #icon>
            <n-icon>
              <Copy />
            </n-icon>
          </template>
        </n-button>
      </n-button-group>
    </n-form-item>
    <n-form-item label=" ">
      <n-input
        placeholder=""
        type="textarea"
        :autosize="{
          minRows: 3,
        }"
        readonly
        v-model:value="validValue"
      />
    </n-form-item>
    <n-form-item label=" " v-if="validValue">
      <span :class="checksumMatched ? 'matched' : 'notMatched'">{{
        checksumMatched ? "一致" : "不一致"
      }}</span>
    </n-form-item>
  </n-form>
</template>

<style lang="scss" scoped>
.matched {
  font-size: 24px;
  color: #18a058;
}

.notMatched {
  font-size: 24px;
  color: #d03050;
}
</style>
