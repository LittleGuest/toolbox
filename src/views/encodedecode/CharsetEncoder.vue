<script setup>
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import { useMessage } from "naive-ui";
import { ArrowDown, Copy, Paste, Close } from "@vicons/carbon";

const message = useMessage();

const inputType = ref("text");
const targetCharset = ref("UTF-8");
const input = ref("");

const outputType = ref("hex");
const delimiterType = ref("space");
const customDelimiter = ref("");
const baseFormat = ref("none");
const output = ref("");

const showUnicode = ref(false);
const showEscape = ref(false);
const showCArray = ref(false);
const showAssembly = ref(false);
const showAuto = ref(false);
const invertNonPrintable = ref(false);
const appendNull = ref(false);

const byteCount = ref(0);
const charCount = ref(0);

const delimiter = computed(() => {
  if (delimiterType.value === "custom") {
    return customDelimiter.value;
  }
  switch (delimiterType.value) {
    case "space":
      return " ";
    case "comma":
      return ",";
    case "colon":
      return ":";
    case "semicolon":
      return ";";
    case "newline":
      return "\n";
    default:
      return " ";
  }
});

const charsetOptions = [
  { label: "UTF-8", value: "UTF-8" },
  { label: "GBK", value: "GBK" },
  { label: "UTF-16BE", value: "UTF-16BE" },
  { label: "UTF-16LE", value: "UTF-16LE" },
  { label: "UTF-32BE", value: "UTF-32BE" },
  { label: "UTF-32LE", value: "UTF-32LE" },
  { label: "ASCII", value: "ASCII" }
];

const inputTypeOptions = [
  { label: "文本", value: "text" },
  { label: "十六进制", value: "hex" },
  { label: "十进制", value: "decimal" },
  { label: "八进制", value: "octal" },
  { label: "二进制", value: "binary" }
];

const outputTypeOptions = [
  { label: "十六进制(Hex)", value: "hex" },
  { label: "十进制(Dec)", value: "decimal" },
  { label: "八进制(Oct)", value: "octal" },
  { label: "二进制(Bin)", value: "binary" }
];

const delimiterOptions = [
  { label: "空格", value: "space" },
  { label: ", ", value: "comma" },
  { label: ": ", value: "colon" },
  { label: "; ", value: "semicolon" },
  { label: "换行", value: "newline" },
  { label: "自定义", value: "custom" }
];

const baseFormatOptions = [
  { label: "无", value: "none" },
  { label: "0x前缀", value: "0x" },
  { label: "0b前缀", value: "0b" },
  { label: "0o前缀", value: "0o" },
  { label: "h后缀", value: "h" }
];

const convert = async () => {
  try {
    const result = await invoke("charset_encode", {
      input: input.value,
      inputType: inputType.value,
      targetCharset: targetCharset.value,
      outputType: outputType.value,
      delimiter: delimiter.value,
      baseFormat: baseFormat.value,
      showUnicode: showUnicode.value,
      showEscape: showEscape.value,
      showCArray: showCArray.value,
      showAssembly: showAssembly.value,
      showAuto: showAuto.value,
      invertNonPrintable: invertNonPrintable.value,
      appendNull: appendNull.value
    });
    
    output.value = result.output;
    byteCount.value = result.byteCount;
    charCount.value = result.charCount;
  } catch (error) {
    message.error(error);
  }
};

const autoDetect = async () => {
  try {
    const result = await invoke("auto_detect_charset", {
      input: input.value
    });
    targetCharset.value = result.charset;
    message.success(`自动检测到编码: ${result.charset}`);
    convert();
  } catch (error) {
    message.error(error);
  }
};

const pasteInput = async () => {
  input.value = await readText();
};

const pasteOutput = async () => {
  output.value = await readText();
};

const copy = (value) => {
  writeText(value);
};

const clear = () => {
  input.value = "";
  output.value = "";
  byteCount.value = 0;
  charCount.value = 0;
};
</script>

<template>
  <n-scrollbar>
    <n-form label-placement="left" label-width="120">
      <n-form-item label="输入类型">
        <n-select placeholder="请选择输入类型" :options="inputTypeOptions" v-model:value="inputType" />
      </n-form-item>
      <n-form-item label="目标编码">
        <n-select placeholder="请选择目标编码" :options="charsetOptions" v-model:value="targetCharset" />
      </n-form-item>
      <n-form-item label="操作">
        <n-button-group>
          <n-button @click="pasteInput">
            <template #icon>
              <n-icon>
                <Paste />
              </n-icon>
            </template>
          </n-button>
          <n-button @click="copy(input)">
            <template #icon>
              <n-icon>
                <Copy />
              </n-icon>
            </template>
          </n-button>
          <n-button @click="autoDetect">
            自动检测
          </n-button>
          <n-button @click="clear">
            <template #icon>
              <n-icon>
                <Close />
              </n-icon>
            </template>
          </n-button>
        </n-button-group>
      </n-form-item>
      <n-form-item label="输入">
        <n-input placeholder="请输入文本或编码数据" v-model:value="input" :rows="6" type="textarea" />
      </n-form-item>
      <n-form-item label="转换">
        <n-button @click="convert">
          <template #icon>
            <n-icon>
              <ArrowDown />
            </n-icon>
          </template>
        </n-button>
      </n-form-item>
      <n-form-item label="输出类型">
        <n-select placeholder="请选择输出类型" :options="outputTypeOptions" v-model:value="outputType" />
      </n-form-item>
      <n-form-item label="分隔符">
        <n-select placeholder="请选择分隔符" :options="delimiterOptions" v-model:value="delimiterType" />
      </n-form-item>
      <n-form-item label="自定义分隔符" v-if="delimiterType === 'custom'">
        <n-input placeholder="输入自定义分隔符" v-model:value="customDelimiter" style="width: 200px" />
      </n-form-item>
      <n-form-item label="进制格式">
        <n-select placeholder="请选择进制格式" :options="baseFormatOptions" v-model:value="baseFormat" />
      </n-form-item>
      <n-form-item label="显示选项">
        <div class="option-group">
          <n-checkbox v-model:checked="showUnicode">Unicode码点</n-checkbox>
          <n-checkbox v-model:checked="showEscape">转义序列</n-checkbox>
          <n-checkbox v-model:checked="showCArray">C/C++数组</n-checkbox>
          <n-checkbox v-model:checked="showAssembly">汇编数据</n-checkbox>
          <n-checkbox v-model:checked="showAuto">自动</n-checkbox>
          <n-checkbox v-model:checked="invertNonPrintable">反转不可打印字符</n-checkbox>
          <n-checkbox v-model:checked="appendNull">追加NUL结尾</n-checkbox>
        </div>
      </n-form-item>
      <n-form-item label="操作">
        <n-button-group>
          <n-button @click="pasteOutput">
            <template #icon>
              <n-icon>
                <Paste />
              </n-icon>
            </template>
          </n-button>
          <n-button @click="copy(output)">
            <template #icon>
              <n-icon>
                <Copy />
              </n-icon>
            </template>
          </n-button>
          <n-button @click="clear">
            <template #icon>
              <n-icon>
                <Close />
              </n-icon>
            </template>
          </n-button>
        </n-button-group>
      </n-form-item>
      <n-form-item label="输出">
        <n-input placeholder="转换结果" v-model:value="output" :rows="6" type="textarea" />
      </n-form-item>
      <n-form-item label="统计信息">
        <span>字节: {{ byteCount }} 字符: {{ charCount }}</span>
      </n-form-item>
    </n-form>
  </n-scrollbar>
</template>

<style scoped>
.option-group {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
}
</style>