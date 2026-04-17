<template>
  <div v-show="visible" class="inline-search">
    <div class="search-box">
      <input
        ref="searchInput"
        v-model="searchQuery"
        type="text"
        class="search-input"
        :placeholder="useRegex ? '正则表达式搜索...' : '搜索...（支持通配符 * 和 ?）'"
        @keydown.enter="onEnter"
        @keydown.escape="closeSearch"
      />
      <div class="search-controls">
        <button 
          class="mode-btn" 
          :class="{ active: useRegex }"
          @click="toggleRegex" 
          :title="useRegex ? '当前：正则表达式模式' : '当前：通配符模式'"
        >
          <span class="mode-icon">.*</span>
        </button>
        <span class="match-count" v-if="matchCount > 0">
          {{ currentMatchIndex + 1 }} / {{ matchCount }}
        </span>
        <button class="nav-btn" @click="findPrev" title="上一个 (Shift+Enter)">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M7.41 15.41L12 10.83l4.59 4.58L18 14l-6-6-6 6z"/>
          </svg>
        </button>
        <button class="nav-btn" @click="findNext" title="下一个 (Enter)">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M7.41 8.59L12 13.17l4.59-4.58L18 10l-6 6-6-6z"/>
          </svg>
        </button>
        <button class="close-btn" @click="closeSearch" title="关闭 (Esc)">
          <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';

interface SearchMatch {
  lineIndex: number;
  columnIndex: number;
  text: string;
}

const props = defineProps<{
  content: string;
}>();

const emit = defineEmits<{
  close: [];
  'highlight-match': [matches: SearchMatch[], currentMatch: SearchMatch];
}>();

const visible = ref(false);
const searchInput = ref<HTMLInputElement | null>(null);
const searchQuery = ref('');
const matches = ref<SearchMatch[]>([]);
const currentMatchIndex = ref(-1);
const useRegex = ref(false);

const matchCount = computed(() => matches.value.length);

// 切换搜索模式
const toggleRegex = () => {
  useRegex.value = !useRegex.value;
  // 切换模式后重新搜索
  if (searchQuery.value) {
    performSearch();
  }
};

// 将通配符模式转换为正则表达式
const wildcardToRegex = (pattern: string): RegExp => {
  if (useRegex.value) {
    // 直接使用正则表达式
    try {
      return new RegExp(pattern, 'gi');
    } catch (e) {
      // 无效的正则表达式，返回匹配不到的正则
      return new RegExp('(?!)', 'gi');
    }
  } else {
    // 转义正则特殊字符，但保留 *
    const escaped = pattern.replace(/[.+?^${}()|[\]\\]/g, '\\$&');
    // 将 * 转换为 .*
    const regexPattern = escaped.replace(/\*/g, '.*');
    return new RegExp(regexPattern, 'gi');
  }
};

// 在内容中搜索匹配项
const findMatches = (query: string): SearchMatch[] => {
  if (!query || !props.content) return [];

  const regex = wildcardToRegex(query);
  const foundMatches: SearchMatch[] = [];
  
  const lines = props.content.split('\n');
  let globalLineIndex = 0;

  for (let lineIndex = 0; lineIndex < lines.length; lineIndex++) {
    const line = lines[lineIndex];
    let match;
    
    // 重置正则的 lastIndex
    regex.lastIndex = 0;
    
    while ((match = regex.exec(line)) !== null) {
      foundMatches.push({
        lineIndex: globalLineIndex,
        columnIndex: match.index,
        text: match[0]
      });
      
      // 避免空匹配导致无限循环
      if (match.index === regex.lastIndex) {
        regex.lastIndex++;
      }
    }
    
    globalLineIndex++;
  }

  return foundMatches;
};

// 执行搜索
const performSearch = () => {
  matches.value = findMatches(searchQuery.value);
  currentMatchIndex.value = matches.value.length > 0 ? 0 : -1;
  
  if (matches.value.length > 0) {
    emit('highlight-match', matches.value, matches.value[currentMatchIndex.value]);
  } else {
    emit('highlight-match', [], null as any);
  }
};

// 查找下一个
const findNext = () => {
  if (matches.value.length === 0) return;
  
  currentMatchIndex.value = (currentMatchIndex.value + 1) % matches.value.length;
  emit('highlight-match', matches.value, matches.value[currentMatchIndex.value]);
};

// 查找上一个
const findPrev = () => {
  if (matches.value.length === 0) return;
  
  currentMatchIndex.value = currentMatchIndex.value <= 0 
    ? matches.value.length - 1 
    : currentMatchIndex.value - 1;
  emit('highlight-match', matches.value, matches.value[currentMatchIndex.value]);
};

// 回车处理
const onEnter = (event: KeyboardEvent) => {
  if (event.shiftKey) {
    findPrev();
  } else {
    findNext();
  }
};

// 关闭搜索
const closeSearch = () => {
  visible.value = false;
  searchQuery.value = '';
  matches.value = [];
  currentMatchIndex.value = -1;
  emit('highlight-match', [], null as any);
  emit('close');
};

// 打开搜索
const openSearch = () => {
  visible.value = true;
  nextTick(() => {
    searchInput.value?.focus();
    searchInput.value?.select();
  });
};

// 监听搜索输入
watch(searchQuery, () => {
  performSearch();
});

// 监听内容变化，重新搜索
watch(() => props.content, () => {
  if (visible.value && searchQuery.value) {
    performSearch();
  }
});

// 暴露方法给父组件
defineExpose({
  openSearch,
  closeSearch,
  findNext,
  findPrev
});
</script>

<style scoped>
.inline-search {
  position: absolute;
  top: 8px;
  right: 8px;
  z-index: 1000;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 6px;
  min-width: 350px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 6px;
}

.search-input {
  flex: 1;
  padding: 5px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 13px;
  outline: none;
  min-width: 200px;
}

.search-input:focus {
  border-color: var(--accent-color);
}

.search-input::placeholder {
  color: var(--text-secondary);
  font-size: 12px;
}

.search-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.mode-btn {
  width: 28px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 3px;
  padding: 0;
  font-size: 11px;
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

.match-count {
  font-size: 12px;
  color: var(--text-secondary);
  min-width: 60px;
  text-align: right;
  padding: 0 4px;
}

.nav-btn,
.close-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 3px;
  padding: 0;
}

.nav-btn:hover,
.close-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.close-btn:hover {
  background-color: #ff4444;
  color: white;
}
</style>
