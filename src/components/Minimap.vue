<template>
  <div class="minimap" ref="minimapRef" @click="handleClick">
    <div class="minimap-content">
      <div
        v-for="(line, index) in visibleLines"
        :key="index"
        class="minimap-line"
        :class="getLineClass(line)"
        :title="`Line ${line.lineNum}: ${line.content.slice(0, 50)}`"
      ></div>
    </div>
    <div class="minimap-viewport" :style="viewportStyle"></div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';

interface DiffLine {
  lineNum: number;
  content: string;
  changeType: string;
  isDiff: boolean;
}

const props = defineProps<{
  lines: DiffLine[];
  scrollTop: number;
  containerHeight: number;
  contentHeight: number;
}>();

const emit = defineEmits<{
  jump: [lineIndex: number];
}>();

const minimapRef = ref<HTMLElement | null>(null);

// 压缩显示的行数（最多显示 100 行）
const visibleLines = computed(() => {
  if (props.lines.length <= 100) {
    return props.lines;
  }
  // 如果行数太多，进行采样
  const result: DiffLine[] = [];
  const step = props.lines.length / 100;
  for (let i = 0; i < 100; i++) {
    const index = Math.floor(i * step);
    result.push(props.lines[index]);
  }
  return result;
});

// 视口指示器样式
const viewportStyle = computed(() => {
  if (!props.contentHeight || props.contentHeight <= props.containerHeight) {
    return { display: 'none' };
  }
  
  const ratio = props.containerHeight / props.contentHeight;
  const top = (props.scrollTop / props.contentHeight) * 100;
  const height = ratio * 100;
  
  return {
    top: `${Math.min(top, 100 - height)}%`,
    height: `${Math.max(height, 5)}%`,
  };
});

// 获取行样式类
const getLineClass = (line: DiffLine) => {
  switch (line.changeType) {
    case 'added':
      return 'line-added';
    case 'removed':
      return 'line-removed';
    case 'changed':
      return 'line-changed';
    default:
      return 'line-normal';
  }
};

// 处理点击事件
const handleClick = (e: MouseEvent) => {
  if (!minimapRef.value || props.lines.length === 0) return;
  
  const rect = minimapRef.value.getBoundingClientRect();
  const clickY = e.clientY - rect.top;
  const ratio = clickY / rect.height;
  const lineIndex = Math.floor(ratio * props.lines.length);
  
  emit('jump', Math.max(0, Math.min(lineIndex, props.lines.length - 1)));
};
</script>

<style scoped>
.minimap {
  width: 60px;
  height: 100%;
  background-color: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
  position: relative;
  overflow: hidden;
  cursor: pointer;
}

.minimap-content {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.minimap-line {
  flex: 1;
  min-height: 2px;
  width: 100%;
  transition: opacity 0.2s;
}

.minimap-line:hover {
  opacity: 0.8;
}

.line-normal {
  background-color: var(--text-secondary);
  opacity: 0.2;
}

.line-added {
  background-color: #4caf50;
}

.line-removed {
  background-color: #f44336;
}

.line-changed {
  background-color: #2196f3;
}

.minimap-viewport {
  position: absolute;
  left: 0;
  width: 100%;
  background-color: var(--accent-color);
  opacity: 0.3;
  border-top: 1px solid var(--accent-color);
  border-bottom: 1px solid var(--accent-color);
  pointer-events: none;
}
</style>
