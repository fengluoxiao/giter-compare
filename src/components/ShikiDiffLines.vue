<template>
  <div class="shiki-diff-lines">
    <div
      v-for="(line, index) in lines"
      :key="index"
      class="diff-line"
      :class="line.changeType"
      :data-line="line.lineNum"
    >
      <span class="line-number">{{ line.lineNum > 0 ? line.lineNum : '' }}</span>
      <span class="line-content" v-html="highlightedLines[index] || escapeHtml(line.content || ' ')"></span>
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
}>();

const highlightedLines = ref<string[]>([]);

const escapeHtml = (text: string): string => {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
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
