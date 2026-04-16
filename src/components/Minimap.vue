<template>
  <div
    class="minimap"
    ref="minimapRef"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  >
    <div
      v-for="(line, index) in scaledLines"
      :key="index"
      class="minimap-line"
      :class="line.changeType"
      :style="{ height: scaledLineHeight + 'px' }"
    ></div>
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
  leftLines: DiffLine[];
  rightLines: DiffLine[];
  scrollTop: number;
  containerHeight: number;
  contentHeight: number;
}>();

const emit = defineEmits<{
  jump: [lineIndex: number];
}>();

const minimapRef = ref<HTMLElement | null>(null);
const isDragging = ref(false);

// 合并左右两侧的 lines，优先显示有变更的类型
const mergedLines = computed(() => {
  const maxLength = Math.max(props.leftLines.length, props.rightLines.length);
  const result: DiffLine[] = [];

  for (let i = 0; i < maxLength; i++) {
    const leftLine = props.leftLines[i];
    const rightLine = props.rightLines[i];

    // 优先顺序：added > removed > changed > unchanged > empty
    if (leftLine?.changeType === 'added') {
      result.push(leftLine);
    } else if (rightLine?.changeType === 'removed') {
      result.push(rightLine);
    } else if (leftLine?.changeType === 'changed' || rightLine?.changeType === 'changed') {
      result.push(leftLine || rightLine);
    } else if (leftLine?.changeType === 'unchanged') {
      result.push(leftLine);
    } else if (rightLine?.changeType === 'unchanged') {
      result.push(rightLine);
    } else if (leftLine) {
      result.push(leftLine);
    } else if (rightLine) {
      result.push(rightLine);
    }
  }

  return result;
});

// 将多行合并为一个像素行，实现缩放效果
const MINIMAP_LINE_HEIGHT = 2; // 每行像素高度

const scaledLines = computed(() => {
  const totalLines = mergedLines.value.length;
  if (totalLines === 0) return [];

  // 根据容器高度计算需要多少像素行
  const minimapHeight = props.containerHeight || 200;
  const pixelRows = Math.ceil(minimapHeight / MINIMAP_LINE_HEIGHT);
  const linesPerPixel = totalLines / pixelRows;

  const result: DiffLine[] = [];

  for (let i = 0; i < pixelRows; i++) {
    const startLine = Math.floor(i * linesPerPixel);
    const endLine = Math.floor((i + 1) * linesPerPixel);

    // 找出这个像素范围内优先级最高的变更类型
    let changeType = 'unchanged';

    for (let j = startLine; j < endLine && j < totalLines; j++) {
      const line = mergedLines.value[j];
      if (!line) continue;

      // 优先级：added > removed > changed > unchanged > empty
      if (line.changeType === 'added') {
        changeType = 'added';
        break; // 最高优先级，直接跳出
      } else if (line.changeType === 'removed' && changeType !== 'added') {
        changeType = 'removed';
      } else if (line.changeType === 'changed' && changeType !== 'added' && changeType !== 'removed') {
        changeType = 'changed';
      } else if (line.changeType === 'empty' && changeType === 'unchanged') {
        changeType = 'empty';
      }
    }

    result.push({
      lineNum: i,
      content: '',
      changeType,
      isDiff: changeType !== 'unchanged' && changeType !== 'empty'
    });
  }

  return result;
});

const scaledLineHeight = computed(() => {
  return MINIMAP_LINE_HEIGHT;
});

const viewportStyle = computed(() => {
  const totalLines = mergedLines.value.length;
  if (totalLines === 0) {
    return { display: 'none' };
  }

  // 计算可见行数
  const estimatedLineHeight = 20; // 代码编辑器中每行大约20px
  const visibleLines = Math.ceil(props.containerHeight / estimatedLineHeight);

  if (totalLines <= visibleLines) {
    return { display: 'none' };
  }

  // 计算滚动比例
  const currentLine = Math.floor(props.scrollTop / estimatedLineHeight);
  const scrollRatio = Math.min(currentLine / totalLines, 1);
  const viewportRatio = Math.min(visibleLines / totalLines, 1);

  // 在缩放的 minimap 上显示 viewport
  const minimapHeight = scaledLines.value.length * MINIMAP_LINE_HEIGHT;
  const top = scrollRatio * minimapHeight;
  const height = Math.max(viewportRatio * minimapHeight, 10);

  return {
    top: `${top}px`,
    height: `${height}px`,
  };
});

const jumpToPosition = (clientY: number) => {
  if (!minimapRef.value || mergedLines.value.length === 0) return;

  const rect = minimapRef.value.getBoundingClientRect();
  const clickY = clientY - rect.top;
  const pixelRow = Math.floor(clickY / MINIMAP_LINE_HEIGHT);

  // 将像素行转换回实际行号
  const totalLines = mergedLines.value.length;
  const pixelRows = scaledLines.value.length;
  const linesPerPixel = totalLines / pixelRows;
  const lineIndex = Math.floor(pixelRow * linesPerPixel);

  emit('jump', Math.max(0, Math.min(lineIndex, totalLines - 1)));
};

const handleMouseDown = (e: MouseEvent) => {
  isDragging.value = true;
  jumpToPosition(e.clientY);
};

const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return;
  jumpToPosition(e.clientY);
};

const handleMouseUp = () => {
  isDragging.value = false;
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
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
}

.minimap-line {
  width: 100%;
  flex-shrink: 0;
}

.minimap-line.unchanged {
  background-color: rgba(128, 128, 128, 0.15);
}

.minimap-line.added {
  background-color: rgba(76, 175, 80, 0.9);
}

.minimap-line.removed {
  background-color: rgba(244, 67, 54, 0.9);
}

.minimap-line.changed {
  background-color: rgba(33, 150, 243, 0.9);
}

.minimap-line.empty {
  background-color: rgba(128, 128, 128, 0.1);
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
