<template>
  <div class="diff-viewer">
    <div v-if="currentFile" class="file-info-bar">
      <div class="file-info">
        <span class="file-label">旧版本</span>
      </div>
      <div class="file-info">
        <span class="file-label">新版本</span>
      </div>
    </div>

    <div v-if="currentFile" class="diff-content">
      <!-- Inline Search -->
      <InlineSearch
        v-if="!isBinary"
        ref="inlineSearch"
        :content="combinedContent"
        @close="handleSearchClose"
        @highlight-match="handleHighlightMatch"
      />
      
      <div v-if="isBinary" class="binary-placeholder">
        [二进制文件]
      </div>
      <template v-else>
        <div class="diff-pane">
          <div class="pane-header">
            <span class="pane-title">{{ isFileViewMode ? '文件内容' : 'HEAD' }}</span>
          </div>
          <div class="code-content" ref="leftCodeContent" @scroll="syncScroll('left')">
            <ShikiDiffLines 
              :lines="isFileViewMode ? leftLines : rightLines" 
              :filename="currentFile?.path || ''"
              :theme="theme || 'light'"
              :search-matches="leftSearchMatches"
              :current-match-index="currentLeftMatchIndex"
              :highlighted-line="highlightedLine"
            />
          </div>
        </div>
        <div class="diff-divider"></div>
        <div class="diff-pane">
          <div class="pane-header">
            <span class="pane-title">{{ isFileViewMode ? '文件内容' : (viewMode === 'working' ? '工作区' : '暂存区') }}</span>
          </div>
          <div class="code-content" ref="rightCodeContent" @scroll="syncScroll('right')">
            <ShikiDiffLines 
              :lines="isFileViewMode ? rightLines : leftLines" 
              :filename="currentFile?.path || ''"
              :theme="theme || 'light'"
              :search-matches="rightSearchMatches"
              :current-match-index="currentRightMatchIndex"
              :highlighted-line="highlightedLine"
            />
          </div>
        </div>
        <!-- Minimap - 合并显示左右两侧的更改 -->
        <Minimap
          :left-lines="leftLines"
          :right-lines="rightLines"
          :scroll-top="leftScrollTop"
          :container-height="codeContainerHeight"
          :content-height="codeContentHeight"
          @jump="handleMinimapJump"
        />
      </template>
    </div>

    <div v-else class="empty-state">
      选择左侧文件查看差异
    </div>

    <div v-if="currentFile && diffStats" class="diff-stats">
      <span class="stat added">+{{ diffStats.added }}</span>
      <span class="stat removed">-{{ diffStats.removed }}</span>
      <span class="stat changed">~{{ diffStats.changed }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick, computed, onMounted, onUnmounted } from 'vue';
import ShikiDiffLines from './ShikiDiffLines.vue';
import Minimap from './Minimap.vue';
import InlineSearch from './InlineSearch.vue';

interface SearchMatch {
  lineIndex: number;
  columnIndex: number;
  text: string;
}

interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  status?: string;
  children: FileNode[];
  expanded?: boolean;
}

interface DiffLine {
  lineNum: number;
  content: string;
  changeType: string;
  isDiff: boolean;
}

interface DiffStats {
  added: number;
  removed: number;
  changed: number;
}

const props = defineProps<{
  currentFile: FileNode | null;
  leftLines: DiffLine[];
  rightLines: DiffLine[];
  isBinary: boolean;
  diffStats: DiffStats | null;
  viewMode: 'working' | 'staged';
  theme?: 'light' | 'dark';
}>();

// 判断是否是文件查看模式（通过检查文件状态是否为空）
const isFileViewMode = computed(() => {
  return props.currentFile && !props.currentFile.status;
});

const emit = defineEmits<{
  'scroll': [scrollTop: number];
}>();

// 代码内容区域 refs，用于同步滚动
const leftCodeContent = ref<HTMLElement | null>(null);
const rightCodeContent = ref<HTMLElement | null>(null);
const inlineSearch = ref<InstanceType<typeof InlineSearch> | null>(null);
let isSyncing = false;

// Minimap 相关
const leftScrollTop = ref(0);
const codeContainerHeight = ref(0);
const codeContentHeight = ref(0);

// 搜索相关
const searchMatches = ref<SearchMatch[]>([]);
const currentSearchMatch = ref<SearchMatch | null>(null);
const leftSearchMatches = ref<SearchMatch[]>([]);
const rightSearchMatches = ref<SearchMatch[]>([]);
const currentLeftMatchIndex = ref(-1);
const currentRightMatchIndex = ref(-1);
const highlightedLine = ref<number | null>(null); // 当前高亮的行号

// 合并左右两侧内容用于搜索
const combinedContent = computed(() => {
  const leftContent = props.leftLines.map(line => line.content).join('\n');
  const rightContent = props.rightLines.map(line => line.content).join('\n');
  return `${leftContent}\n--- 分隔线 ---\n${rightContent}`;
});

// 搜索相关函数
const handleSearchClose = () => {
  searchMatches.value = [];
  currentSearchMatch.value = null;
};

const handleHighlightMatch = (matches: SearchMatch[], currentMatch: SearchMatch) => {
  searchMatches.value = matches;
  currentSearchMatch.value = currentMatch;
  
  // 将匹配项分配到左右两侧
  // 假设分隔线之前的行属于右侧（工作区），之后的行属于左侧（HEAD）
  const separatorLineIndex = props.rightLines.length;
  
  leftSearchMatches.value = [];
  rightSearchMatches.value = [];
  
  for (let i = 0; i < matches.length; i++) {
    const match = matches[i];
    if (match.lineIndex < separatorLineIndex) {
      // 右侧（工作区）
      rightSearchMatches.value.push({
        lineIndex: match.lineIndex,
        columnIndex: match.columnIndex,
        text: match.text
      });
    } else {
      // 左侧（HEAD）
      leftSearchMatches.value.push({
        lineIndex: match.lineIndex - separatorLineIndex,
        columnIndex: match.columnIndex,
        text: match.text
      });
    }
  }
  
  // 计算当前匹配项在左右两侧的索引
  if (currentMatch) {
    if (currentMatch.lineIndex < separatorLineIndex) {
      currentRightMatchIndex.value = rightSearchMatches.value.findIndex(
        m => m.lineIndex === currentMatch.lineIndex && m.columnIndex === currentMatch.columnIndex
      );
      currentLeftMatchIndex.value = -1;
    } else {
      currentLeftMatchIndex.value = leftSearchMatches.value.findIndex(
        m => m.lineIndex === currentMatch.lineIndex - separatorLineIndex && m.columnIndex === currentMatch.columnIndex
      );
      currentRightMatchIndex.value = -1;
    }
    
    // 跳转到匹配项
    scrollToMatch(currentMatch);
  } else {
    currentLeftMatchIndex.value = -1;
    currentRightMatchIndex.value = -1;
  }
};

// 滚动到匹配项
const scrollToMatch = (match: SearchMatch) => {
  const separatorLineIndex = props.rightLines.length;
  const targetEl = match.lineIndex < separatorLineIndex ? rightCodeContent.value : leftCodeContent.value;
  const lineIndex = match.lineIndex < separatorLineIndex ? match.lineIndex : match.lineIndex - separatorLineIndex;
  
  if (targetEl) {
    const lineHeight = 24; // 假设每行高度为 24px
    const targetScrollTop = lineIndex * lineHeight - 100; // 减去 100px 让匹配项显示在可见区域上方
    
    targetEl.scrollTop = Math.max(0, targetScrollTop);
    
    // 更新 minimap 状态
    leftScrollTop.value = targetEl.scrollTop;
  }
};

// 快捷键监听
const handleKeyDown = (event: KeyboardEvent) => {
  // Ctrl/Cmd + F 打开搜索
  if ((event.ctrlKey || event.metaKey) && event.key === 'f') {
    event.preventDefault();
    inlineSearch.value?.openSearch();
  }
};

// 同步滚动函数
const syncScroll = (source: 'left' | 'right') => {
  if (isSyncing) return;
  isSyncing = true;

  const sourceEl = source === 'left' ? leftCodeContent.value : rightCodeContent.value;
  const targetEl = source === 'left' ? rightCodeContent.value : leftCodeContent.value;

  if (sourceEl && targetEl) {
    // 同步纵向滚动
    targetEl.scrollTop = sourceEl.scrollTop;
    // 同步横向滚动
    targetEl.scrollLeft = sourceEl.scrollLeft;
  }

  // 更新 minimap 的 scrollTop
  if (sourceEl) {
    leftScrollTop.value = sourceEl.scrollTop;
    codeContainerHeight.value = sourceEl.clientHeight;
    codeContentHeight.value = sourceEl.scrollHeight;
    emit('scroll', sourceEl.scrollTop);
  }

  // 使用 requestAnimationFrame 确保同步
  requestAnimationFrame(() => {
    isSyncing = false;
  });
};

// Minimap 跳转处理
const handleMinimapJump = (scrollTop: number) => {
  if (!leftCodeContent.value) return;

  leftCodeContent.value.scrollTop = scrollTop;
  if (rightCodeContent.value) {
    rightCodeContent.value.scrollTop = scrollTop;
  }

  // 更新 minimap 状态
  leftScrollTop.value = scrollTop;
};

// 生命周期钩子
onMounted(() => {
  window.addEventListener('keydown', handleKeyDown);
  // 监听跳转行的事件
  window.addEventListener('jump-to-line', handleJumpToLine);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleKeyDown);
  window.removeEventListener('jump-to-line', handleJumpToLine);
});

// 处理跳转到指定行
const handleJumpToLine = (event: Event) => {
  const customEvent = event as CustomEvent;
  const lineNumber = customEvent.detail as number;
  
  console.log('跳转到行:', lineNumber);
  
  if (leftCodeContent.value && lineNumber) {
    const lineHeight = 24; // 假设每行高度为 24px
    const targetScrollTop = (lineNumber - 1) * lineHeight - 100;
    leftCodeContent.value.scrollTop = Math.max(0, targetScrollTop);
    
    // 同步右侧滚动
    if (rightCodeContent.value) {
      rightCodeContent.value.scrollTop = leftCodeContent.value.scrollTop;
    }
    
    // 更新 minimap 状态
    leftScrollTop.value = leftCodeContent.value.scrollTop;
    
    // 高亮目标行
    highlightedLine.value = lineNumber;
    console.log('高亮行:', highlightedLine.value);
    
    // 3 秒后移除高亮
    setTimeout(() => {
      highlightedLine.value = null;
      console.log('移除高亮');
    }, 3000);
  }
};

// 监听文件变化，重新搜索
watch(() => props.currentFile, () => {
  // 切换文件时，如果有搜索内容，重新执行搜索
  // InlineSearch 会自动根据新的 content 重新搜索
});

// 监听线条变化，更新 minimap 尺寸
watch(() => props.leftLines, () => {
  nextTick(() => {
    if (leftCodeContent.value) {
      codeContainerHeight.value = leftCodeContent.value.clientHeight;
      codeContentHeight.value = leftCodeContent.value.scrollHeight;
    }
  });
}, { deep: true });

// 暴露方法给父组件
defineExpose({
  leftCodeContent,
  rightCodeContent,
  syncScroll
});
</script>

<style scoped>
.diff-viewer {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--bg-primary);
}

.file-info-bar {
  display: flex;
  padding: 8px 16px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  gap: 20px;
}

.file-info {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-label {
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  font-weight: 600;
}

.file-path {
  font-size: 13px;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.diff-content {
  flex: 1;
  display: flex;
  overflow: hidden;
  position: relative;
}

.binary-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.diff-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.pane-header {
  padding: 6px 12px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  font-weight: 600;
}

.code-content {
  flex: 1;
  overflow: auto;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  line-height: 24px;
}

.diff-divider {
  width: 1px;
  background-color: var(--border-color);
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.diff-stats {
  display: flex;
  gap: 16px;
  padding: 8px 16px;
  background-color: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.stat {
  font-weight: 600;
}

.stat.added {
  color: #4caf50;
}

.stat.removed {
  color: #f44336;
}

.stat.changed {
  color: #2196f3;
}
</style>
