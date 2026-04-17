<template>
  <div v-if="visible" class="global-search-overlay" @click.self="close">
    <div class="global-search-dialog" :class="{ 'with-preview': showPreview }">
      <div class="search-header">
        <h3>🔍 全局搜索</h3>
        <button class="close-btn" @click="close">
          <svg viewBox="0 0 24 24" width="20" height="20" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>

      <div class="search-form">
        <div class="search-input-group">
          <input
            ref="searchInput"
            v-model="searchQuery"
            type="text"
            class="search-input"
            placeholder="搜索内容..."
            @keydown.enter="performSearch"
          />
          <button 
            class="mode-btn" 
            :class="{ active: useRegex }"
            @click="toggleRegex" 
            :title="useRegex ? '正则表达式模式' : '通配符模式'"
          >
            <span class="mode-icon">.*</span>
          </button>
          <button class="search-btn" @click="performSearch" :disabled="isSearching">
            {{ isSearching ? '搜索中...' : '搜索' }}
          </button>
        </div>

        <div class="search-options">
          <label class="option-label">
            <input v-model="filePattern" type="text" class="file-pattern-input" placeholder="文件过滤（可选）" />
          </label>
          <span class="search-path">路径：{{ repoPath }}</span>
        </div>
      </div>

      <div class="search-main-content">
        <!-- 搜索结果列表 -->
        <div class="search-results" v-if="searchResults.length > 0" :class="{ 'narrow': showPreview }">
          <div class="results-header">
            <span class="results-count">找到 {{ totalMatches }} 个匹配，{{ searchResults.length }} 个文件</span>
          </div>
          
          <div class="results-list">
            <div 
              v-for="(result, index) in searchResults" 
              :key="index"
              class="result-item"
              :class="{ active: selectedResultIndex === index }"
            >
              <div class="result-file">
                <span 
                  class="expand-icon" 
                  :class="{ expanded: expandedFiles[index] }"
                  @click="toggleExpand(index)"
                >▶</span>
                <span class="file-icon" @click="openFileInMain(result.file_path)">📄</span>
                <span class="file-path" @click="selectResult(index)">{{ getRelativePath(result.file_path) }}</span>
                <span class="match-count-badge" @click="selectResult(index)">{{ result.match_count }} 处匹配</span>
              </div>
              <div v-if="expandedFiles[index]" class="result-matches">
                <!-- 对匹配行进行去重，同一行号只显示一次 -->
                <div 
                  v-for="(match, matchIndex) in getUniqueMatches(result.matches).slice(0, 10)" 
                  :key="matchIndex"
                  class="match-line"
                  :class="{ active: selectedMatch?.fileIndex === index && selectedMatch?.matchIndex === matchIndex }"
                  @click.stop="selectMatch(index, matchIndex, match)"
                >
                  <span class="line-number">{{ match.line_number }}</span>
                  <span class="line-content" v-html="highlightMatch(getContextAroundMatch(match.line_content, match.matched_text), match.matched_text)"></span>
                </div>
                <div v-if="result.matches.length > 10" class="more-matches">
                  还有 {{ result.matches.length - 10 }} 个匹配，请打开文件查看
                </div>
              </div>
            </div>
          </div>
        </div>

        <div v-else-if="hasSearched && !isSearching" class="search-results no-results">
          <span class="no-results-icon">🔍</span>
          <span class="no-results-text">没有找到匹配项</span>
        </div>

        <!-- 代码预览区域 -->
        <div v-if="showPreview" class="code-preview-panel">
          <div class="preview-header">
            <span class="preview-title">{{ getRelativePath(previewFilePath) }}</span>
            <div class="preview-actions">
              <button class="preview-btn" @click="openFileInMain(previewFilePath, selectedMatch?.match?.line_number)">
                在主窗口打开
              </button>
              <button class="preview-btn close" @click="closePreview">✕</button>
            </div>
          </div>
          <div class="preview-content">
            <div class="diff-view">
              <div class="diff-pane">
                <div class="code-content" ref="oldCodeContent" @scroll="syncScroll('left')">
                  <div v-if="oldLines.length === 0" class="empty-content">无内容</div>
                  <div 
                    v-for="(line, idx) in oldLines" 
                    :key="idx"
                    class="code-line"
                    :class="{ 
                      highlight: isLineHighlighted(line.lineNum, 'old'),
                      'search-match': isSearchMatchLine(line.lineNum, 'old')
                    }"
                  >
                    <span class="line-num">{{ line.lineNum || '' }}</span>
                    <span class="line-text" :class="line.changeType">{{ line.content }}</span>
                  </div>
                </div>
              </div>
              <div class="diff-divider"></div>
              <div class="diff-pane">
                <div class="code-content" ref="newCodeContent" @scroll="syncScroll('right')">
                  <div v-if="newLines.length === 0" class="empty-content">无内容</div>
                  <div 
                    v-for="(line, idx) in newLines" 
                    :key="idx"
                    class="code-line"
                    :class="{ 
                      highlight: isLineHighlighted(line.lineNum, 'new'),
                      'search-match': isSearchMatchLine(line.lineNum, 'new')
                    }"
                  >
                    <span class="line-num">{{ line.lineNum || '' }}</span>
                    <span class="line-text" :class="line.changeType">{{ line.content }}</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface SearchMatch {
  line_number: number;
  column: number;
  line_content: string;
  matched_text: string;
}

interface SearchResult {
  file_path: string;
  matches: SearchMatch[];
  match_count: number;
}

interface DiffLine {
  lineNum: number;
  content: string;
  changeType: string;
  isDiff: boolean;
}

interface FileDiff {
  old_path: string;
  new_path: string;
  hunks: {
    old_start: number;
    old_lines: number;
    new_start: number;
    new_lines: number;
    lines: {
      line_number: number;
      content: string;
      change_type: string;
    }[];
  }[];
  old_content: string[];
  new_content: string[];
}

const emit = defineEmits<{
  close: [];
  'open-file': [path: string, lineNumber?: number, searchText?: string];
}>();

const visible = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');
const filePattern = ref('');
const useRegex = ref(false);
const isSearching = ref(false);
const searchResults = ref<SearchResult[]>([]);
const hasSearched = ref(false);
const repoPath = ref('');
const expandedFiles = ref<boolean[]>([]);
const selectedResultIndex = ref<number | null>(null);
const selectedMatch = ref<{ fileIndex: number; matchIndex: number; match: SearchMatch } | null>(null);

// 代码预览相关
const showPreview = ref(false);
const previewFilePath = ref('');
const oldLines = ref<DiffLine[]>([]);
const newLines = ref<DiffLine[]>([]);
const oldCodeContent = ref<HTMLElement | null>(null);
const newCodeContent = ref<HTMLElement | null>(null);
const highlightedLine = ref<{ lineNum: number; side: 'old' | 'new' } | null>(null);

const totalMatches = computed(() => {
  return searchResults.value.reduce((sum, result) => sum + result.match_count, 0);
});

// 切换展开/折叠
const toggleExpand = (index: number) => {
  expandedFiles.value[index] = !expandedFiles.value[index];
  // 强制触发响应式更新
  expandedFiles.value = [...expandedFiles.value];
};

// 选择搜索结果文件
const selectResult = async (index: number) => {
  selectedResultIndex.value = index;
  const result = searchResults.value[index];
  if (result) {
    // 先显示预览面板
    showPreview.value = true;
    // 自动展开 - 强制触发响应式更新
    expandedFiles.value[index] = true;
    expandedFiles.value = [...expandedFiles.value];
    // 加载文件差异
    await loadFileDiff(result.file_path);
  }
};

// 选择具体匹配行
const selectMatch = async (fileIndex: number, matchIndex: number, match: SearchMatch) => {
  selectedMatch.value = { fileIndex, matchIndex, match };
  const result = searchResults.value[fileIndex];
  if (result) {
    // 先显示预览面板
    showPreview.value = true;
    // 加载文件差异
    await loadFileDiff(result.file_path);
    // 在加载完差异后，使用搜索结果的行号直接高亮
    // 因为搜索结果是基于文件当前内容的行号，与 oldLines 的行号一致
    nextTick(() => {
      // 直接使用搜索结果的行号
      const lineNum = match.line_number;
      
      // 高亮对应的行（在左边旧版本中）
      highlightedLine.value = { lineNum, side: 'old' };
      // 滚动到对应位置（滚动左边旧版本）
      scrollToLine(lineNum, 'old');
    });
  }
};

// 加载文件差异
const loadFileDiff = async (filePath: string) => {
  previewFilePath.value = filePath;

  const relativePath = getRelativePath(filePath);

  try {
    // 并行获取旧版本（HEAD）和新版本（工作区）内容
    const [oldContentResult, newContentResult] = await Promise.allSettled([
      // 获取 HEAD 版本
      invoke<string>('get_file_content_at_revision', {
        repoPath: repoPath.value,
        filePath: relativePath,
        revision: 'HEAD'
      }).catch(() => ''), // 如果文件不存在于 HEAD，返回空字符串
      // 获取工作区版本
      invoke<string>('read_file_content', { filePath }).catch(() => '')
    ]);

    // 处理旧版本内容
    const oldContentStr = oldContentResult.status === 'fulfilled' ? oldContentResult.value : '';
    // 处理新版本内容
    const newContentStr = newContentResult.status === 'fulfilled' ? newContentResult.value : '';

    // 使用 compare_strings 获取差异对齐
    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent: oldContentStr,
      newContent: newContentStr
    });

    // 根据 hunks 对齐行号
    // 与主窗口保持一致：左边显示旧版本（rightLines），右边显示新版本（leftLines）
    const alignedLeftLines: DiffLine[] = [];  // 新版本（工作区）- 显示在右边
    const alignedRightLines: DiffLine[] = []; // 旧版本（HEAD）- 显示在左边

    let leftLineNum = 1;  // 新版本行号
    let rightLineNum = 1; // 旧版本行号

    diffResult.hunks.forEach(hunk => {
      // 添加上下文行 - hunk.old_start 和 hunk.new_start 是 1-based 行号
      const contextStart = Math.min(hunk.old_start, hunk.new_start) - 1;
      for (let i = alignedLeftLines.length; i < contextStart; i++) {
        const oldContent = diffResult.old_content[i] || '';
        const newContent = diffResult.new_content[i] || '';

        // 右边显示新版本
        alignedLeftLines.push({
          lineNum: leftLineNum++,
          content: newContent,
          changeType: 'unchanged',
          isDiff: false
        });
        // 左边显示旧版本
        alignedRightLines.push({
          lineNum: rightLineNum++,
          content: oldContent,
          changeType: 'unchanged',
          isDiff: false
        });
      }

      let pendingRemoved: { content: string; lineNum: number } | null = null;

      hunk.lines.forEach(line => {
        if (line.change_type === 'removed') {
          // 旧版本中删除的行
          pendingRemoved = { content: line.content, lineNum: rightLineNum++ };
        } else if (line.change_type === 'added') {
          if (pendingRemoved) {
            // 修改的行：右边（新版本）显示新内容，左边（旧版本）显示旧内容
            alignedLeftLines.push({
              lineNum: leftLineNum++,
              content: line.content,
              changeType: 'changed',
              isDiff: true
            });
            alignedRightLines.push({
              lineNum: pendingRemoved.lineNum,
              content: pendingRemoved.content,
              changeType: 'changed',
              isDiff: true
            });
            pendingRemoved = null;
          } else {
            // 新增的行：只在新版本中
            alignedLeftLines.push({
              lineNum: leftLineNum++,
              content: line.content,
              changeType: 'added',
              isDiff: true
            });
            alignedRightLines.push({
              lineNum: 0,
              content: '',
              changeType: 'empty',
              isDiff: false
            });
          }
        } else {
          // 未修改的行
          if (pendingRemoved) {
            // 处理挂起的删除行
            alignedLeftLines.push({
              lineNum: 0,
              content: '',
              changeType: 'empty',
              isDiff: false
            });
            alignedRightLines.push({
              lineNum: pendingRemoved.lineNum,
              content: pendingRemoved.content,
              changeType: 'removed',
              isDiff: true
            });
            pendingRemoved = null;
          }

          // 添加未修改的行
          alignedLeftLines.push({
            lineNum: leftLineNum++,
            content: line.content,
            changeType: 'unchanged',
            isDiff: false
          });
          alignedRightLines.push({
            lineNum: rightLineNum++,
            content: line.content,
            changeType: 'unchanged',
            isDiff: false
          });
        }
      });

      // 处理最后挂起的删除行
      if (pendingRemoved) {
        alignedLeftLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: pendingRemoved.lineNum,
          content: pendingRemoved.content,
          changeType: 'removed',
          isDiff: true
        });
      }
    });

    // 添加剩余的上下文行
    const maxLines = Math.max(diffResult.old_content.length, diffResult.new_content.length);
    for (let i = alignedLeftLines.length; i < maxLines; i++) {
      const oldContent = diffResult.old_content[i] || '';
      const newContent = diffResult.new_content[i] || '';

      if (oldContent && !newContent) {
        // 只有旧版本有，新版本没有 = 删除
        alignedLeftLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: rightLineNum++,
          content: oldContent,
          changeType: 'removed',
          isDiff: true
        });
      } else if (!oldContent && newContent) {
        // 只有新版本有，旧版本没有 = 新增
        alignedLeftLines.push({
          lineNum: leftLineNum++,
          content: newContent,
          changeType: 'added',
          isDiff: true
        });
        alignedRightLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
      } else if (oldContent && newContent) {
        // 两边都有
        alignedLeftLines.push({
          lineNum: leftLineNum++,
          content: newContent,
          changeType: 'unchanged',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: rightLineNum++,
          content: oldContent,
          changeType: 'unchanged',
          isDiff: false
        });
      }
    }

    // 与主窗口保持一致：
    // 主窗口左边显示 rightLines（旧版本），右边显示 leftLines（新版本）
    // 全局搜索弹窗左边显示 oldLines，右边显示 newLines
    // 所以：oldLines = alignedRightLines（旧版本），newLines = alignedLeftLines（新版本）
    oldLines.value = alignedRightLines;  // 旧版本内容，显示在左边
    newLines.value = alignedLeftLines;   // 新版本内容，显示在右边

  } catch (error) {
    console.error('加载文件差异失败:', error);
    oldLines.value = [];
    newLines.value = [];
  }
};

// 滚动到指定行
const scrollToLine = (lineNum: number, side: 'old' | 'new' = 'old') => {
  const lineHeight = 20;
  const targetScrollTop = (lineNum - 1) * lineHeight - 100;

  if (side === 'old' && oldCodeContent.value) {
    oldCodeContent.value.scrollTop = Math.max(0, targetScrollTop);
    // 同步右侧滚动
    if (newCodeContent.value) {
      newCodeContent.value.scrollTop = oldCodeContent.value.scrollTop;
    }
  } else if (side === 'new' && newCodeContent.value) {
    newCodeContent.value.scrollTop = Math.max(0, targetScrollTop);
    // 同步左侧滚动
    if (oldCodeContent.value) {
      oldCodeContent.value.scrollTop = newCodeContent.value.scrollTop;
    }
  }
};

// 同步滚动
let isScrolling = false;
const syncScroll = (source: 'left' | 'right') => {
  if (isScrolling) return;
  isScrolling = true;

  const sourceEl = source === 'left' ? oldCodeContent.value : newCodeContent.value;
  const targetEl = source === 'left' ? newCodeContent.value : oldCodeContent.value;

  if (sourceEl && targetEl) {
    // 同步纵向滚动
    targetEl.scrollTop = sourceEl.scrollTop;
    // 同步横向滚动
    targetEl.scrollLeft = sourceEl.scrollLeft;
  }

  setTimeout(() => {
    isScrolling = false;
  }, 50);
};

// 检查行是否高亮
const isLineHighlighted = (lineNum: number, side: 'old' | 'new') => {
  return highlightedLine.value?.lineNum === lineNum && highlightedLine.value?.side === side;
};

// 检查是否是搜索匹配行
const isSearchMatchLine = (lineNum: number, side: 'old' | 'new') => {
  if (!selectedMatch.value || !searchQuery.value) return false;
  
  const match = selectedMatch.value.match;
  if (match.line_number !== lineNum) return false;
  
  // 检查该行内容是否包含搜索词
  const lines = side === 'old' ? oldLines.value : newLines.value;
  const line = lines.find(l => l.lineNum === lineNum);
  if (!line) return false;
  
  const searchLower = searchQuery.value.toLowerCase();
  const contentLower = line.content.toLowerCase();
  return contentLower.includes(searchLower);
};

// 关闭预览
const closePreview = () => {
  showPreview.value = false;
  previewFilePath.value = '';
  oldLines.value = [];
  newLines.value = [];
  selectedMatch.value = null;
  highlightedLine.value = null;
};

// 打开搜索对话框
const open = (path: string) => {
  repoPath.value = path;
  visible.value = true;
  nextTick(() => {
    searchInput.value?.focus();
  });
};

// 关闭对话框
const close = () => {
  visible.value = false;
  searchResults.value = [];
  hasSearched.value = false;
  showPreview.value = false;
  selectedResultIndex.value = null;
  selectedMatch.value = null;
  emit('close');
};

// 切换搜索模式
const toggleRegex = () => {
  useRegex.value = !useRegex.value;
  searchInput.value?.focus();
};

// 执行搜索
const performSearch = async () => {
  if (!searchQuery.value || !repoPath.value) return;

  isSearching.value = true;
  hasSearched.value = true;
  searchResults.value = [];
  showPreview.value = false;
  selectedResultIndex.value = null;
  selectedMatch.value = null;

  try {
    const results = await invoke<SearchResult[]>('search_in_directory', {
      dirPath: repoPath.value,
      pattern: searchQuery.value,
      filePattern: filePattern.value || null
    });
    
    searchResults.value = results;
    expandedFiles.value = new Array(results.length).fill(false);
  } catch (error) {
    console.error('搜索失败:', error);
    alert('搜索失败：' + error);
  } finally {
    isSearching.value = false;
  }
};

// 在主窗口打开文件
const openFileInMain = (path: string, lineNumber?: number) => {
  emit('open-file', path, lineNumber, searchQuery.value);
};

// 高亮匹配文本（显示整行内容）
const highlightMatch = (content: string, matchedText: string): string => {
  const escaped = matchedText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escaped})`, 'gi');
  return content.replace(regex, '<span class="search-highlight">$1</span>');
};

// 获取唯一的匹配行（去重，同一行号只显示一次）
const getUniqueMatches = (matches: SearchMatch[]): SearchMatch[] => {
  const lineMap = new Map<number, SearchMatch>();
  matches.forEach(match => {
    if (!lineMap.has(match.line_number)) {
      lineMap.set(match.line_number, match);
    }
  });
  return Array.from(lineMap.values()).sort((a, b) => a.line_number - b.line_number);
};

// 获取匹配内容的上下文（前后各 80 个字符）
const getContextAroundMatch = (content: string, matchedText: string, contextLength: number = 80): string => {
  const index = content.toLowerCase().indexOf(matchedText.toLowerCase());
  if (index === -1) return content;
  
  const start = Math.max(0, index - contextLength);
  const end = Math.min(content.length, index + matchedText.length + contextLength);
  
  let result = content.substring(start, end);
  if (start > 0) result = '...' + result;
  if (end < content.length) result = result + '...';
  
  return result;
};

// 获取相对路径
const getRelativePath = (fullPath: string): string => {
  if (!repoPath.value) return fullPath;
  if (fullPath.startsWith(repoPath.value)) {
    return fullPath.substring(repoPath.value.length).replace(/^[/\\]/, '');
  }
  return fullPath;
};

// 暴露方法给父组件
defineExpose({
  open,
  close
});
</script>

<style scoped>
.global-search-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.global-search-dialog {
  background-color: var(--bg-primary);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.3);
  width: 800px;
  max-height: 80vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.global-search-dialog.with-preview {
  width: 1400px;
  max-width: 95vw;
}

.search-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.search-header h3 {
  margin: 0;
  font-size: 16px;
  color: var(--text-primary);
}

.close-btn {
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.close-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.search-form {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.search-input-group {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.search-input {
  flex: 1;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
  outline: none;
}

.search-input:focus {
  border-color: var(--accent-color);
}

.mode-btn {
  width: 44px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
  transition: all 0.2s;
}

.mode-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.mode-btn.active {
  background-color: var(--accent-color);
  border-color: var(--accent-color);
  color: white;
}

.mode-icon {
  font-family: monospace;
}

.search-btn {
  width: 80px;
  height: 38px;
  background-color: var(--accent-color);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
  transition: opacity 0.2s;
}

.search-btn:hover:not(:disabled) {
  opacity: 0.9;
}

.search-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.search-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
}

.file-pattern-input {
  flex: 1;
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  max-width: 300px;
}

.file-pattern-input:focus {
  border-color: var(--accent-color);
}

.search-path {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.search-main-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.search-results {
  flex: 1;
  overflow-y: auto;
  padding: 0;
  min-width: 400px;
}

.search-results.narrow {
  flex: 0 0 400px;
  border-right: 1px solid var(--border-color);
}

.results-header {
  padding: 12px 20px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.results-count {
  font-size: 13px;
  color: var(--text-secondary);
}

.results-list {
  padding: 0;
}

.result-item {
  border-bottom: 1px solid var(--border-color);
}

.result-item.active {
  background-color: var(--bg-hover);
}

.result-file {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  font-weight: 500;
  color: var(--text-primary);
  border-bottom: 1px solid var(--border-color);
  cursor: pointer;
}

.result-file:hover {
  background-color: var(--bg-hover);
}

.expand-icon {
  font-size: 12px;
  color: var(--text-secondary);
  transition: transform 0.2s;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.expand-icon.expanded {
  transform: rotate(90deg);
}

.file-icon {
  font-size: 16px;
  cursor: pointer;
}

.file-path {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-weight: 500;
}

.match-count-badge {
  background-color: var(--accent-color);
  color: white;
  padding: 2px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

.result-matches {
  padding: 0;
}

.match-line {
  display: flex;
  padding: 6px 20px 6px 48px;
  gap: 12px;
  font-size: 13px;
  line-height: 1.5;
  border-bottom: 1px solid var(--border-color-light);
  cursor: pointer;
  transition: background-color 0.2s;
}

.match-line:hover {
  background-color: var(--bg-hover);
}

.match-line.active {
  background-color: var(--accent-color);
  color: white;
}

.match-line.active .line-number {
  color: rgba(255, 255, 255, 0.8);
}

.line-number {
  color: var(--text-secondary);
  font-family: monospace;
  font-size: 12px;
  min-width: 40px;
  text-align: right;
  flex-shrink: 0;
}

.line-content {
  flex: 1;
  font-family: monospace;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--text-primary);
}

:deep(.search-highlight) {
  background-color: rgba(255, 235, 59, 0.5);
  border-radius: 2px;
  padding: 1px 2px;
}

.match-line.active :deep(.search-highlight) {
  background-color: rgba(255, 255, 0, 0.8);
  color: black;
}

.more-matches {
  padding: 8px 20px 8px 48px;
  font-size: 12px;
  color: var(--text-secondary);
  font-style: italic;
}

.no-results {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.no-results-icon {
  font-size: 48px;
  margin-bottom: 16px;
}

.no-results-text {
  font-size: 14px;
}

/* 代码预览面板 */
.code-preview-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--bg-primary);
}

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
}

.preview-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preview-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.preview-btn {
  padding: 6px 12px;
  background-color: var(--accent-color);
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
}

.preview-btn:hover {
  opacity: 0.9;
}

.preview-btn.close {
  background-color: transparent;
  color: var(--text-secondary);
  font-size: 16px;
  padding: 4px 8px;
}

.preview-btn.close:hover {
  color: var(--text-primary);
  background-color: var(--bg-hover);
}

.preview-content {
  flex: 1;
  overflow: hidden;
}

.diff-view {
  display: flex;
  height: 100%;
  overflow: hidden;
}

.diff-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.pane-header {
  padding: 8px 12px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  font-weight: 600;
}

.diff-divider {
  width: 1px;
  background-color: var(--border-color);
}

.code-content {
  flex: 1;
  overflow: auto;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 12px;
  line-height: 20px;
  padding: 8px 0;
  user-select: text !important;
  -webkit-user-select: text !important;
  -moz-user-select: text !important;
  -ms-user-select: text !important;
}

.empty-content {
  padding: 40px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}

.code-line {
  display: flex;
  padding: 0 8px;
  white-space: pre;
  min-height: 20px;
}

.code-line:hover {
  background-color: var(--bg-hover);
}

.code-line.highlight {
  background-color: rgba(255, 235, 59, 0.3);
}

.code-line.search-match {
  background-color: rgba(255, 235, 59, 0.5);
}

.line-num {
  min-width: 50px;
  text-align: right;
  padding-right: 12px;
  color: var(--text-secondary);
  user-select: none;
}

.line-text {
  flex: 1;
  white-space: pre;
}

.line-text.added {
  background-color: rgba(76, 175, 80, 0.2);
}

.line-text.removed {
  background-color: rgba(244, 67, 54, 0.2);
}

.line-text.changed {
  background-color: rgba(33, 150, 243, 0.2);
}

.line-text.empty {
  background-color: transparent;
}
</style>
