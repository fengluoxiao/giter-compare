<template>
  <div v-if="visible" class="global-search-overlay" @click.self="close">
    <div class="global-search-dialog">
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

      <div class="search-results" v-if="searchResults.length > 0">
        <div class="results-header">
          <span class="results-count">找到 {{ totalMatches }} 个匹配，{{ searchResults.length }} 个文件</span>
        </div>
        
        <div class="results-list">
          <div 
            v-for="(result, index) in searchResults" 
            :key="index"
            class="result-item"
            @click="openFile(result.file_path)"
          >
            <div class="result-file">
              <span class="file-icon">📄</span>
              <span class="file-path">{{ result.file_path }}</span>
              <span class="match-count-badge">{{ result.match_count }}</span>
            </div>
            <div class="result-matches">
              <div 
                v-for="(match, matchIndex) in result.matches.slice(0, 5)" 
                :key="matchIndex"
                class="match-line"
                @click.stop="openFileAtLine(result.file_path, match.line_number)"
              >
                <span class="line-number">{{ match.line_number }}</span>
                <span class="line-content" v-html="highlightMatch(match.line_content, match.matched_text)"></span>
              </div>
              <div v-if="result.matches.length > 5" class="more-matches">
                还有 {{ result.matches.length - 5 }} 个匹配...
              </div>
            </div>
          </div>
        </div>
      </div>

      <div v-else-if="hasSearched && !isSearching" class="no-results">
        <span class="no-results-icon">🔍</span>
        <span class="no-results-text">没有找到匹配项</span>
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

const emit = defineEmits<{
  close: [];
  'open-file': [path: string, lineNumber?: number];
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

const totalMatches = computed(() => {
  return searchResults.value.reduce((sum, result) => sum + result.match_count, 0);
});

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

  try {
    const results = await invoke<SearchResult[]>('search_in_directory', {
      dirPath: repoPath.value,
      pattern: searchQuery.value,
      filePattern: filePattern.value || null
    });
    
    searchResults.value = results;
  } catch (error) {
    console.error('搜索失败:', error);
    alert('搜索失败：' + error);
  } finally {
    isSearching.value = false;
  }
};

// 打开文件
const openFile = (path: string) => {
  emit('open-file', path);
};

// 打开文件并跳转到指定行
const openFileAtLine = (path: string, lineNumber: number) => {
  emit('open-file', path, lineNumber);
};

// 高亮匹配文本
const highlightMatch = (content: string, matchedText: string): string => {
  const escaped = matchedText.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const regex = new RegExp(`(${escaped})`, 'gi');
  return content.replace(regex, '<span class="search-highlight">$1</span>');
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

.search-results {
  flex: 1;
  overflow-y: auto;
  padding: 0;
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
  cursor: pointer;
  transition: background-color 0.2s;
}

.result-item:hover {
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
}

.file-icon {
  font-size: 16px;
}

.file-path {
  flex: 1;
  font-size: 13px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
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
}

.match-line:hover {
  background-color: var(--bg-hover);
}

.line-number {
  color: var(--text-secondary);
  font-family: monospace;
  font-size: 12px;
  min-width: 40px;
  text-align: right;
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
</style>
