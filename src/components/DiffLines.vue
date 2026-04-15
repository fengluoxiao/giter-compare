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

<style scoped>
.diff-lines {
  display: flex;
  flex-direction: column;
  min-width: fit-content;
}

.diff-line {
  display: flex;
  min-height: 24px;
  line-height: 24px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
}

.diff-line.unchanged {
  background-color: transparent;
}

.diff-line.added {
  background-color: rgba(76, 175, 80, 0.2);
}

.diff-line.removed {
  background-color: rgba(244, 67, 54, 0.2);
}

.diff-line.changed {
  background-color: rgba(33, 150, 243, 0.2);
}

.diff-line.empty {
  background-color: transparent;
}

.line-number {
  width: 50px;
  padding: 0 8px;
  text-align: right;
  color: var(--text-secondary);
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  user-select: none;
  flex-shrink: 0;
}

.line-content {
  flex: 1;
  padding: 0 12px;
  white-space: pre;
  color: var(--text-primary);
}

.diff-line.added .line-content {
  color: #4caf50;
}

.diff-line.removed .line-content {
  color: #f44336;
}

.diff-line.changed .line-content {
  color: #2196f3;
}
</style>
