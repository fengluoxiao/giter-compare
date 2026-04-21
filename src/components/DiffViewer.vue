<template>
  <div class="diff-viewer">
    <div v-if="currentFile" class="file-info-bar">
      <div class="file-info">
        <span class="file-label">旧版本</span>
        <select
          v-if="commitList && commitList.length > 0"
          :value="oldVersion"
          class="version-select"
          @change="$emit('change-old-version', ($event.target as HTMLSelectElement).value)"
        >
          <option v-for="commit in commitList" :key="commit.hash" :value="commit.hash">
            {{ commit.short_hash }} - {{ commit.message }}
          </option>
        </select>
        <span v-else class="version-text">{{ oldVersion || 'HEAD' }}</span>
      </div>
      <div class="file-info">
        <span class="file-label">新版本</span>
        <select
          v-if="commitList && commitList.length > 0"
          :value="newVersion"
          class="version-select"
          @change="$emit('change-new-version', ($event.target as HTMLSelectElement).value)"
        >
          <option value="WORKING">工作区</option>
          <option v-for="commit in availableNewVersions" :key="commit.hash" :value="commit.hash">
            {{ commit.short_hash }} - {{ commit.message }}
          </option>
        </select>
        <span v-else class="version-text">{{ newVersion === 'WORKING' ? '工作区' : (newVersion || '工作区') }}</span>
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
              :lines="rightLines" 
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
              :lines="leftLines" 
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

interface CommitInfo {
  hash: string;
  short_hash: string;
  message: string;
}

const props = defineProps<{
  currentFile: FileNode | null;
  leftLines: DiffLine[];
  rightLines: DiffLine[];
  isBinary: boolean;
  diffStats: DiffStats | null;
  viewMode: 'working' | 'staged';
  theme?: 'light' | 'dark';
  oldVersion?: string;
  newVersion?: string;
  commitList?: CommitInfo[];
}>();

// 判断是否是文件查看模式（通过检查文件状态是否为空）
const isFileViewMode = computed(() => {
  return props.currentFile && !props.currentFile.status;
});

const emit = defineEmits<{
  'scroll': [scrollTop: number];
  'change-old-version': [version: string];
  'change-new-version': [version: string];
}>();

// 根据旧版本选择，计算可用的新版本列表
const availableNewVersions = computed(() => {
  if (!props.commitList || !props.oldVersion) {
    return props.commitList || [];
  }
  const oldIndex = props.commitList.findIndex(c => c.hash === props.oldVersion);
  if (oldIndex === -1) {
    return props.commitList;
  }
  return props.commitList.slice(0, oldIndex);
});

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
  const { lineNumber, searchText } = customEvent.detail as { lineNumber: number; searchText?: string };
  
  console.log('跳转到行:', lineNumber, '搜索词:', searchText);
  
  if (leftCodeContent.value && lineNumber) {
    const lineHeight = 24; // 假设每行高度为 24px
    const containerHeight = leftCodeContent.value.clientHeight;
    
    // 在差异对比视图中搜索匹配的文本
    let targetLineIndex = lineNumber - 1; // 默认使用原始行号
    let foundMatch = false;
    
    if (searchText && props.leftLines.length > 0) {
      // 在 leftLines（新版本）中搜索包含 searchText 的行
      const searchRegex = new RegExp(searchText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'), 'i');
      
      // 从原始行号附近开始搜索，找到最近的匹配（±50 行范围）
      for (let offset = 0; offset < 50; offset++) {
        // 检查下方行
        if (lineNumber - 1 + offset < props.leftLines.length) {
          const line = props.leftLines[lineNumber - 1 + offset];
          if (line && searchRegex.test(line.content)) {
            targetLineIndex = lineNumber - 1 + offset;
            foundMatch = true;
            console.log('在新版本找到匹配行:', targetLineIndex + 1);
            break;
          }
        }
        // 检查上方行
        if (lineNumber - 1 - offset >= 0 && offset > 0) {
          const line = props.leftLines[lineNumber - 1 - offset];
          if (line && searchRegex.test(line.content)) {
            targetLineIndex = lineNumber - 1 - offset;
            foundMatch = true;
            console.log('在新版本找到匹配行:', targetLineIndex + 1);
            break;
          }
        }
      }
      
      // 如果新版本没找到，尝试在 rightLines（旧版本）中搜索
      if (!foundMatch && props.rightLines.length > 0) {
        for (let offset = 0; offset < 50; offset++) {
          if (lineNumber - 1 + offset < props.rightLines.length) {
            const line = props.rightLines[lineNumber - 1 + offset];
            if (line && searchRegex.test(line.content)) {
              targetLineIndex = lineNumber - 1 + offset;
              foundMatch = true;
              console.log('在旧版本找到匹配行:', targetLineIndex + 1);
              break;
            }
          }
          if (lineNumber - 1 - offset >= 0 && offset > 0) {
            const line = props.rightLines[lineNumber - 1 - offset];
            if (line && searchRegex.test(line.content)) {
              targetLineIndex = lineNumber - 1 - offset;
              foundMatch = true;
              console.log('在旧版本找到匹配行:', targetLineIndex + 1);
              break;
            }
          }
        }
      }
    }
    
    // 计算滚动位置
    const targetScrollTop = targetLineIndex * lineHeight - (containerHeight / 2) + (lineHeight / 2);
    leftCodeContent.value.scrollTop = Math.max(0, targetScrollTop);
    
    // 同步右侧滚动
    if (rightCodeContent.value) {
      rightCodeContent.value.scrollTop = leftCodeContent.value.scrollTop;
    }
    
    // 更新 minimap 状态
    leftScrollTop.value = leftCodeContent.value.scrollTop;
    
    // 高亮目标行（使用找到的实际行号）
    highlightedLine.value = targetLineIndex + 1;
    console.log('高亮行:', highlightedLine.value, '滚动位置:', leftScrollTop.value);
    
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
  flex-shrink: 0;
}

.version-select {
  flex: 1;
  min-width: 0;
  padding: 5px 28px 5px 10px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
  background-color: var(--bg-primary);
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%236b7280' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  background-size: 14px;
  appearance: none;
  -webkit-appearance: none;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.04);
}

.version-select:hover {
  border-color: var(--accent-color);
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.06), 0 0 0 3px rgba(74, 126, 255, 0.08);
}

.version-select:focus {
  outline: none;
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px rgba(74, 126, 255, 0.15);
}

.version-select option {
  padding: 8px;
  font-size: 12px;
}

.version-text {
  flex: 1;
  min-width: 0;
  font-size: 12px;
  color: var(--text-primary);
  padding: 4px 8px;
  background-color: var(--bg-primary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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
  user-select: text !important;
  -webkit-user-select: text !important;
  -moz-user-select: text !important;
  -ms-user-select: text !important;
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
