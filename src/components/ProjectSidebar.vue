<template>
  <aside
    class="project-sidebar"
    :class="{ collapsed: isCollapsed }"
    :style="{ width: isCollapsed ? '40px' : width + 'px' }"
  >
    <div class="project-header">
      <div class="workspace-selector" v-if="!isCollapsed">
        <select :value="currentWorkspaceId" @change="onWorkspaceChange">
          <option v-for="ws in workspaces" :key="ws.id" :value="ws.id">
            📁 {{ ws.name }}
          </option>
        </select>
        <button class="btn btn-icon btn-add-workspace" @click="$emit('add-workspace')" title="新建工作区">
          <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
          </svg>
        </button>
      </div>
      <h3 v-show="!isCollapsed">项目列表</h3>
      <div class="project-header-buttons">
        <button class="btn btn-icon btn-add" @click="$emit('add-project')" title="添加项目">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
          </svg>
        </button>
        <button class="btn btn-icon btn-export" @click="$emit('export-projects')" title="导出当前工作区">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
          </svg>
        </button>
        <button class="btn btn-icon btn-import" @click="$emit('import-projects')" title="导入工作区">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z"/>
          </svg>
        </button>
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
    </div>
    <div v-show="!isCollapsed" class="project-list">
      <div
        v-for="project in projects"
        :key="project.id"
        class="project-item"
        :class="{ active: currentProjectId === project.id }"
        @click="$emit('switch-project', project)"
        @contextmenu.prevent="showContextMenu($event, project)"
        :title="(project.name ? project.name + '\n' : '') + project.path"
      >
        <span class="project-icon">📁</span>
        <span class="project-name">{{ project.name || getFolderName(project.path) }}</span>
        <button
          class="btn btn-icon btn-delete"
          @click.stop="$emit('remove-project', project.id)"
          title="删除项目"
        >
          <svg viewBox="0 0 24 24" width="12" height="12" fill="currentColor">
            <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
          </svg>
        </button>
      </div>
      <div v-if="projects.length === 0" class="empty-state">
        点击 + 添加 Git 项目
      </div>
    </div>
    <!-- 拖拽调整宽度的手柄 -->
    <div
      v-show="!isCollapsed"
      class="resize-handle"
      @mousedown="$emit('start-resize', $event)"
    ></div>

    <!-- 右键菜单 -->
    <div
      v-if="contextMenu.visible"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="context-menu-item" @click="handleOpenInTerminal">
        <span class="menu-icon">🖥️</span>
        <span class="menu-text">在终端中打开</span>
      </div>
      <div class="context-menu-item" @click="handleOpenInExplorer">
        <span class="menu-icon">📂</span>
        <span class="menu-text">在资源管理器中打开</span>
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item" @click="handleProjectSettings">
        <span class="menu-icon">⚙️</span>
        <span class="menu-text">项目设置</span>
      </div>
      <div class="context-menu-item" @click="handleCopyPath">
        <span class="menu-icon">📋</span>
        <span class="menu-text">复制路径</span>
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item danger" @click="handleRemoveProject">
        <span class="menu-icon">🗑️</span>
        <span class="menu-text">从列表中移除</span>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';

interface Project {
  id: string;
  name: string;
  path: string;
}

interface Workspace {
  id: string;
  name: string;
  projects: Project[];
}

interface ContextMenuState {
  visible: boolean;
  x: number;
  y: number;
  project: Project | null;
}

const props = defineProps<{
  workspaces: Workspace[];
  currentWorkspaceId: string;
  projects: Project[];
  currentProjectId: string;
  isCollapsed: boolean;
  width: number;
}>();

const emit = defineEmits<{
  'add-project': [];
  'add-workspace': [];
  'toggle-collapse': [];
  'switch-project': [project: Project];
  'remove-project': [projectId: string];
  'start-resize': [event: MouseEvent];
  'export-projects': [];
  'import-projects': [];
  'switch-workspace': [workspaceId: string];
  'project-settings': [project: Project];
}>();

const contextMenu = ref<ContextMenuState>({
  visible: false,
  x: 0,
  y: 0,
  project: null
});

const onWorkspaceChange = (event: Event) => {
  const target = event.target as HTMLSelectElement;
  emit('switch-workspace', target.value);
};

// 从路径获取文件夹名称
const getFolderName = (path: string): string => {
  // 处理 Windows 和 Unix 路径
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || parts[parts.length - 2] || '新项目';
};

// 显示右键菜单
const showContextMenu = (event: MouseEvent, project: Project) => {
  contextMenu.value = {
    visible: true,
    x: event.clientX,
    y: event.clientY,
    project
  };
};

// 隐藏右键菜单
const hideContextMenu = () => {
  contextMenu.value.visible = false;
  contextMenu.value.project = null;
};

// 在终端中打开
const handleOpenInTerminal = async () => {
  if (contextMenu.value.project) {
    try {
      await invoke('open_in_terminal', { path: contextMenu.value.project.path });
    } catch (error) {
      console.error('打开终端失败:', error);
      alert('打开终端失败：' + error);
    }
  }
  hideContextMenu();
};

// 在资源管理器中打开
const handleOpenInExplorer = async () => {
  if (contextMenu.value.project) {
    try {
      await invoke('open_in_explorer', { path: contextMenu.value.project.path });
    } catch (error) {
      console.error('打开资源管理器失败:', error);
      alert('打开资源管理器失败：' + error);
    }
  }
  hideContextMenu();
};

// 项目设置
const handleProjectSettings = () => {
  if (contextMenu.value.project) {
    emit('project-settings', contextMenu.value.project);
  }
  hideContextMenu();
};

// 复制路径
const handleCopyPath = async () => {
  if (contextMenu.value.project) {
    try {
      await navigator.clipboard.writeText(contextMenu.value.project.path);
      console.log('路径已复制:', contextMenu.value.project.path);
    } catch (error) {
      console.error('复制路径失败:', error);
    }
  }
  hideContextMenu();
};

// 从列表中移除项目
const handleRemoveProject = () => {
  if (contextMenu.value.project) {
    emit('remove-project', contextMenu.value.project.id);
  }
  hideContextMenu();
};

// 点击其他地方隐藏菜单
const handleClickOutside = (event: MouseEvent) => {
  const target = event.target as HTMLElement;
  if (!target.closest('.context-menu')) {
    hideContextMenu();
  }
};

onMounted(() => {
  document.addEventListener('click', handleClickOutside);
});

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside);
});
</script>

<style scoped>
.project-sidebar {
  background-color: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  position: relative;
}

.project-header {
  display: flex;
  flex-direction: column;
  gap: 8px;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
}

.workspace-selector {
  display: flex;
  align-items: center;
  gap: 4px;
  width: 100%;
}

.workspace-selector select {
  flex: 1;
  padding: 6px 28px 6px 8px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 12px;
  cursor: pointer;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 24 24' fill='none' stroke='%23888' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 8px center;
  background-size: 12px;
}

.workspace-selector select:focus {
  outline: none;
  border-color: var(--accent-color);
}

.btn-add-workspace {
  width: 30px;
  height: 30px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-secondary);
  cursor: pointer;
  transition: all 0.2s;
  padding: 0;
  flex-shrink: 0;
}

.btn-add-workspace svg {
  width: 14px;
  height: 14px;
  fill: currentColor;
}

.btn-add-workspace:hover {
  background-color: var(--bg-hover);
  color: var(--accent-color);
  border-color: var(--accent-color);
}

.project-header-buttons {
  display: flex;
  gap: 4px;
}

.project-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.project-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
  margin-bottom: 4px;
}

.project-item:hover {
  background-color: var(--bg-hover);
}

.project-item.active {
  background-color: var(--accent-color);
  color: white;
}

.project-item.active .project-name {
  color: white;
}

.project-item.active .btn-delete {
  opacity: 1;
  visibility: visible;
  color: rgba(255, 255, 255, 0.8);
}

.project-item.active .btn-delete:hover {
  color: white;
  background-color: rgba(255, 255, 255, 0.2);
}

.project-icon {
  font-size: 16px;
}

.project-name {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

/* 折叠状态 */
.project-sidebar.collapsed {
  min-width: 40px;
  max-width: 40px;
}

.project-sidebar.collapsed .project-header {
  padding: 8px 4px;
  justify-content: center;
  flex-direction: column;
  gap: 8px;
}

.project-sidebar.collapsed .project-header-buttons {
  flex-direction: column;
  gap: 8px;
}

.project-sidebar.collapsed .btn-icon {
  width: 28px;
  height: 28px;
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

.empty-state {
  padding: 16px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 12px;
}

.btn-export,
.btn-import {
  color: var(--text-secondary);
}

.btn-export:hover,
.btn-import:hover {
  color: var(--accent-color);
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 1000;
  min-width: 180px;
  padding: 4px 0;
}

.context-menu-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  cursor: pointer;
  transition: background-color 0.2s;
  font-size: 13px;
  color: var(--text-primary);
}

.context-menu-item:hover {
  background-color: var(--bg-hover);
}

.context-menu-item.danger {
  color: #f44336;
}

.context-menu-item.danger:hover {
  background-color: rgba(244, 67, 54, 0.1);
}

.menu-icon {
  font-size: 14px;
  width: 20px;
  text-align: center;
}

.menu-text {
  flex: 1;
}

.context-menu-divider {
  height: 1px;
  background-color: var(--border-color);
  margin: 4px 0;
}
</style>
