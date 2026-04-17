<template>
  <div class="shiki-diff-lines">
    <div
      v-for="(line, index) in lines"
      :key="index"
      class="diff-line"
      :class="[line.changeType, { 'current-match': isCurrentMatch(index) }]"
      :data-line="line.lineNum"
    >
      <span class="line-number">{{ line.lineNum > 0 ? line.lineNum : '' }}</span>
      <span class="line-content" v-html="getLineContent(index)"></span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { highlightCode } from '../utils/shikiHighlighter';

const props = defineProps<{
  lines: {
    lineNum: number;
    content: string;
    changeType: string;
    isDiff: boolean;
  }[];
  filename: string;
  theme?: 'light' | 'dark';
  searchMatches?: SearchMatch[];
  currentMatchIndex?: number;
}>();

interface SearchMatch {
  lineIndex: number;
  columnIndex: number;
  text: string;
}

const highlightedLines = ref<string[]>([]);

const escapeHtml = (text: string): string => {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
};

// 检查是否是当前匹配项
const isCurrentMatch = (index: number): boolean => {
  if (!props.searchMatches || props.currentMatchIndex === undefined || props.currentMatchIndex < 0) {
    return false;
  }
  
  const currentMatch = props.searchMatches[props.currentMatchIndex];
  return currentMatch && currentMatch.lineIndex === index;
};

// 获取行内容并高亮搜索匹配项
const getLineContent = (index: number): string => {
  const originalContent = props.lines[index]?.content || ' ';
  
  if (!props.searchMatches) {
    // 没有搜索匹配，直接使用 Shiki 高亮
    return highlightedLines.value[index] || escapeHtml(originalContent);
  }
  
  // 查找这一行的所有匹配项
  const lineMatches = props.searchMatches.filter(m => m.lineIndex === index);
  
  if (lineMatches.length === 0) {
    // 这一行没有匹配，使用 Shiki 高亮
    return highlightedLines.value[index] || escapeHtml(originalContent);
  }
  
  // 有搜索匹配，先转义 HTML 特殊字符，然后高亮搜索词
  let result = escapeHtml(originalContent);
  
  // 按列位置排序，从后往前替换，避免影响后续位置
  lineMatches.sort((a, b) => b.columnIndex - a.columnIndex);
  
  for (const match of lineMatches) {
    const escapedMatch = escapeHtml(match.text);
    // 创建一个简单的正则来匹配文本
    const regex = new RegExp(`(${escapedMatch})`, 'g');
    result = result.replace(regex, '<span class="search-match">$1</span>');
  }
  
  // 如果是当前匹配项，添加额外的高亮
  if (isCurrentMatch(index)) {
    result = `<span class="current-search-match">${result}</span>`;
  }
  
  return result;
};

const highlightAllLines = async () => {
  if (props.lines.length === 0) return;

  try {
    // 获取所有行的内容
    const fullCode = props.lines.map(line => line.content).join('\n');
    
    // 使用 Shiki 高亮整个代码
    const highlightedHtml = await highlightCode(
      fullCode,
      props.filename,
      props.theme || 'light'
    );

    // 解析高亮后的 HTML，提取每行的内容
    const parser = new DOMParser();
    const doc = parser.parseFromString(highlightedHtml, 'text/html');
    const codeElement = doc.querySelector('code');
    
    if (codeElement) {
      // 获取高亮后的代码内容，按行分割
      const lines = codeElement.innerHTML.split('\n');
      highlightedLines.value = lines.map(line => line || ' ');
    } else {
      // 如果解析失败，使用原始内容
      highlightedLines.value = props.lines.map(line => escapeHtml(line.content || ' '));
    }
  } catch (e) {
    console.error('Failed to highlight lines:', e);
    highlightedLines.value = props.lines.map(line => escapeHtml(line.content || ' '));
  }
};

onMounted(() => {
  highlightAllLines();
});

watch(() => [props.lines, props.filename, props.theme], () => {
  highlightAllLines();
}, { deep: true });
</script>

<style scoped>
.shiki-diff-lines {
  display: flex;
  flex-direction: column;
  min-width: fit-content;
}

.diff-line {
  display: flex;
  height: 24px;
  min-height: 24px;
  line-height: 24px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  box-sizing: border-box;
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

/* 搜索匹配高亮 */
.diff-line.current-match {
  background-color: rgba(255, 235, 59, 0.3) !important;
}

.search-match {
  background-color: rgba(255, 235, 59, 0.5);
  border-radius: 2px;
  padding: 1px 2px;
}

.current-search-match {
  background-color: rgba(255, 193, 7, 0.6);
  border-radius: 2px;
  padding: 1px 2px;
  box-shadow: 0 0 4px rgba(255, 193, 7, 0.5);
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
