<template>
  <div class="diff-lines">
    <div
      v-for="(line, index) in lines"
      :key="index"
      class="diff-line"
      :class="line.changeType"
      :data-line="line.lineNum"
    >
      <span class="line-number">{{ line.lineNum > 0 ? line.lineNum : '' }}</span>
      <span class="line-content">{{ line.content || ' ' }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  lines: {
    lineNum: number;
    content: string;
    changeType: string;
    isDiff: boolean;
  }[];
}>();
</script>
