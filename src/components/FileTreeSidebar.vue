<template>
  <aside
    class="sidebar"
    :class="{ collapsed: isCollapsed }"
    :style="{ width: isCollapsed ? '40px' : width + 'px' }"
  >
    <div class="sidebar-header">
      <div class="sidebar-header-title">
        <h3 v-show="!isCollapsed">文件变更</h3>
        <button
          class="btn btn-icon btn-collapse"
          @click="$emit('toggle-collapse')"
          :title="isCollapsed ? '展开' : '折叠'"
        >
          <svg v-if="isCollapsed" viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M8.59 16.59L13.17 12 8.59 7.41 10 6l6 6-6 6-1.41-1.41z"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M15.41 7.41L10.83 12l4.58 4.59L14 18l-6-6 6-6 1.41 1.41z"/>
          </svg>
        </button>
      </div>
      <div v-show="!isCollapsed" class="filter-tabs">
        <button
          class="tab"
          :class="{ active: viewMode === 'working' }"
          @click="$emit('update:viewMode', 'working')"
        >
          文件树
        </button>
        <button
          class="tab"
          :class="{ active: viewMode === 'staged' }"
          @click="$emit('update:viewMode', 'staged')"
        >
          更改
        </button>
      </div>
      <!-- 只在工作区显示"显示所有文件"选项 -->
      <div v-show="!isCollapsed && viewMode === 'working'" class="view-toggle">
        <label class="toggle-label">
          <input
            type="checkbox"
            :checked="showAllFiles"
            @change="$emit('update:showAllFiles', ($event.target as HTMLInputElement).checked)"
          />
          <span class="toggle-text">显示所有文件</span>
        </label>
      </div>
      <!-- 显示已删除文件选项 -->
      <div v-show="!isCollapsed && viewMode === 'working' && hasDeletedFiles" class="view-toggle">
        <label class="toggle-label">
          <input
            type="checkbox"
            :checked="showDeletedFiles"
            @change="$emit('update:showDeletedFiles', ($event.target as HTMLInputElement).checked)"
          />
          <span class="toggle-text">显示已删除文件</span>
        </label>
      </div>
      <!-- 搜索框 -->
      <div v-show="!isCollapsed" class="search-box">
        <input
          v-model="searchQuery"
          type="text"
          class="search-input"
          placeholder="搜索文件..."
        />
        <svg v-if="searchQuery" class="search-clear" viewBox="0 0 24 24" width="16" height="16" fill="currentColor" @click="searchQuery = ''">
          <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
        </svg>
      </div>
    </div>
    <div v-show="!isCollapsed" class="file-list">
      <!-- 工作区：显示文件树 -->
      <FileTree
        v-if="viewMode === 'working' && fileTree.length > 0"
        :nodes="filteredFileTree"
        @select="$emit('select-file', $event)"
        @toggle="$emit('toggle-directory', $event)"
      />
      <!-- 暂存区：显示更改的文件列表 -->
      <StagedFileList
        v-else-if="viewMode === 'staged'"
        :staged-files="filteredStagedFiles"
        :selected-path="selectedStagedPath"
        @select="$emit('select-staged-file', $event)"
      />
      <div v-else class="empty-state">
        选择左侧项目或点击"打开文件夹"
      </div>
    </div>
    <!-- 拖拽调整宽度的手柄 -->
    <div
      v-show="!isCollapsed"
      class="resize-handle resize-handle-file"
      @mousedown="$emit('start-resize', $event)"
    ></div>
  </aside>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import FileTree from './FileTree.vue';
import StagedFileList from './StagedFileList.vue';

interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  status?: string;
  children: FileNode[];
  expanded?: boolean;
}

interface StagedFile {
  name: string;
  path: string;
  status?: string;
}

interface GitChange {
  path: string;
  status: string;
}

const props = defineProps<{
  fileTree: FileNode[];
  viewMode: 'working' | 'staged';
  showAllFiles: boolean;
  showDeletedFiles: boolean;
  gitChanges: GitChange[];
  isCollapsed: boolean;
  width: number;
  stagedFiles?: StagedFile[];
  selectedStagedPath?: string;
}>();

defineEmits<{
  'update:viewMode': [mode: 'working' | 'staged'];
  'update:showAllFiles': [value: boolean];
  'update:showDeletedFiles': [value: boolean];
  'toggle-collapse': [];
  'select-file': [path: string];
  'toggle-directory': [node: FileNode];
  'select-staged-file': [path: string];
  'start-resize': [event: MouseEvent];
}>();

// 搜索功能
const searchQuery = ref('');

// 检查文件是否匹配搜索
const matchesSearch = (name: string, path: string): boolean => {
  if (!searchQuery.value) return true;
  const query = searchQuery.value.toLowerCase();
  return name.toLowerCase().includes(query) || path.toLowerCase().includes(query);
};

// 递归过滤文件树
const filterFileTree = (nodes: FileNode[]): FileNode[] => {
  if (!searchQuery.value) return nodes;
  
  return nodes
    .map(node => {
      if (node.type === 'directory') {
        const filteredChildren = filterFileTree(node.children);
        // 如果子节点有匹配的，或者文件夹本身匹配，则包含该文件夹
        if (filteredChildren.length > 0 || matchesSearch(node.name, node.path)) {
          return {
            ...node,
            children: filteredChildren,
            expanded: true // 搜索时自动展开匹配的文件夹
          };
        }
        return null;
      } else {
        // 文件节点
        return matchesSearch(node.name, node.path) ? node : null;
      }
    })
    .filter(node => node !== null) as FileNode[];
};

// 过滤更改列表
const filterStagedFiles = (files: StagedFile[]): StagedFile[] => {
  if (!searchQuery.value) return files;
  return files.filter(file => matchesSearch(file.name, file.path));
};

// 计算属性
const filteredFileTree = computed(() => filterFileTree(props.fileTree));
const filteredStagedFiles = computed(() => filterStagedFiles(props.stagedFiles || []));

// 检查是否有已删除的文件（基于 Git 变更数据，而不是文件树）
const hasDeletedFiles = computed(() => {
  return props.gitChanges.some(change => change.status === 'Deleted');
});
</script>

<style scoped>
.sidebar {
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  position: relative;
}

.sidebar-header {
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
}

.sidebar-header-title {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.sidebar-header-title h3 {
  font-size: 14px;
  color: var(--text-primary);
}

.filter-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 8px;
}

.tab {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--border-color);
  background-color: var(--bg-primary);
  color: var(--text-secondary);
  font-size: 11px;
  cursor: pointer;
  border-radius: 3px;
  transition: all 0.2s;
}

.tab:hover {
  background-color: var(--bg-hover);
}

.tab.active {
  background-color: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.view-toggle {
  padding-top: 4px;
}

.toggle-label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  font-size: 12px;
  color: var(--text-secondary);
  transition: color 0.2s;
}

.toggle-label:hover {
  color: var(--text-primary);
}

.toggle-label input[type="checkbox"] {
  appearance: none;
  -webkit-appearance: none;
  width: 18px;
  height: 18px;
  border: 2px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  cursor: pointer;
  position: relative;
  transition: all 0.2s;
  flex-shrink: 0;
}

.toggle-label input[type="checkbox"]:hover {
  border-color: var(--accent-color);
}

.toggle-label input[type="checkbox"]:checked {
  background-color: var(--accent-color);
  border-color: var(--accent-color);
}

.toggle-label input[type="checkbox"]:checked::after {
  content: '✓';
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: white;
  font-size: 12px;
  font-weight: bold;
}

.toggle-text {
  user-select: none;
}

.search-box {
  position: relative;
  margin-top: 8px;
}

.search-input {
  width: 100%;
  padding: 6px 32px 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  outline: none;
  transition: border-color 0.2s;
  box-sizing: border-box;
}

.search-input:focus {
  border-color: var(--accent-color);
}

.search-input::placeholder {
  color: var(--text-secondary);
}

.search-clear {
  position: absolute;
  right: 8px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-secondary);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.search-clear:hover {
  color: var(--text-primary);
}

.file-list {
  flex: 1;
  overflow: auto;
  padding: 8px;
}

.empty-state {
  padding: 16px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 12px;
}

/* 折叠状态 */
.sidebar.collapsed {
  min-width: 40px;
  max-width: 40px;
}

.sidebar.collapsed .sidebar-header {
  padding: 8px 4px;
}

.sidebar.collapsed .sidebar-header-title {
  flex-direction: column;
  gap: 8px;
  margin-bottom: 0;
  align-items: center;
}

.sidebar.collapsed .btn-icon {
  width: 28px;
  height: 28px;
}

.sidebar.collapsed .view-toggle,
.sidebar.collapsed .filter-tabs {
  display: none !important;
}

/* 拖拽手柄 */
.resize-handle {
  position: absolute;
  right: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  background-color: transparent;
  transition: background-color 0.2s;
  z-index: 10;
}

.resize-handle:hover,
.resize-handle:active {
  background-color: var(--accent-color);
}
</style>
