<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useMessage } from "naive-ui";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { ArchiveOutlined } from "@vicons/material";
import { Copy } from "@vicons/carbon";

const message = useMessage();

const input = ref("");
const output = ref("");
const chechsum = ref({});
const fileList = ref([]);

const api = async () => {
  return await invoke("checksum", {})
    .then((res) => {
      return res;
    })
    .catch((error) => message.error(error));
};

// 自定义上传
const customRequest = async ({
  file,
  data,
  headers,
  withCredentials,
  action,
  onFinish,
  onError,
  onProgress,
}) => {
  try {
    // 读取文件为 ArrayBuffer
    const arrayBuffer = await file.file.arrayBuffer();
    const uint8Array = new Uint8Array(arrayBuffer);

    // FIXME: 大文件上传

    const res = await invoke("checksum", {
      fileName: file.name,
      fileData: Array.from(uint8Array), // 转换为普通数组
    });
    chechsum.value = res;
    return file;
  } catch (error) {
    console.error("上传失败:", error);
    throw error;
  }
};

// 上传完成的回调
const handleFinish = ({ file, event }) => {
  // console.log(event);
  // message.success((event?.target).response);
  // const ext = file.name.split(".")[1];
  // file.name = `更名.${ext}`;
  // file.url = "__HTTPS__://www.mocky.io/v2/5e4bafc63100007100d8b70f";
  // return file;
  message.success(`文件 ${file.name} 上传成功！`);
};

// 上传前的回调
const handleBeforeUpload = ({ file }) => {
  // 只上传一个文件
  fileList.value = [file];
  return true;
};

// 复制
const copy = (value) => {
  if (!value) {
    return;
  }
  writeText(value);
  message.success("复制成功");
};
</script>

<template>
  <n-form label-placement="left" label-width="180">
    <n-form-item>
      <n-upload
        :show-file-list="true"
        v-model:file-list="fileList"
        :custom-request="customRequest"
        @finish="handleFinish"
        :on-before-upload="handleBeforeUpload"
      >
        <n-upload-dragger>
          <div>
            <n-icon size="48" :depth="3">
              <ArchiveOutlined />
            </n-icon>
          </div>
          <n-text> 点击或者拖动文件到该区域来上传 </n-text>
        </n-upload-dragger>
      </n-upload>
    </n-form-item>

    <n-form-item label="MD5">
      <n-input placeholder="" readonly v-model:value="chechsum.md5" />
      <n-button @click="copy(chechsum.md5)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA1">
      <n-input placeholder="" readonly v-model:value="chechsum.sha1" />
      <n-button @click="copy(chechsum.sha1)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA256">
      <n-input placeholder="" readonly v-model:value="chechsum.sha256" />
      <n-button @click="copy(chechsum.sha256)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA512">
      <n-input placeholder="" readonly v-model:value="chechsum.sha512" />
      <n-button @click="copy(chechsum.sha512)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA2 224">
      <n-input placeholder="" readonly v-model:value="chechsum.sha2_224" />
      <n-button @click="copy(chechsum.sha2_224)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA2 384">
      <n-input placeholder="" readonly v-model:value="chechsum.sha2_384" />
      <n-button @click="copy(chechsum.sha2_384)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA3 256">
      <n-input placeholder="" readonly v-model:value="chechsum.sha3_256" />
      <n-button @click="copy(chechsum.sha3_256)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA3 384">
      <n-input placeholder="" readonly v-model:value="chechsum.sha3_384" />
      <n-button @click="copy(chechsum.sha3_384)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
    <n-form-item label="SHA3 512">
      <n-input placeholder="" readonly v-model:value="chechsum.sha3_512" />
      <n-button @click="copy(chechsum.sha3_512)">
        <template #icon>
          <n-icon>
            <Copy />
          </n-icon>
        </template>
      </n-button>
    </n-form-item>
  </n-form>
</template>
