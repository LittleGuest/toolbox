<script setup>
import { ref } from "vue";
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const text = ref('');

</script>

<template>
  <MdEditor v-model="text" style="height: calc(100vh - 48px);" />
</template>
