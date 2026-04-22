<template>
  <div class="virtual-diff-lines" ref="containerRef">
    <!-- 占位元素，保持滚动高度 -->
    <div class="scroll-spacer" :style="{ height: totalHeight + 'px' }"></div>
    <!-- 可见区域的行 - 使用固定节点池 -->
    <div class="visible-lines" :style="{ transform: `translateY(${offsetY}px)` }">
      <div
        v-for="i in poolSize"
        :key="`pool-${i}`"
        :ref="el => setLineRef(el as HTMLElement, i - 1)"
        class="diff-line"
        style="display: none;"
      >
        <span class="line-number"></span>
        <span class="blame-info" style="display: none;"></span>
        <span class="line-content"></span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onUnmounted, computed, nextTick } from 'vue';
import { initHighlighter, getLanguageFromFilename } from '../utils/shikiHighlighter';
import type { HighlighterCore } from 'shiki/core';

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

// 虚拟滚动配置
const LINE_HEIGHT = 24;
const BUFFER_LINES = 20;
const poolSize = computed(() => {
  return Math.ceil(containerHeight.value / LINE_HEIGHT) + BUFFER_LINES * 2;
});

// 大文件阈值
const LARGE_FILE_THRESHOLD = 2000;
const isLargeFile = computed(() => props.lines.length > LARGE_FILE_THRESHOLD);

// 语法高亮缓存
const highlightCache = ref<Map<number, string>>(new Map());
const isHighlighting = ref(false);

// 计算总高度
const totalHeight = computed(() => {
  return props.lines.length * LINE_HEIGHT;
});

// 计算可见区域的起始索引
const startIndex = computed(() => {
  const start = Math.floor(scrollTop.value / LINE_HEIGHT) - BUFFER_LINES;
  return Math.max(0, start);
});

// 偏移量
const offsetY = computed(() => {
  return startIndex.value * LINE_HEIGHT;
});

// DOM 节点池
const lineRefs = ref<(HTMLElement | null)[]>([]);
const setLineRef = (el: HTMLElement | null, index: number) => {
  if (el) {
    lineRefs.value[index] = el;
  }
};

const selectedLineIndex = ref<number | null>(null);

const emit = defineEmits<{
  'line-click': [lineIndex: number, lineNum: number];
  'scroll': [scrollTop: number];
}>();

// 转义 HTML
const escapeHtml = (text: string): string => {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
};

// 简单的语法高亮（降级模式）
const simpleHighlight = (content: string): string => {
  if (!content) return '';
  let html = escapeHtml(content);
  html = html.replace(/(".*?")/g, '<span style="color: #0d7377;">$1</span>');
  html = html.replace(/\b(\d+)\b/g, '<span style="color: #098658;">$1</span>');
  html = html.replace(/(\/\/.*$)/gm, '<span style="color: #6a9955;">$1</span>');
  return html;
};

// Shiki 语法高亮（异步）
let highlighter: HighlighterCore | null = null;
const initShiki = async () => {
  if (!highlighter) {
    highlighter = await initHighlighter();
  }
};

// 增量语法高亮 - 优先处理可视区域
const highlightVisibleLines = async () => {
  if (isLargeFile.value) {
    // 大文件使用简单高亮，更新显示
    updateVisibleLines();
    return;
  }

  if (isHighlighting.value) return;
  isHighlighting.value = true;

  try {
    await initShiki();
    const lang = getLanguageFromFilename(props.filename);
    const themeName = props.theme === 'dark' ? 'github-dark' : 'github-light';

    const start = startIndex.value;
    const end = Math.min(start + poolSize.value, props.lines.length);

    // 优先处理可视区域
    for (let i = start; i < end; i++) {
      if (highlightCache.value.has(i)) continue;

      const line = props.lines[i];
      if (!line || !line.content) {
        highlightCache.value.set(i, '');
        continue;
      }

      try {
        const result = await highlighter!.codeToTokens(line.content, {
          lang,
          theme: themeName,
        });

        // codeToTokens 返回 { tokens: ThemedToken[][] }
        // tokens[0] 是第一行的 token 数组
        const lineTokens = result.tokens[0] || [];
        let html = '';
        for (const token of lineTokens) {
          const color = token.color;
          const content = escapeHtml(token.content);
          if (color) {
            html += `<span style="color: ${color};">${content}</span>`;
          } else {
            html += content;
          }
        }
        highlightCache.value.set(i, html);
      } catch (e) {
        highlightCache.value.set(i, simpleHighlight(line.content));
      }
    }

    // 更新显示
    updateVisibleLines();

    // 后台处理其他区域
    if (typeof requestIdleCallback !== 'undefined') {
      requestIdleCallback(() => {
        highlightBackgroundLines(start, end);
      });
    } else {
      setTimeout(() => highlightBackgroundLines(start, end), 100);
    }
  } catch (e) {
    console.error('Highlight error:', e);
  } finally {
    isHighlighting.value = false;
  }
};

// 后台高亮其他行
const highlightBackgroundLines = async (visibleStart: number, visibleEnd: number) => {
  if (!highlighter || isLargeFile.value) return;

  const lang = getLanguageFromFilename(props.filename);
  const themeName = props.theme === 'dark' ? 'github-dark' : 'github-light';

  // 分块处理，避免阻塞
  const chunkSize = 50;
  for (let chunkStart = 0; chunkStart < props.lines.length; chunkStart += chunkSize) {
    // 跳过已处理和可视区域
    if (chunkStart >= visibleStart && chunkStart < visibleEnd) continue;

    const chunkEnd = Math.min(chunkStart + chunkSize, props.lines.length);
    for (let i = chunkStart; i < chunkEnd; i++) {
      if (highlightCache.value.has(i)) continue;

      const line = props.lines[i];
      if (!line || !line.content) {
        highlightCache.value.set(i, '');
        continue;
      }

      try {
        const result = await highlighter.codeToTokens(line.content, {
          lang,
          theme: themeName,
        });

        const lineTokens = result.tokens[0] || [];
        let html = '';
        for (const token of lineTokens) {
          const color = token.color;
          const content = escapeHtml(token.content);
          if (color) {
            html += `<span style="color: ${color};">${content}</span>`;
          } else {
            html += content;
          }
        }
        highlightCache.value.set(i, html);
      } catch (e) {
        highlightCache.value.set(i, simpleHighlight(line.content));
      }
    }

    // 每处理一块，让出主线程
    await new Promise(resolve => setTimeout(resolve, 0));
  }
};

// 更新可见行的显示
const updateVisibleLines = () => {
  const start = startIndex.value;
  const visibleCount = poolSize.value;

  for (let i = 0; i < visibleCount; i++) {
    const lineEl = lineRefs.value[i];
    if (!lineEl) continue;

    const globalIndex = start + i;
    const line = props.lines[globalIndex];

    if (!line) {
      lineEl.style.display = 'none';
      continue;
    }

    lineEl.style.display = 'flex';

    // 更新 class
    lineEl.className = 'diff-line';
    if (line.changeType) lineEl.classList.add(line.changeType);
    if (isCurrentMatch(globalIndex)) lineEl.classList.add('current-match');
    if (isJumpHighlight(globalIndex)) lineEl.classList.add('jump-highlight');
    if (selectedLineIndex.value === globalIndex) lineEl.classList.add('selected');
    if (showBlameInfo.value) lineEl.classList.add('has-blame');

    // 更新行号
    const lineNumEl = lineEl.querySelector('.line-number') as HTMLElement;
    if (lineNumEl) {
      lineNumEl.textContent = line.lineNum > 0 ? String(line.lineNum) : '';
    }

    // 更新 blame 信息
    const blameEl = lineEl.querySelector('.blame-info') as HTMLElement;
    if (blameEl) {
      if (showBlameInfo.value) {
        blameEl.style.display = 'block';
        blameEl.textContent = getBlameText(globalIndex);
        blameEl.title = getBlameTooltip(globalIndex);
      } else {
        blameEl.style.display = 'none';
      }
    }

    // 更新内容
    const contentEl = lineEl.querySelector('.line-content') as HTMLElement;
    if (contentEl) {
      // 优先使用缓存的高亮内容
      const cached = highlightCache.value.get(globalIndex);
      if (cached !== undefined) {
        contentEl.innerHTML = cached;
      } else if (isLargeFile.value) {
        // 大文件使用简单高亮
        contentEl.innerHTML = simpleHighlight(line.content);
      } else {
        // 尚未高亮，先显示纯文本
        contentEl.textContent = line.content;
      }
    }

    // 更新 data-line 属性
    lineEl.setAttribute('data-line', String(line.lineNum));
    lineEl.onclick = () => onLineClick(globalIndex);
  }
};

const onLineClick = (globalIndex: number) => {
  selectedLineIndex.value = globalIndex;
  const line = props.lines[globalIndex];
  if (line) {
    emit('line-click', globalIndex, line.lineNum);
  }
};

// 处理滚动事件 - 直接同步更新，避免延迟
const handleScroll = () => {
  if (containerRef.value) {
    scrollTop.value = containerRef.value.scrollTop;
    // 直接更新可见行，不使用 RAF 节流
    updateVisibleLines();
    // 通知父组件滚动位置变化（用于 minimap 同步）
    emit('scroll', scrollTop.value);
  }
};

// 外部调用同步滚动
let isSyncingScroll = false;
const syncScrollTo = (targetScrollTop: number) => {
  if (containerRef.value && !isSyncingScroll) {
    isSyncingScroll = true;
    containerRef.value.scrollTop = targetScrollTop;
    // 同步更新 scrollTop 值
    scrollTop.value = targetScrollTop;
    updateVisibleLines();
    // 使用 setTimeout 重置标志，避免阻塞后续同步
    setTimeout(() => {
      isSyncingScroll = false;
    }, 0);
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
    containerRef.value.addEventListener('scroll', handleScroll, { passive: true });
    window.addEventListener('resize', updateContainerHeight);
  }
  // 初始渲染
  nextTick(() => {
    updateVisibleLines();
    highlightVisibleLines();
  });
});

onUnmounted(() => {
  if (containerRef.value) {
    containerRef.value.removeEventListener('scroll', handleScroll);
    window.removeEventListener('resize', updateContainerHeight);
  }
});

// 监听数据变化
watch(() => props.lines, () => {
  highlightCache.value.clear();
  nextTick(() => {
    updateVisibleLines();
    highlightVisibleLines();
  });
}, { deep: true });

watch(() => [props.theme, props.filename], () => {
  highlightCache.value.clear();
  highlightVisibleLines();
});

watch(() => props.highlightedLine, () => {
  updateVisibleLines();
});

watch(() => props.searchMatches, () => {
  updateVisibleLines();
}, { deep: true });

watch(() => props.currentMatchIndex, () => {
  updateVisibleLines();
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

// 暴露方法给父组件
defineExpose({
  syncScrollTo,
  containerRef
});
</script>

<style scoped>
.virtual-diff-lines {
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
  background-color: #e6ffed !important;
}

.diff-line.removed {
  background-color: #ffeef0 !important;
}

.diff-line.modified {
  background-color: #fff3cd !important;
}

.diff-line.empty {
  background-color: transparent;
}

.diff-line.unchanged {
  background-color: transparent;
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
