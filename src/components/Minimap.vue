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
  leftLines: DiffLine[];
  rightLines: DiffLine[];
  scrollTop: number;
  containerHeight: number;
  contentHeight: number;
}>();

const emit = defineEmits<{
  jump: [scrollTop: number];
}>();

const minimapRef = ref<HTMLElement | null>(null);
const canvasRef = ref<HTMLCanvasElement | null>(null);
const isDragging = ref(false);

// 合并左右两侧的 lines
const mergedLines = computed(() => {
  const maxLength = Math.max(props.leftLines.length, props.rightLines.length);
  const result: DiffLine[] = [];

  for (let i = 0; i < maxLength; i++) {
    const leftLine = props.leftLines[i];
    const rightLine = props.rightLines[i];

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

// 检测深色模式
const isDarkMode = () => {
  return document.body.getAttribute('data-theme') === 'dark';
};

// 颜色定义（根据主题动态选择）
const getColors = () => {
  const dark = isDarkMode();
  return {
    background: dark ? { r: 30, g: 30, b: 30 } : { r: 240, g: 240, b: 240 },
    unchanged: dark ? { r: 60, g: 60, b: 60 } : { r: 212, g: 212, b: 212 },
    added: dark ? { r: 76, g: 175, b: 80 } : { r: 76, g: 175, b: 80 },
    removed: dark ? { r: 244, g: 67, b: 54 } : { r: 244, g: 67, b: 54 },
    changed: dark ? { r: 33, g: 150, b: 243 } : { r: 33, g: 150, b: 243 }
  };
};

// 使用 ImageData 直接操作像素渲染
const render = () => {
  const canvas = canvasRef.value;
  if (!canvas || !minimapRef.value) return;

  const ctx = canvas.getContext('2d');
  if (!ctx) return;

  const lines = mergedLines.value;
  const totalLines = lines.length;

  if (totalLines === 0) return;

  // 设置 canvas 尺寸
  const rect = minimapRef.value.getBoundingClientRect();
  canvas.width = rect.width;
  canvas.height = rect.height;

  // 创建 ImageData
  const imageData = ctx.createImageData(canvas.width, canvas.height);
  const data = imageData.data;

  // 计算每行代码对应的像素高度
  const lineHeight = canvas.height / totalLines;

  // 填充每个像素
  for (let y = 0; y < canvas.height; y++) {
    // 计算这个像素对应的代码行
    const lineIndex = Math.floor(y / lineHeight);
    const line = lines[Math.min(lineIndex, totalLines - 1)];

    // 确定颜色 - 只要不是 unchanged 就是有变更
    const COLORS = getColors();
    let color = COLORS.unchanged;
    if (line) {
      if (line.changeType === 'added' || line.changeType === 'empty') {
        // empty 表示对应侧没有这行，也属于变更
        color = COLORS.added;
      } else if (line.changeType === 'removed') {
        color = COLORS.removed;
      } else if (line.changeType === 'changed') {
        color = COLORS.changed;
      }
    }

    // 填充这一行的所有像素
    for (let x = 0; x < canvas.width; x++) {
      const idx = (y * canvas.width + x) * 4;
      data[idx] = color.r;
      data[idx + 1] = color.g;
      data[idx + 2] = color.b;
      data[idx + 3] = 255;
    }
  }

  // 将 ImageData 绘制到 canvas
  ctx.putImageData(imageData, 0, 0);
};

// 监听数据变化
watch(() => props.leftLines, render, { deep: true });
watch(() => props.rightLines, render, { deep: true });

onMounted(() => {
  render();
  window.addEventListener('resize', render);

  // 监听主题变化
  const observer = new MutationObserver(() => {
    render();
  });
  observer.observe(document.body, {
    attributes: true,
    attributeFilter: ['data-theme']
  });

  // 保存 observer 引用以便清理
  (minimapRef.value as any)?._themeObserver = observer;
});

onUnmounted(() => {
  window.removeEventListener('resize', render);

  // 清理 observer
  const observer = (minimapRef.value as any)?._themeObserver;
  if (observer) {
    observer.disconnect();
  }
});

const viewportStyle = computed(() => {
  const totalLines = mergedLines.value.length;
  if (totalLines === 0) {
    return { display: 'none' };
  }

  // 使用与代码区域一致的行高
  const LINE_HEIGHT = 24;
  const visibleLines = Math.ceil(props.containerHeight / LINE_HEIGHT);

  if (totalLines <= visibleLines) {
    return { display: 'none' };
  }

  // 使用实际的 scrollTop 和行高计算当前行
  const currentLine = Math.floor(props.scrollTop / LINE_HEIGHT);
  
  // 使用像素比例而不是行数比例，更精确
  const totalHeight = totalLines * LINE_HEIGHT;
  const scrollRatio = Math.min(props.scrollTop / totalHeight, 1);
  const viewportRatio = Math.min(props.containerHeight / totalHeight, 1);

  return {
    top: `${scrollRatio * 100}%`,
    height: `${Math.max(viewportRatio * 100, 5)}%`,
  };
});

const jumpToPosition = (clientY: number) => {
  if (!minimapRef.value || mergedLines.value.length === 0) return;

  const rect = minimapRef.value.getBoundingClientRect();
  const clickY = clientY - rect.top;
  const ratio = clickY / rect.height;
  const lineIndex = Math.floor(ratio * mergedLines.value.length);

  // 使用固定的行高计算实际的 scrollTop
  const LINE_HEIGHT = 24;
  const scrollTop = lineIndex * LINE_HEIGHT;

  emit('jump', scrollTop);
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
