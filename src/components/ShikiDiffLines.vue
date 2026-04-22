<template>
  <div class="shiki-diff-lines" ref="containerRef">
    <!-- 占位元素，保持滚动高度 -->
    <div class="scroll-spacer" :style="{ height: totalHeight + 'px' }"></div>
    <!-- 可见区域的行 -->
    <div class="visible-lines" :style="{ transform: `translateY(${offsetY}px)` }">
      <div
        v-for="(line, index) in visibleLines"
        :key="`${line.lineNum}-${line.globalIndex}`"
        class="diff-line"
        :class="[line.changeType, { 'current-match': isCurrentMatch(line.globalIndex), 'jump-highlight': isJumpHighlight(line.globalIndex), 'selected': selectedLineIndex === line.globalIndex, 'has-blame': showBlameInfo }]"
        :data-line="line.lineNum"
        @click="onLineClick(line.globalIndex)"
      >
        <span class="line-number">{{ line.lineNum > 0 ? line.lineNum : '' }}</span>
        <span v-if="showBlameInfo" class="blame-info" :title="getBlameTooltip(line.globalIndex)">
          {{ getBlameText(line.globalIndex) }}
        </span>
        <span class="line-content" v-html="line.content"></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed, nextTick } from 'vue';

interface BlameInfo {
  line_number: number;
  commit_hash: string;
  short_hash: string;
  author: string;
  email: string;
  timestamp: number;
  summary: string;
}

interface SearchMatch {
  lineIndex: number;
  columnIndex: number;
  text: string;
}

interface DiffLine {
  lineNum: number;
  content: string;
  changeType: string;
  isDiff: boolean;
}

interface VisibleLine extends DiffLine {
  globalIndex: number;
}

const props = defineProps<{
  lines: DiffLine[];
  filename: string;
  theme?: 'light' | 'dark';
  searchMatches?: SearchMatch[];
  currentMatchIndex?: number;
  highlightedLine?: number | null;
  blameInfo?: BlameInfo[];
}>();

const containerRef = ref<HTMLElement | null>(null);
const scrollTop = ref(0);
const containerHeight = ref(800);

// 虚拟滚动相关
const LINE_HEIGHT = 24; // 每行高度
const BUFFER_LINES = 30; // 上下缓冲行数

// 计算总高度
const totalHeight = computed(() => {
  return props.lines.length * LINE_HEIGHT;
});

// 计算可见区域的起始索引
const startIndex = computed(() => {
  const start = Math.floor(scrollTop.value / LINE_HEIGHT) - BUFFER_LINES;
  return Math.max(0, start);
});

// 计算可见区域的结束索引
const endIndex = computed(() => {
  const end = Math.ceil((scrollTop.value + containerHeight.value) / LINE_HEIGHT) + BUFFER_LINES;
  return Math.min(props.lines.length, end);
});

// 可见的行 - 包含全局索引
const visibleLines = computed((): VisibleLine[] => {
  const result: VisibleLine[] = [];
  const start = startIndex.value;
  const end = endIndex.value;
  for (let i = start; i < end && i < props.lines.length; i++) {
    const line = props.lines[i];
    if (line) {
      result.push({
        ...line,
        globalIndex: i
      });
    }
  }
  return result;
});

// 偏移量
const offsetY = computed(() => {
  return startIndex.value * LINE_HEIGHT;
});

const selectedLineIndex = ref<number | null>(null);

const emit = defineEmits<{
  'line-click': [lineIndex: number, lineNum: number];
}>();

const onLineClick = (globalIndex: number) => {
  selectedLineIndex.value = globalIndex;
  const line = props.lines[globalIndex];
  if (line) {
    emit('line-click', globalIndex, line.lineNum);
  }
};

// 处理滚动事件
const handleScroll = () => {
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop;
  }
};

// 更新容器高度
const updateContainerHeight = () => {
  if (containerRef.value) {
    containerHeight.value = containerRef.value.clientHeight;
  }
};

onMounted(() => {
  updateContainerHeight();
  if (containerRef.value) {
    containerRef.value.addEventListener('scroll', handleScroll);
    window.addEventListener('resize', updateContainerHeight);
  }
});

onUnmounted(() => {
  if (containerRef.value) {
    containerRef.value.removeEventListener('scroll', handleScroll);
    window.removeEventListener('resize', updateContainerHeight);
  }
});

// 监听行数变化，更新滚动位置
watch(() => props.lines.length, () => {
  nextTick(() => {
    if (containerRef.value) {
      scrollTop.value = containerRef.value.scrollTop;
    }
  });
});

const isCurrentMatch = (globalIndex: number): boolean => {
  if (!props.searchMatches || props.currentMatchIndex === undefined) return false;
  const match = props.searchMatches[props.currentMatchIndex];
  return match && match.lineIndex === globalIndex;
};

const isJumpHighlight = (globalIndex: number): boolean => {
  return props.highlightedLine === props.lines[globalIndex]?.lineNum;
};

const getBlameForLine = (globalIndex: number): BlameInfo | undefined => {
  if (!props.blameInfo || props.blameInfo.length === 0) return undefined;
  
  const lineNum = props.lines[globalIndex]?.lineNum;
  if (!lineNum || lineNum <= 0) return undefined;
  
  return props.blameInfo.find(b => b.line_number === lineNum);
};

const getBlameText = (globalIndex: number): string => {
  const blame = getBlameForLine(globalIndex);
  if (!blame) return '';
  return `${blame.author}: ${blame.summary}`;
};

const getBlameTooltip = (globalIndex: number): string => {
  const blame = getBlameForLine(globalIndex);
  if (!blame) return '';
  const date = new Date(blame.timestamp * 1000).toLocaleString();
  return `${blame.short_hash} - ${blame.author} (${blame.email})\n${date}\n${blame.summary}`;
};

const showBlameInfo = computed(() => {
  return props.blameInfo && props.blameInfo.length > 0;
});
</script>

<style scoped>
.shiki-diff-lines {
  position: relative;
  overflow: auto;
  min-width: fit-content;
  height: 100%;
}

.scroll-spacer {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  pointer-events: none;
}

.visible-lines {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
}

.diff-line {
  display: flex;
  align-items: center;
  height: 24px;
  line-height: 24px;
  white-space: pre;
  font-family: 'JetBrains Mono', 'Fira Code', 'Cascadia Code', 'SF Mono', Monaco, monospace;
  font-size: 13px;
  cursor: pointer;
  user-select: text;
  -webkit-user-select: text;
  -moz-user-select: text;
  -ms-user-select: text;
}

.diff-line:hover {
  background-color: rgba(0, 0, 0, 0.05);
}

.diff-line.added {
  background-color: #e6ffed;
}

.diff-line.removed {
  background-color: #ffeef0;
}

.diff-line.modified {
  background-color: #fff3cd;
}

.diff-line.selected {
  background-color: #e3f2fd;
}

.diff-line.current-match {
  background-color: #ffeb3b;
}

.diff-line.jump-highlight {
  animation: jump-highlight 2s ease-in-out;
}

@keyframes jump-highlight {
  0%, 100% { background-color: transparent; }
  50% { background-color: #ffeb3b; }
}

.line-number {
  width: 60px;
  text-align: right;
  padding-right: 10px;
  color: #999;
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

.blame-info {
  width: 200px;
  padding: 0 10px;
  color: #666;
  font-size: 11px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  flex-shrink: 0;
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

.line-content {
  flex: 1;
  padding: 0 10px;
  overflow: visible;
}
</style>
