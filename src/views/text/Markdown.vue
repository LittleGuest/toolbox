<script setup>
import { ref } from "vue";
import { MdEditor } from 'md-editor-v3';
import 'md-editor-v3/lib/style.css';

const text = ref('');

</script>

<template>
  <MdEditor v-model="text" style="min-height: 1000px;" />
</template>
