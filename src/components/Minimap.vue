<template>
  <div
    class="minimap"
    ref="minimapRef"
    @mousedown="handleMouseDown"
    @mousemove="handleMouseMove"
    @mouseup="handleMouseUp"
    @mouseleave="handleMouseUp"
  >
    <canvas ref="canvasRef" class="minimap-canvas"></canvas>
    <div class="minimap-viewport" :style="viewportStyle"></div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue';

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
const canvasRef = ref<HTMLCanvasElement | null>(null);

// 拖动状态
const isDragging = ref(false);

// 颜色配置 - 使用更高的对比度
const colors = {
  normal: 'rgba(128, 128, 128, 0.15)',
  added: 'rgba(76, 175, 80, 0.9)',
  removed: 'rgba(244, 67, 54, 0.9)',
  changed: 'rgba(33, 150, 243, 0.9)',
  empty: 'rgba(128, 128, 128, 0.05)',
};

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

// 获取行颜色
const getLineColor = (line: DiffLine): string => {
  switch (line.changeType) {
    case 'added':
      return colors.added;
    case 'removed':
      return colors.removed;
    case 'changed':
      return colors.changed;
    case 'empty':
      return colors.empty;
    default:
      return colors.normal;
  }
};

// 绘制 minimap
const drawMinimap = () => {
  const canvas = canvasRef.value;
  const container = minimapRef.value;
  if (!canvas || !container || props.lines.length === 0) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  // 设置 canvas 尺寸
  const width = container.clientWidth;
  const height = container.clientHeight;
  canvas.width = width;
  canvas.height = height;

  // 清空画布
  ctx.clearRect(0, 0, width, height);

  // 计算每行的高度
  const lineHeight = height / props.lines.length;

  // 绘制每一行
  props.lines.forEach((line, index) => {
    const y = index * lineHeight;
    const lineH = Math.max(lineHeight, 1); // 至少 1 像素

    ctx.fillStyle = getLineColor(line);
    ctx.fillRect(0, y, width, lineH);
  });
};

// 计算并跳转到指定位置
const jumpToPosition = (clientY: number) => {
  if (!minimapRef.value || props.lines.length === 0) return;

  const rect = minimapRef.value.getBoundingClientRect();
  const clickY = clientY - rect.top;
  const ratio = clickY / rect.height;
  const lineIndex = Math.floor(ratio * props.lines.length);

  emit('jump', Math.max(0, Math.min(lineIndex, props.lines.length - 1)));
};

// 处理鼠标按下事件
const handleMouseDown = (e: MouseEvent) => {
  isDragging.value = true;
  jumpToPosition(e.clientY);
};

// 处理鼠标移动事件
const handleMouseMove = (e: MouseEvent) => {
  if (!isDragging.value) return;
  jumpToPosition(e.clientY);
};

// 处理鼠标释放事件
const handleMouseUp = () => {
  isDragging.value = false;
};

// 监听线条变化，重新绘制
watch(() => props.lines, drawMinimap, { deep: true });

// 监听尺寸变化，重新绘制
const handleResize = () => {
  drawMinimap();
};

onMounted(() => {
  drawMinimap();
  window.addEventListener('resize', handleResize);
});

onUnmounted(() => {
  window.removeEventListener('resize', handleResize);
});
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
}

.minimap-canvas {
  width: 100%;
  height: 100%;
  display: block;
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
