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
      v-for="(line, index) in mergedLines"
      :key="index"
      class="minimap-line"
      :class="line.changeType"
      :style="{ height: lineHeight + 'px' }"
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

const MINIMAP_LINE_HEIGHT = 3;

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

const minimapHeight = computed(() => {
  return mergedLines.value.length * MINIMAP_LINE_HEIGHT;
});

const lineHeight = computed(() => {
  return MINIMAP_LINE_HEIGHT;
});

const viewportStyle = computed(() => {
  if (!props.contentHeight || props.contentHeight <= props.containerHeight) {
    return { display: 'none' };
  }

  const scrollRatio = props.scrollTop / props.contentHeight;
  const viewportRatio = props.containerHeight / props.contentHeight;

  const top = scrollRatio * minimapHeight.value;
  const height = Math.max(viewportRatio * minimapHeight.value, 20);

  return {
    top: `${top}px`,
    height: `${height}px`,
  };
});

const jumpToPosition = (clientY: number) => {
  if (!minimapRef.value || mergedLines.value.length === 0) return;

  const rect = minimapRef.value.getBoundingClientRect();
  const clickY = clientY - rect.top;
  const lineIndex = Math.floor(clickY / MINIMAP_LINE_HEIGHT);

  emit('jump', Math.max(0, Math.min(lineIndex, mergedLines.value.length - 1)));
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
