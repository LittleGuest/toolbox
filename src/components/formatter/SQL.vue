<script setup>
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/tauri";
import { writeText, readText } from '@tauri-apps/api/clipboard';
import { Document, CopyDocument } from "@element-plus/icons-vue";
import { format } from 'sql-formatter';

const language = ref("mysql");
const indent = ref(2);
const upper = ref('upper');
const sql = ref("");

const formatSql = () => {
  sql.value = format(sql.value, { language: language.value, tabWidth: indent.value, keywordCase: "upper" });
}

const paste = async () => {
  const clip = await readText();
  sql.value = clip;
};

const copy = (value) => {
  writeText(value);
};
</script>

<template>
  <el-form label-position="right" label-width="100px">
    <el-form-item label="语言">
      <el-select v-model="language" class="m-2" size="large">
        <el-option key="MySQL" label="MySQL" value="mysql" />
        <el-option key="Sqlite" label="Sqlite" value="sqlite" />
        <el-option key="PostgreSQL" label="PostgreSQL" value="postgresql" />
      </el-select>
    </el-form-item>

    <el-form-item label="缩进">
      <el-select v-model="indent" class="m-2" size="large">
        <el-option key="2" label="2" value="2" />
        <el-option key="4" label="4" value="4" />
      </el-select>
    </el-form-item>

    <el-row>
      <el-col>
        <el-form-item label="SQL">
          <el-button-group class="ml-4">
            <el-button type="primary" :icon="Document" @click="paste(sql)" />
            <el-button type="primary" :icon="CopyDocument" @click="copy(sql)" />
            <el-button @click="formatSql">格式化</el-button>
          </el-button-group>
          <el-input v-model="sql" :rows="10" type="textarea" />
        </el-form-item>
      </el-col>
    </el-row>
  </el-form>
</template>
