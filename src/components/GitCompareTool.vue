<template>
  <div class="app" :data-theme="theme">
    <!-- 工具栏 -->
    <div class="toolbar">
      <div class="toolbar-left">
        <button class="btn btn-primary" @click="openFolder">
          <span class="btn-icon">📂</span>
          打开文件夹
        </button>
        <button class="btn btn-secondary" @click="showCompareFileDialog">
          <span class="btn-icon">📄</span>
          比对文件
        </button>
        <button class="btn btn-secondary" @click="showTextCompare = true">
          <span class="btn-icon">📝</span>
          比对文本
        </button>
      </div>
      <div class="toolbar-center">
        <span class="path-display">{{ currentPath || '未选择文件夹' }}</span>
      </div>
      <div class="toolbar-right">
        <button class="btn btn-secondary" @click="toggleTheme" title="切换主题">
          {{ theme === 'dark' ? '☀️ 浅色' : '🌙 深色' }}
        </button>
        <button class="btn btn-secondary" @click="navigatePrev" title="上一个" :disabled="!hasPrev">
          ⬆️ 上一个
        </button>
        <button class="btn btn-secondary" @click="navigateNext" title="下一个" :disabled="!hasNext">
          ⬇️ 下一个
        </button>
        <button class="btn btn-secondary" @click="refresh" title="刷新">
          🔄 刷新
        </button>
      </div>
    </div>

    <!-- 标签栏 -->
    <TabBar
      v-if="tabs.length > 0"
      :tabs="tabs"
      :active-tab-id="activeTabId"
      @activate="activateTab"
      @close="closeTab"
      @close-all="closeAllTabs"
      @close-others="closeOtherTabs"
      @close-to-right="closeTabsToRight"
    />

    <!-- 主内容区 -->
    <div class="main-container">
      <!-- 左侧项目列表 -->
      <aside 
        class="project-sidebar" 
        :class="{ collapsed: isProjectSidebarCollapsed }"
        :style="{ width: isProjectSidebarCollapsed ? '40px' : projectSidebarWidth + 'px' }"
      >
        <div class="project-header">
          <h3 v-show="!isProjectSidebarCollapsed">项目列表</h3>
          <div class="project-header-buttons">
            <button class="btn btn-icon" @click="showAddProjectDialog" title="添加项目">
              ➕
            </button>
            <button 
              class="btn btn-icon btn-collapse" 
              @click="toggleProjectSidebar"
              :title="isProjectSidebarCollapsed ? '展开' : '折叠'"
            >
              {{ isProjectSidebarCollapsed ? '▶' : '◀' }}
            </button>
          </div>
        </div>
        <div v-show="!isProjectSidebarCollapsed" class="project-list">
          <div
            v-for="project in projects"
            :key="project.id"
            class="project-item"
            :class="{ active: currentProjectId === project.id }"
            @click="switchProject(project)"
          >
            <span class="project-icon">📁</span>
            <span class="project-name">{{ project.name }}</span>
            <button 
              class="btn btn-icon btn-delete" 
              @click.stop="removeProject(project.id)"
              title="删除项目"
            >
              ✕
            </button>
          </div>
          <div v-if="projects.length === 0" class="empty-state">
            点击 + 添加 Git 项目
          </div>
        </div>
        <!-- 拖拽调整宽度的手柄 -->
        <div 
          v-show="!isProjectSidebarCollapsed"
          class="resize-handle"
          @mousedown="startResizeProjectSidebar"
        ></div>
      </aside>

      <!-- 中间文件树 -->
      <aside 
        class="sidebar"
        :class="{ collapsed: isFileSidebarCollapsed }"
        :style="{ width: isFileSidebarCollapsed ? '40px' : fileSidebarWidth + 'px' }"
      >
        <div class="sidebar-header">
          <div class="sidebar-header-title">
            <h3 v-show="!isFileSidebarCollapsed">文件变更</h3>
            <button 
              class="btn btn-icon btn-collapse" 
              @click="toggleFileSidebar"
              :title="isFileSidebarCollapsed ? '展开' : '折叠'"
            >
              {{ isFileSidebarCollapsed ? '▶' : '◀' }}
            </button>
          </div>
          <div v-show="!isFileSidebarCollapsed" class="filter-tabs">
            <button 
              class="tab" 
              :class="{ active: viewMode === 'working' }"
              @click="viewMode = 'working'"
            >
              工作区
            </button>
            <button 
              class="tab" 
              :class="{ active: viewMode === 'staged' }"
              @click="viewMode = 'staged'"
            >
              暂存区
            </button>
          </div>
          <div class="view-toggle">
            <label class="toggle-label">
              <input 
                type="checkbox" 
                v-model="showAllFiles"
                @change="onShowAllFilesChange"
              />
              <span class="toggle-text">显示所有文件</span>
            </label>
          </div>
        </div>
        <div v-show="!isFileSidebarCollapsed" class="file-list">
          <FileTree
            v-if="fileTree.length > 0"
            :nodes="fileTree"
            @select="selectFile"
            @toggle="toggleDirectory"
          />
          <div v-else class="empty-state">
            选择左侧项目或点击"打开文件夹"
          </div>
        </div>
        <!-- 拖拽调整宽度的手柄 -->
        <div 
          v-show="!isFileSidebarCollapsed"
          class="resize-handle resize-handle-file"
          @mousedown="startResizeFileSidebar"
        ></div>
      </aside>

      <!-- 右侧比对区 -->
      <div class="diff-viewer">
        <div v-if="currentFile" class="file-info-bar">
          <div class="file-info">
            <span class="file-label">旧版本</span>
            <span class="file-path">{{ currentFile.path }}</span>
          </div>
          <div class="file-info">
            <span class="file-label">新版本</span>
            <span class="file-path">{{ currentFile.path }}</span>
          </div>
        </div>
        
        <div v-if="currentFile" class="diff-content">
          <div v-if="isBinary" class="binary-placeholder">
            [二进制文件]
          </div>
          <template v-else>
            <div class="diff-pane">
              <div class="pane-header">
                <span class="pane-title">{{ viewMode === 'working' ? '工作区' : '暂存区' }}</span>
              </div>
              <div class="code-content" ref="leftCodeContent" @scroll="syncScroll('left')">
                <DiffLines :lines="leftLines" />
              </div>
            </div>
            <div class="diff-divider"></div>
            <div class="diff-pane">
              <div class="pane-header">
                <span class="pane-title">HEAD</span>
              </div>
              <div class="code-content" ref="rightCodeContent" @scroll="syncScroll('right')">
                <DiffLines :lines="rightLines" />
              </div>
            </div>
            <!-- Minimap -->
            <Minimap
              :lines="leftLines"
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
    </div>

    <!-- 文件比对对话框 -->
    <div class="dialog" :class="{ open: showCompareFile }">
      <div class="dialog-content">
        <h3>选择要比对的文件</h3>
        <div class="text-inputs">
          <div class="text-input-group">
            <label>旧版本</label>
            <button class="btn btn-secondary" @click="selectOldFile">选择文件</button>
            <span v-if="compareOldPath" class="selected-path">{{ compareOldPath }}</span>
          </div>
          <div class="text-input-group">
            <label>新版本</label>
            <button class="btn btn-secondary" @click="selectNewFile">选择文件</button>
            <span v-if="compareNewPath" class="selected-path">{{ compareNewPath }}</span>
          </div>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-secondary" @click="showCompareFile = false">取消</button>
          <button class="btn btn-primary" @click="doFileCompare" :disabled="!compareOldPath || !compareNewPath">比对</button>
        </div>
      </div>
    </div>

    <!-- 文本比对对话框 -->
    <div class="dialog" :class="{ open: showTextCompare }">
      <div class="dialog-content">
        <h3>文本比对</h3>
        <div class="text-inputs">
          <div class="text-input-group">
            <label>文本 1</label>
            <textarea v-model="compareText1" placeholder="在此粘贴第一段文本..."></textarea>
          </div>
          <div class="text-input-group">
            <label>文本 2</label>
            <textarea v-model="compareText2" placeholder="在此粘贴第二段文本..."></textarea>
          </div>
        </div>
        <div class="dialog-actions">
          <button class="btn btn-secondary" @click="showTextCompare = false">取消</button>
          <button class="btn btn-primary" @click="doTextCompare">比对</button>
        </div>
      </div>
    </div>

    <!-- 添加项目对话框 -->
    <div class="dialog" :class="{ open: showAddProject }">
      <div class="dialog-content dialog-content-large">
        <h3>添加项目</h3>
        <div class="add-project-form">
          <div class="form-row">
            <div class="form-group form-group-name">
              <label>项目备注（可选）</label>
              <input v-model="newProjectName" placeholder="留空则使用文件夹名称" />
            </div>
            <button class="btn btn-primary" @click="selectProjectPath">选择文件夹</button>
          </div>
        </div>

        <!-- 待添加项目列表 -->
        <div v-if="pendingProjects.length > 0" class="pending-projects">
          <h4>待添加项目 ({{ pendingProjects.length }})</h4>
          <div class="pending-list">
            <div v-for="(item, index) in pendingProjects" :key="index" class="pending-item">
              <div class="pending-name-wrapper">
                <input
                  v-if="editingPendingIndex === index"
                  v-model="editingPendingName"
                  class="pending-name-input"
                  @blur="savePendingName(index)"
                  @keyup.enter="savePendingName(index)"
                  @keyup.esc="cancelEditPendingName"
                  ref="pendingNameInput"
                />
                <span
                  v-else
                  class="pending-name editable"
                  @click="startEditPendingName(index, item.name)"
                  title="点击修改备注"
                >{{ item.name }}</span>
              </div>
              <span class="pending-path">{{ item.path }}</span>
              <button class="btn btn-icon btn-remove" @click="removeFromPending(index)" title="删除">✕</button>
            </div>
          </div>
        </div>

        <div class="dialog-actions">
          <button class="btn btn-secondary" @click="closeAddProjectDialog">取消</button>
          <button class="btn btn-primary" @click="confirmAddProjects" :disabled="pendingProjects.length === 0">确定添加 ({{ pendingProjects.length }})</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import FileTree from './FileTree.vue';
import DiffLines from './DiffLines.vue';
import Minimap from './Minimap.vue';
import TabBar, { type Tab } from './TabBar.vue';

interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  status?: string;
  children: FileNode[];
  expanded?: boolean;
}

interface GitStatus {
  path: string;
  status: string;
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
  is_binary: boolean;
}

interface Project {
  id: string;
  name: string;
  path: string;
}

const theme = ref('dark');
const viewMode = ref<'working' | 'staged'>('working');
const showAllFiles = ref(true);
const showCompareFile = ref(false);
const showTextCompare = ref(false);
const fileTree = ref<FileNode[]>([]);
const currentFile = ref<FileNode | null>(null);
const currentPath = ref('');
const leftLines = ref<DiffLine[]>([]);
const rightLines = ref<DiffLine[]>([]);
const isBinary = ref(false);
const diffStats = ref<{ added: number; removed: number; changed: number } | null>(null);

// 标签页支持
const tabs = ref<Tab[]>([]);
const activeTabId = ref<string>('');

// 多项目支持
const projects = ref<Project[]>([]);
const currentProjectId = ref<string>('');
const showAddProject = ref(false);
const newProjectName = ref('');
const newProjectPath = ref('');
const pendingProjects = ref<{ name: string; path: string }[]>([]);
const editingPendingIndex = ref<number>(-1);
const editingPendingName = ref('');
const pendingNameInput = ref<HTMLInputElement | null>(null);

// 侧边栏宽度和折叠状态
const projectSidebarWidth = ref(200);
const fileSidebarWidth = ref(280);
const isProjectSidebarCollapsed = ref(false);
const isFileSidebarCollapsed = ref(false);
const isResizingProject = ref(false);
const isResizingFile = ref(false);

// 代码内容区域 refs，用于同步滚动
const leftCodeContent = ref<HTMLElement | null>(null);
const rightCodeContent = ref<HTMLElement | null>(null);
let isSyncing = false;

// Minimap 相关
const leftScrollTop = ref(0);
const codeContainerHeight = ref(0);
const codeContentHeight = ref(0);

const compareOldPath = ref('');
const compareNewPath = ref('');
const compareText1 = ref('');
const compareText2 = ref('');

let unlistenFileChange: (() => void) | null = null;

// 获取所有文件列表（扁平化）
const allFiles = computed(() => {
  const files: FileNode[] = [];
  const collect = (nodes: FileNode[]) => {
    nodes.forEach(node => {
      if (node.type === 'file') {
        files.push(node);
      }
      if (node.children) {
        collect(node.children);
      }
    });
  };
  collect(fileTree.value);
  return files;
});

// 当前文件索引
const currentFileIndex = computed(() => {
  if (!currentFile.value) return -1;
  return allFiles.value.findIndex(f => f.path === currentFile.value!.path);
});

// 是否有上一个
const hasPrev = computed(() => currentFileIndex.value > 0);

// 是否有下一个
const hasNext = computed(() => currentFileIndex.value < allFiles.value.length - 1);

onMounted(async () => {
  const savedTheme = localStorage.getItem('theme');
  if (savedTheme) {
    theme.value = savedTheme;
  }

  const savedShowAll = localStorage.getItem('showAllFiles');
  if (savedShowAll !== null) {
    showAllFiles.value = savedShowAll === 'true';
  }

  // 加载保存的项目列表
  loadProjects();

  unlistenFileChange = await listen('file-changed', (event) => {
    console.log('File changed event received:', event);
    if (currentPath.value) {
      console.log('Refreshing due to file change');
      refresh();
    }
  });
});

onUnmounted(() => {
  if (unlistenFileChange) {
    unlistenFileChange();
  }
});

const toggleTheme = () => {
  theme.value = theme.value === 'dark' ? 'light' : 'dark';
  localStorage.setItem('theme', theme.value);
};

const onShowAllFilesChange = () => {
  localStorage.setItem('showAllFiles', showAllFiles.value.toString());
  if (currentPath.value) {
    loadFileTree(currentPath.value);
  }
};

const openFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Git 仓库文件夹'
    });

    if (selected && typeof selected === 'string') {
      currentPath.value = selected;
      await loadFileTree(selected);
      await invoke('start_file_watcher', { repoPath: selected });
    }
  } catch (e) {
    console.error('Failed to open folder:', e);
    alert('打开文件夹失败: ' + e);
  }
};

// 项目相关方法
const showAddProjectDialog = () => {
  showAddProject.value = true;
  newProjectName.value = '';
  newProjectPath.value = '';
};

const selectProjectPath = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 Git 仓库文件夹'
    });

    if (selected && typeof selected === 'string') {
      // 检查是否已存在
      const exists = pendingProjects.value.some(p => p.path === selected);
      if (exists) {
        alert('该项目已添加到列表中');
        return;
      }

      // 获取项目名称
      let projectName = newProjectName.value.trim();
      if (!projectName) {
        const parts = selected.split('/');
        projectName = parts[parts.length - 1] || parts[parts.length - 2] || '新项目';
      }

      // 直接添加到列表
      pendingProjects.value.push({
        name: projectName,
        path: selected
      });

      // 清空备注输入，准备添加下一个
      newProjectName.value = '';
    }
  } catch (e) {
    console.error('Failed to select path:', e);
  }
};

// 批量添加项目相关方法
const addToPendingList = () => {
  if (!newProjectPath.value) return;

  // 如果没有输入备注名称，使用文件夹名
  let projectName = newProjectName.value.trim();
  if (!projectName) {
    const parts = newProjectPath.value.split('/');
    projectName = parts[parts.length - 1] || parts[parts.length - 2] || '新项目';
  }

  // 检查是否已存在
  const exists = pendingProjects.value.some(p => p.path === newProjectPath.value);
  if (exists) {
    alert('该项目已添加到列表中');
    return;
  }

  pendingProjects.value.push({
    name: projectName,
    path: newProjectPath.value
  });

  // 清空输入，准备添加下一个
  newProjectName.value = '';
  newProjectPath.value = '';
};

const removeFromPending = (index: number) => {
  pendingProjects.value.splice(index, 1);
  // 如果正在编辑的项目被删除，取消编辑状态
  if (editingPendingIndex.value === index) {
    editingPendingIndex.value = -1;
    editingPendingName.value = '';
  }
};

// 编辑待添加项目名称
const startEditPendingName = (index: number, name: string) => {
  editingPendingIndex.value = index;
  editingPendingName.value = name;
  // 下一个 tick 聚焦输入框
  setTimeout(() => {
    pendingNameInput.value?.focus();
    pendingNameInput.value?.select();
  }, 0);
};

const savePendingName = (index: number) => {
  if (editingPendingName.value.trim()) {
    pendingProjects.value[index].name = editingPendingName.value.trim();
  }
  editingPendingIndex.value = -1;
  editingPendingName.value = '';
};

const cancelEditPendingName = () => {
  editingPendingIndex.value = -1;
  editingPendingName.value = '';
};

const closeAddProjectDialog = () => {
  showAddProject.value = false;
  pendingProjects.value = [];
  newProjectName.value = '';
  newProjectPath.value = '';
};

const confirmAddProjects = async () => {
  if (pendingProjects.value.length === 0) return;

  // 批量添加项目
  for (const item of pendingProjects.value) {
    const project: Project = {
      id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
      name: item.name,
      path: item.path
    };
    projects.value.push(project);
  }

  saveProjects();

  // 切换到最后一个添加的项目
  const lastProject = projects.value[projects.value.length - 1];
  await switchProject(lastProject);

  // 关闭对话框并清空
  showAddProject.value = false;
  pendingProjects.value = [];
  newProjectName.value = '';
  newProjectPath.value = '';
};

const addProject = async () => {
  if (!newProjectPath.value) return;

  // 如果没有输入备注名称，使用文件夹名
  let projectName = newProjectName.value.trim();
  if (!projectName) {
    const parts = newProjectPath.value.split('/');
    projectName = parts[parts.length - 1] || parts[parts.length - 2] || '新项目';
  }

  const project: Project = {
    id: Date.now().toString(),
    name: projectName,
    path: newProjectPath.value
  };

  projects.value.push(project);
  saveProjects();

  // 自动切换到新项目
  await switchProject(project);

  showAddProject.value = false;
  newProjectName.value = '';
  newProjectPath.value = '';
};

const removeProject = (projectId: string) => {
  const index = projects.value.findIndex(p => p.id === projectId);
  if (index === -1) return;

  projects.value.splice(index, 1);
  saveProjects();

  // 如果删除的是当前项目，清空当前状态
  if (currentProjectId.value === projectId) {
    currentProjectId.value = '';
    currentPath.value = '';
    fileTree.value = [];
    currentFile.value = null;
  }
};

const switchProject = async (project: Project) => {
  currentProjectId.value = project.id;
  currentPath.value = project.path;
  await loadFileTree(project.path);
  await invoke('start_file_watcher', { repoPath: project.path });
};

const saveProjects = () => {
  localStorage.setItem('giter-projects', JSON.stringify(projects.value));
};

const loadProjects = () => {
  const saved = localStorage.getItem('giter-projects');
  if (saved) {
    try {
      projects.value = JSON.parse(saved);
    } catch (e) {
      console.error('Failed to load projects:', e);
    }
  }

  // 加载侧边栏状态
  const savedProjectWidth = localStorage.getItem('giter-project-sidebar-width');
  const savedFileWidth = localStorage.getItem('giter-file-sidebar-width');
  const savedProjectCollapsed = localStorage.getItem('giter-project-sidebar-collapsed');
  const savedFileCollapsed = localStorage.getItem('giter-file-sidebar-collapsed');

  if (savedProjectWidth) projectSidebarWidth.value = parseInt(savedProjectWidth);
  if (savedFileWidth) fileSidebarWidth.value = parseInt(savedFileWidth);
  if (savedProjectCollapsed) isProjectSidebarCollapsed.value = savedProjectCollapsed === 'true';
  if (savedFileCollapsed) isFileSidebarCollapsed.value = savedFileCollapsed === 'true';
};

// 侧边栏折叠/展开
const toggleProjectSidebar = () => {
  isProjectSidebarCollapsed.value = !isProjectSidebarCollapsed.value;
  localStorage.setItem('giter-project-sidebar-collapsed', isProjectSidebarCollapsed.value.toString());
};

const toggleFileSidebar = () => {
  isFileSidebarCollapsed.value = !isFileSidebarCollapsed.value;
  localStorage.setItem('giter-file-sidebar-collapsed', isFileSidebarCollapsed.value.toString());
};

// 拖拽调整宽度
const startResizeProjectSidebar = (e: MouseEvent) => {
  isResizingProject.value = true;
  const startX = e.clientX;
  const startWidth = projectSidebarWidth.value;

  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizingProject.value) return;
    const delta = e.clientX - startX;
    const newWidth = Math.max(150, Math.min(400, startWidth + delta));
    projectSidebarWidth.value = newWidth;
  };

  const handleMouseUp = () => {
    isResizingProject.value = false;
    localStorage.setItem('giter-project-sidebar-width', projectSidebarWidth.value.toString());
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  };

  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

const startResizeFileSidebar = (e: MouseEvent) => {
  isResizingFile.value = true;
  const startX = e.clientX;
  const startWidth = fileSidebarWidth.value;

  const handleMouseMove = (e: MouseEvent) => {
    if (!isResizingFile.value) return;
    const delta = e.clientX - startX;
    const newWidth = Math.max(200, Math.min(500, startWidth + delta));
    fileSidebarWidth.value = newWidth;
  };

  const handleMouseUp = () => {
    isResizingFile.value = false;
    localStorage.setItem('giter-file-sidebar-width', fileSidebarWidth.value.toString());
    document.removeEventListener('mousemove', handleMouseMove);
    document.removeEventListener('mouseup', handleMouseUp);
  };

  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
};

// 递归构建文件树
const buildFileTreeRecursive = async (
  entries: any[],
  basePath: string,
  changes: GitStatus[],
  parentPath: string = ''
): Promise<FileNode[]> => {
  const root: FileNode[] = [];

  // 创建更改状态的查找表
  const changeMap = new Map<string, string>();
  changes.forEach(c => changeMap.set(c.path, c.status));

  // 过滤掉隐藏文件和不需要的目录
  const filteredEntries = entries.filter(entry => {
    const name = entry.name;
    // 隐藏文件和目录
    if (name.startsWith('.')) return false;
    // node_modules
    if (name === 'node_modules') return false;
    // target 目录
    if (name === 'target') return false;
    return true;
  });

  // 排序：目录在前，文件在后
  filteredEntries.sort((a, b) => {
    const aIsDir = a.is_directory;
    const bIsDir = b.is_directory;
    if (aIsDir && !bIsDir) return -1;
    if (!aIsDir && bIsDir) return 1;
    return a.name.localeCompare(b.name);
  });

  for (const entry of filteredEntries) {
    const relativePath = parentPath ? `${parentPath}/${entry.name}` : entry.name;

    if (entry.is_directory) {
      // 递归读取子目录
      let children: FileNode[] = [];
      try {
        const subPath = `${basePath}/${relativePath}`;
        console.log('Reading subdirectory:', subPath);
        const subEntries = await invoke<any[]>('read_directory', { path: subPath });
        console.log('Subdirectory entries:', subEntries);
        children = await buildFileTreeRecursive(subEntries, basePath, changes, relativePath);
      } catch (e) {
        console.error(`Failed to read subdirectory: ${relativePath}`, e);
      }

      // 如果不显示所有文件，且子目录中没有变更文件，则跳过此目录
      if (!showAllFiles.value && children.length === 0) {
        continue;
      }

      root.push({
        name: entry.name,
        path: relativePath,
        type: 'directory',
        children: children,
        expanded: false
      });
    } else {
      // 检查是否有 Git 更改状态
      const status = changeMap.get(relativePath);

      // 如果不显示所有文件，且文件没有变更，则跳过
      if (!showAllFiles.value && !status) {
        continue;
      }

      root.push({
        name: entry.name,
        path: relativePath,
        type: 'file',
        status: status,
        children: []
      });
    }
  }

  return root;
};

const loadFileTree = async (path: string) => {
  try {
    // 读取文件夹内容
    const entries = await invoke<any[]>('read_directory', { path });

    // 获取 Git 更改状态
    let changes: GitStatus[] = [];
    try {
      changes = await invoke<GitStatus[]>('get_working_tree_changes', {
        repoPath: path
      });
    } catch (e) {
      // 如果不是 Git 仓库，忽略错误
      console.log('Not a git repository or error getting changes');
    }

    // 递归构建文件树
    fileTree.value = await buildFileTreeRecursive(entries, path, changes);
  } catch (e) {
    console.error('Failed to load file tree:', e);
    alert('加载文件树失败: ' + e);
  }
};

// 同步滚动函数
const syncScroll = (source: 'left' | 'right') => {
  if (isSyncing) return;
  isSyncing = true;

  const sourceEl = source === 'left' ? leftCodeContent.value : rightCodeContent.value;
  const targetEl = source === 'left' ? rightCodeContent.value : leftCodeContent.value;

  if (sourceEl && targetEl) {
    targetEl.scrollTop = sourceEl.scrollTop;
  }

  // 更新 minimap 的 scrollTop
  if (sourceEl) {
    leftScrollTop.value = sourceEl.scrollTop;
    codeContainerHeight.value = sourceEl.clientHeight;
    codeContentHeight.value = sourceEl.scrollHeight;
  }

  // 使用 requestAnimationFrame 确保同步
  requestAnimationFrame(() => {
    isSyncing = false;
  });
};

// 刷新 - 更新文件树和当前打开的标签
const refresh = async () => {
  if (currentPath.value) {
    await loadFileTree(currentPath.value);

    // 更新所有当前项目的标签
    for (const tab of tabs.value) {
      if (tab.projectPath === currentPath.value) {
        await updateTabFileStatus(tab.path);
      }
    }
  }
};

// Minimap 跳转处理
const handleMinimapJump = (lineIndex: number) => {
  if (!leftCodeContent.value) return;
  
  // 计算目标滚动位置
  const lineHeight = 24; // 每行高度
  const targetScrollTop = lineIndex * lineHeight;
  
  leftCodeContent.value.scrollTop = targetScrollTop;
  if (rightCodeContent.value) {
    rightCodeContent.value.scrollTop = targetScrollTop;
  }
  
  // 更新 minimap 状态
  leftScrollTop.value = targetScrollTop;
};

// 加载文件 diff 内容
const loadFileDiff = async (file: FileNode): Promise<{ leftLines: DiffLine[]; rightLines: DiffLine[]; isBinary: boolean; diffStats: any } | null> => {
  try {
    const workContent = await invoke<string>('read_file_content', {
      filePath: `${currentPath.value}/${file.path}`
    });

    // 根据文件状态选择比较方式
    let indexContent = '';
    const fileStatus = file.status?.toLowerCase();

    if (fileStatus === 'added') {
      indexContent = '';
    } else {
      try {
        indexContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: 'HEAD'
        });
      } catch (e) {
        indexContent = '';
      }
    }

    const isBinaryFile = workContent === '[二进制文件]' || indexContent === '[二进制文件]';

    if (isBinaryFile) {
      return { leftLines: [], rightLines: [], isBinary: true, diffStats: null };
    }

    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent: indexContent,
      newContent: workContent
    });

    // 构建对齐的行列表
    const alignedLeftLines: DiffLine[] = [];
    const alignedRightLines: DiffLine[] = [];

    let added = 0, removed = 0, changed = 0;
    let leftLineNum = 1;
    let rightLineNum = 1;

    diffResult.hunks.forEach(hunk => {
      // 首先添加未更改的行（上下文）
      for (let i = 0; i < Math.min(hunk.old_start - 1, hunk.new_start - 1); i++) {
        const oldContent = diffResult.old_content[i] || '';
        const newContent = diffResult.new_content[i] || '';

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

      // 处理 hunks 中的每一行
      let pendingRemoved: { content: string; lineNum: number } | null = null;

      hunk.lines.forEach(line => {
        if (line.change_type === 'removed') {
          pendingRemoved = {
            content: line.content,
            lineNum: rightLineNum++
          };
          removed++;
        } else if (line.change_type === 'added') {
          if (pendingRemoved) {
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
            changed++;
          } else {
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
            added++;
          }
        } else {
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
            pendingRemoved = null;
          }

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

      const finalPending = pendingRemoved;
      if (finalPending) {
        alignedLeftLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: finalPending.lineNum,
          content: finalPending.content,
          changeType: 'removed',
          isDiff: true
        });
      }
    });

    // 添加剩余的行
    const maxLines = Math.max(diffResult.old_content.length, diffResult.new_content.length);
    for (let i = alignedLeftLines.length; i < maxLines; i++) {
      const oldContent = diffResult.old_content[i] || '';
      const newContent = diffResult.new_content[i] || '';

      if (oldContent && !newContent) {
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
        removed++;
      } else if (!oldContent && newContent) {
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
        added++;
      } else if (oldContent && newContent) {
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

    return {
      leftLines: alignedLeftLines,
      rightLines: alignedRightLines,
      isBinary: false,
      diffStats: { added, removed, changed }
    };
  } catch (e) {
    console.error('Failed to load diff:', e);
    return null;
  }
};

// 选择文件 - 支持多标签
const selectFile = async (path: string) => {
  const findFile = (nodes: FileNode[]): FileNode | null => {
    for (const node of nodes) {
      if (node.path === path && node.type === 'file') {
        return node;
      }
      if (node.children) {
        const found = findFile(node.children);
        if (found) return found;
      }
    }
    return null;
  };

  const file = findFile(fileTree.value);
  if (!file) return;

  // 检查是否已有相同文件的标签
  const existingTab = tabs.value.find(tab => tab.path === path && tab.projectPath === currentPath.value);
  if (existingTab) {
    // 切换到已有标签
    await activateTab(existingTab.id);
    return;
  }

  // 创建新标签
  const fileExtension = path.split('.').pop() || '';
  const newTab: Tab = {
    id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
    name: file.name,
    path: file.path,
    projectPath: currentPath.value,
    fileType: fileExtension,
    isModified: false,
    leftLines: [],
    rightLines: [],
    isBinary: false,
    diffStats: null,
    scrollTop: 0
  };

  // 加载文件内容
  const diffResult = await loadFileDiff(file);
  if (diffResult) {
    newTab.leftLines = diffResult.leftLines;
    newTab.rightLines = diffResult.rightLines;
    newTab.isBinary = diffResult.isBinary;
    newTab.diffStats = diffResult.diffStats;
  }

  tabs.value.push(newTab);
  activeTabId.value = newTab.id;

  // 更新当前显示
  updateCurrentViewFromTab(newTab);
};

// 从标签更新当前视图
const updateCurrentViewFromTab = (tab: Tab) => {
  currentFile.value = {
    name: tab.name,
    path: tab.path,
    type: 'file',
    status: '',
    children: []
  };
  leftLines.value = tab.leftLines;
  rightLines.value = tab.rightLines;
  isBinary.value = tab.isBinary;
  diffStats.value = tab.diffStats;
};

// 激活标签
const activateTab = async (tabId: string) => {
  const tab = tabs.value.find(t => t.id === tabId);
  if (!tab) return;

  activeTabId.value = tabId;

  // 如果项目不同，切换项目
  if (tab.projectPath !== currentPath.value) {
    currentPath.value = tab.projectPath;
    await loadFileTree(tab.projectPath);
    await invoke('start_file_watcher', { repoPath: tab.projectPath });
  }

  updateCurrentViewFromTab(tab);
};

// 关闭标签
const closeTab = (tabId: string) => {
  const index = tabs.value.findIndex(t => t.id === tabId);
  if (index === -1) return;

  tabs.value.splice(index, 1);

  // 如果关闭的是当前标签，切换到其他标签
  if (activeTabId.value === tabId) {
    if (tabs.value.length > 0) {
      // 切换到前一个标签，如果没有则切换到第一个
      const newIndex = Math.min(index, tabs.value.length - 1);
      const newTab = tabs.value[newIndex];
      activateTab(newTab.id);
    } else {
      // 没有标签了，清空视图
      activeTabId.value = '';
      currentFile.value = null;
      leftLines.value = [];
      rightLines.value = [];
      isBinary.value = false;
      diffStats.value = null;
    }
  }
};

// 关闭所有标签
const closeAllTabs = () => {
  tabs.value = [];
  activeTabId.value = '';
  currentFile.value = null;
  leftLines.value = [];
  rightLines.value = [];
  isBinary.value = false;
  diffStats.value = null;
};

// 关闭其他标签
const closeOtherTabs = (tabId: string) => {
  const tab = tabs.value.find(t => t.id === tabId);
  if (!tab) return;

  tabs.value = [tab];
  activeTabId.value = tabId;
  updateCurrentViewFromTab(tab);
};

// 关闭右侧标签
const closeTabsToRight = (tabId: string) => {
  const index = tabs.value.findIndex(t => t.id === tabId);
  if (index === -1) return;

  tabs.value = tabs.value.slice(0, index + 1);

  // 如果当前标签被关闭了，激活最后一个
  if (!tabs.value.find(t => t.id === activeTabId.value)) {
    const lastTab = tabs.value[tabs.value.length - 1];
    if (lastTab) {
      activateTab(lastTab.id);
    }
  }
};

// 更新标签的文件状态（用于文件变更检测）
const updateTabFileStatus = async (filePath: string) => {
  const tab = tabs.value.find(t => t.path === filePath && t.projectPath === currentPath.value);
  if (!tab) return;

  // 重新加载文件内容
  const fileNode: FileNode = {
    name: tab.name,
    path: tab.path,
    type: 'file',
    status: '',
    children: []
  };

  const diffResult = await loadFileDiff(fileNode);
  if (diffResult) {
    tab.leftLines = diffResult.leftLines;
    tab.rightLines = diffResult.rightLines;
    tab.isBinary = diffResult.isBinary;
    tab.diffStats = diffResult.diffStats;
    tab.isModified = true;

    // 如果是当前标签，更新视图
    if (tab.id === activeTabId.value) {
      leftLines.value = tab.leftLines;
      rightLines.value = tab.rightLines;
      isBinary.value = tab.isBinary;
      diffStats.value = tab.diffStats;
    }
  }
};

const toggleDirectory = (node: FileNode) => {
  node.expanded = !node.expanded;
};

const navigatePrev = () => {
  if (hasPrev.value) {
    const prevFile = allFiles.value[currentFileIndex.value - 1];
    selectFile(prevFile.path);
  }
};

const navigateNext = () => {
  if (hasNext.value) {
    const nextFile = allFiles.value[currentFileIndex.value + 1];
    selectFile(nextFile.path);
  }
};

const showCompareFileDialog = () => {
  compareOldPath.value = '';
  compareNewPath.value = '';
  showCompareFile.value = true;
};

const selectOldFile = async () => {
  const selected = await open({
    multiple: false,
    title: '选择旧版本文件'
  });
  if (selected && typeof selected === 'string') {
    compareOldPath.value = selected;
  }
};

const selectNewFile = async () => {
  const selected = await open({
    multiple: false,
    title: '选择新版本文件'
  });
  if (selected && typeof selected === 'string') {
    compareNewPath.value = selected;
  }
};

const doFileCompare = async () => {
  try {
    const oldContent = await invoke<string>('read_file_content', {
      filePath: compareOldPath.value
    });

    const newContent = await invoke<string>('read_file_content', {
      filePath: compareNewPath.value
    });

    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent,
      newContent
    });

    leftLines.value = diffResult.new_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    rightLines.value = diffResult.old_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    diffResult.hunks.forEach(hunk => {
      hunk.lines.forEach(line => {
        if (line.change_type === 'added') {
          const leftLine = leftLines.value.find(l => l.lineNum === line.line_number);
          if (leftLine) {
            leftLine.changeType = 'added';
            leftLine.isDiff = true;
          }
        } else if (line.change_type === 'removed') {
          const rightLine = rightLines.value.find(l => l.lineNum === line.line_number);
          if (rightLine) {
            rightLine.changeType = 'removed';
            rightLine.isDiff = true;
          }
        }
      });
    });

    currentFile.value = {
      name: compareNewPath.value.split('/').pop() || '',
      path: compareNewPath.value,
      type: 'file',
      status: 'Modified',
      children: []
    };

    isBinary.value = false;
    showCompareFile.value = false;
  } catch (e) {
    console.error('Failed to compare files:', e);
    alert('文件比对失败: ' + e);
  }
};

const doTextCompare = async () => {
  try {
    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent: compareText1.value,
      newContent: compareText2.value
    });

    leftLines.value = diffResult.new_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    rightLines.value = diffResult.old_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    diffResult.hunks.forEach(hunk => {
      hunk.lines.forEach(line => {
        if (line.change_type === 'added') {
          const leftLine = leftLines.value.find(l => l.lineNum === line.line_number);
          if (leftLine) {
            leftLine.changeType = 'added';
            leftLine.isDiff = true;
          }
        } else if (line.change_type === 'removed') {
          const rightLine = rightLines.value.find(l => l.lineNum === line.line_number);
          if (rightLine) {
            rightLine.changeType = 'removed';
            rightLine.isDiff = true;
          }
        }
      });
    });

    currentFile.value = {
      name: '文本比对结果',
      path: 'text-compare',
      type: 'file',
      status: 'Modified',
      children: []
    };

    isBinary.value = false;
    showTextCompare.value = false;
  } catch (e) {
    console.error('Failed to compare text:', e);
    alert('文本比对失败: ' + e);
  }
};
</script>
