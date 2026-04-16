<template>
  <div class="app">
    <!-- 工具栏 -->
    <Toolbar
      :theme="theme"
      :current-path="currentPath"
      :has-prev="hasPrev"
      :has-next="hasNext"
      @compare-file="showCompareFileDialog"
      @compare-text="showTextCompare = true"
      @toggle-theme="toggleTheme"
      @navigate-prev="navigatePrev"
      @navigate-next="navigateNext"
      @refresh="handleRefresh"
      @manage-plugins="showPluginManager = true"
      @manage-workspace="showWorkspaceManager = true"
    />

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
      <ProjectSidebar
        :workspaces="workspaces"
        :current-workspace-id="currentWorkspaceId"
        :projects="projects"
        :current-project-id="currentProjectId"
        :is-collapsed="isProjectSidebarCollapsed"
        :width="projectSidebarWidth"
        @add-project="showAddProjectDialog"
        @add-workspace="addWorkspace"
        @toggle-collapse="toggleProjectSidebar"
        @switch-project="switchProject"
        @remove-project="removeProject"
        @start-resize="startResizeProjectSidebar"
        @export-projects="exportProjects"
        @import-projects="importProjects"
        @switch-workspace="switchWorkspace"
      />

      <!-- 中间文件树 -->
      <FileTreeSidebar
        :file-tree="fileTree"
        :view-mode="viewMode"
        :show-all-files="showAllFiles"
        :show-deleted-files="showDeletedFiles"
        :git-changes="gitChanges"
        :is-collapsed="isFileSidebarCollapsed"
        :width="fileSidebarWidth"
        :staged-files="stagedFiles"
        :selected-staged-path="selectedStagedPath"
        @update:view-mode="onViewModeChange"
        @update:show-all-files="onShowAllFilesChange"
        @update:show-deleted-files="onShowDeletedFilesChange"
        @toggle-collapse="toggleFileSidebar"
        @select-file="selectFile"
        @toggle-directory="toggleDirectory"
        @select-staged-file="selectStagedFile"
        @start-resize="startResizeFileSidebar"
      />

      <!-- 右侧比对区 -->
      <DiffViewer
        :current-file="currentFile"
        :left-lines="leftLines"
        :right-lines="rightLines"
        :is-binary="isBinary"
        :diff-stats="diffStats"
        :view-mode="viewMode"
        :theme="theme"
        @scroll="handleScroll"
      />
    </div>

    <!-- 文件比对对话框 -->
    <FileCompareDialog
      :open="showCompareFile"
      :old-path="compareOldPath"
      :new-path="compareNewPath"
      @close="showCompareFile = false"
      @select-old="selectOldFile"
      @select-new="selectNewFile"
      @compare="doFileCompare"
    />

    <!-- 文本比对对话框 -->
    <TextCompareDialog
      :open="showTextCompare"
      v-model:text1="compareText1"
      v-model:text2="compareText2"
      @close="showTextCompare = false"
      @compare="doTextCompare"
    />

    <!-- 添加项目对话框 -->
    <AddProjectDialog
      :open="showAddProject"
      v-model:project-name="newProjectName"
      :pending-projects="pendingProjects"
      :editing-index="editingPendingIndex"
      v-model:editing-name="editingPendingName"
      :is-loading="isAddingProjects"
      @close="closeAddProjectDialog"
      @select-path="selectProjectPath"
      @remove="removeFromPending"
      @confirm="confirmAddProjects"
      @start-edit="startEditPendingName"
      @save-name="savePendingName"
      @cancel-edit="cancelEditPendingName"
    />

    <!-- 插件管理对话框 -->
    <PluginManagerDialog
      :open="showPluginManager"
      @close="showPluginManager = false"
      @plugins-changed="onPluginsChanged"
    />

    <!-- 工作区管理对话框 -->
    <WorkspaceManagerDialog
      :open="showWorkspaceManager"
      :current-projects="projects"
      @close="showWorkspaceManager = false"
      @load-workspace="onLoadWorkspace"
    />

    <!-- 权限提示弹窗 -->
    <div v-if="showPermissionDialog" class="permission-overlay" @click.self="showPermissionDialog = false">
      <div class="permission-dialog">
        <div class="permission-header">
          <h3>需要磁盘访问权限</h3>
          <button class="close-btn" @click="showPermissionDialog = false">×</button>
        </div>
        <div class="permission-content">
          <p>导出项目列表需要授予应用磁盘访问权限。</p>
          <p>请按照以下步骤操作：</p>
          <ol>
            <li>打开 <strong>系统设置</strong> → <strong>隐私与安全性</strong> → <strong>完全磁盘访问权限</strong></li>
            <li>点击 <strong>+</strong> 按钮</li>
            <li>找到并添加 <strong>git-compare-tool</strong> 应用</li>
            <li>重启应用</li>
          </ol>
        </div>
        <div class="permission-actions">
          <button class="btn btn-primary" @click="openSystemSettings">打开系统设置</button>
          <button class="btn btn-secondary" @click="showPermissionDialog = false">稍后再说</button>
        </div>
      </div>
    </div>

    <!-- 通用输入对话框 -->
    <PromptDialog
      :open="showPromptDialog"
      :title="promptTitle"
      :message="promptMessage"
      v-model:default-value="promptValue"
      @confirm="handleCreateWorkspace"
      @cancel="showPromptDialog = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import Toolbar from './Toolbar.vue';
import ProjectSidebar from './ProjectSidebar.vue';
import FileTreeSidebar from './FileTreeSidebar.vue';
import DiffViewer from './DiffViewer.vue';
import TabBar, { type Tab } from './TabBar.vue';
import { FileCompareDialog, TextCompareDialog, AddProjectDialog, PluginManagerDialog, WorkspaceManagerDialog, PromptDialog } from './dialogs';

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

interface Workspace {
  id: string;
  name: string;
  projects: Project[];
}

interface Workspace {
  id: string;
  name: string;
  projects: Project[];
}

// 主题和视图状态
const theme = ref('light');
const viewMode = ref<'working' | 'staged'>('working');
const showAllFiles = ref(true);
const showDeletedFiles = ref(false);

// 对话框状态
const showCompareFile = ref(false);
const showTextCompare = ref(false);
const showAddProject = ref(false);
const showPluginManager = ref(false);
const showWorkspaceManager = ref(false);
const showPermissionDialog = ref(false);
const showPromptDialog = ref(false);
const promptTitle = ref('');
const promptMessage = ref('');
const promptValue = ref('');

// 调试：监听 showPluginManager 变化
watch(showPluginManager, (newVal, oldVal) => {
  console.log('showPluginManager changed:', oldVal, '->', newVal);
  console.trace('Stack trace:');
});

// 文件和数据状态
const fileTree = ref<FileNode[]>([]);
const currentFile = ref<FileNode | null>(null);
const currentPath = ref('');
const leftLines = ref<DiffLine[]>([]);
const rightLines = ref<DiffLine[]>([]);
const isBinary = ref(false);
const diffStats = ref<{ added: number; removed: number; changed: number } | null>(null);
const gitChanges = ref<GitStatus[]>([]);

// 文件树缓存 - 键为项目路径，值为文件树数据（永久缓存，文件变更时自动刷新）
const fileTreeCache = ref<Map<string, { tree: FileNode[]; changes: GitStatus[]; timestamp: number }>>(new Map());

// Diff 缓存 - 键为"项目路径:文件路径"，值为 diff 结果（永久缓存，文件变更时自动刷新）
const diffCache = ref<Map<string, { leftLines: DiffLine[]; rightLines: DiffLine[]; isBinary: boolean; diffStats: any; timestamp: number }>>(new Map());

// 暂存区状态
const stagedFiles = ref<{ name: string; path: string; status?: string }[]>([]);
const selectedStagedPath = ref<string>('');

// 标签页支持
const tabs = ref<Tab[]>([]);
const activeTabId = ref<string>('');

// 多项目支持 - 使用工作区（文件夹）管理
const workspaces = ref<Workspace[]>([]);
const currentWorkspaceId = ref<string>('');
const projects = ref<Project[]>([]); // 当前工作区的项目列表
const currentProjectId = ref<string>('');
const newProjectName = ref('');
const newProjectPath = ref('');
const pendingProjects = ref<{ name: string; path: string }[]>([]);
const editingPendingIndex = ref<number>(-1);
const editingPendingName = ref('');
const isAddingProjects = ref(false); // 防抖状态：是否正在添加项目中

// 侧边栏宽度和折叠状态
const projectSidebarWidth = ref(280);
const fileSidebarWidth = ref(280);
const isProjectSidebarCollapsed = ref(false);
const isFileSidebarCollapsed = ref(false);
const isResizingProject = ref(false);
const isResizingFile = ref(false);

// 比对相关
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

// 是否有上一个/下一个
const hasPrev = computed(() => currentFileIndex.value > 0);
const hasNext = computed(() => currentFileIndex.value < allFiles.value.length - 1);

onMounted(async () => {
  const savedTheme = localStorage.getItem('theme');
  theme.value = savedTheme || 'light';
  // 初始化时设置 body 的 data-theme 属性
  document.body.setAttribute('data-theme', theme.value);

  const savedShowAll = localStorage.getItem('showAllFiles');
  if (savedShowAll !== null) {
    showAllFiles.value = savedShowAll === 'true';
  }

  const savedShowDeleted = localStorage.getItem('showDeletedFiles');
  if (savedShowDeleted !== null) {
    showDeletedFiles.value = savedShowDeleted === 'true';
  }

  loadWorkspaces();

  unlistenFileChange = await listen('file-changed', (event: any) => {
    if (currentPath.value) {
      // 检查是否是结构变化（新增/删除文件或文件夹）
      const payload = event.payload;
      const isStructuralChange = payload?.is_structural_change === true;
      const changedFilePath = payload?.path;
      
      if (isStructuralChange) {
        // 文件结构变化，清除当前项目的文件树缓存
        clearFileTreeCache(currentPath.value);
      }
      
      // 清除变更文件的 diff 缓存
      if (changedFilePath) {
        const diffCacheKey = getDiffCacheKey(currentPath.value, changedFilePath);
        if (diffCache.value.has(diffCacheKey)) {
          diffCache.value.delete(diffCacheKey);
          console.log('Cleared diff cache for changed file:', changedFilePath);
        }
      }
      
      refresh();
    }
  });
});

onUnmounted(() => {
  if (unlistenFileChange) {
    unlistenFileChange();
  }
});

// 主题切换
const toggleTheme = () => {
  theme.value = theme.value === 'dark' ? 'light' : 'dark';
  localStorage.setItem('theme', theme.value);
  // 设置 body 的 data-theme 属性，让对话框也能继承主题
  document.body.setAttribute('data-theme', theme.value);
};

// 显示所有文件切换
const onShowAllFilesChange = (value: boolean) => {
  showAllFiles.value = value;
  localStorage.setItem('showAllFiles', showAllFiles.value.toString());
  if (currentPath.value) {
    loadFileTree(currentPath.value);
  }
};

// 显示已删除文件切换
const onShowDeletedFilesChange = (value: boolean) => {
  showDeletedFiles.value = value;
  localStorage.setItem('showDeletedFiles', showDeletedFiles.value.toString());
  if (currentPath.value) {
    loadFileTree(currentPath.value);
    // 如果勾选显示已删除文件，展开所有目录
    if (value) {
      expandAllDirectories(fileTree.value);
    }
  }
};

// 展开所有目录
const expandAllDirectories = (nodes: FileNode[]) => {
  for (const node of nodes) {
    if (node.type === 'directory') {
      node.expanded = true;
      if (node.children && node.children.length > 0) {
        expandAllDirectories(node.children);
      }
    }
  }
};

// 视图模式切换
const onViewModeChange = async (mode: 'working' | 'staged') => {
  viewMode.value = mode;
  if (mode === 'staged') {
    await loadStagedFiles();
  }
};

// 选择暂存区文件
const selectStagedFile = async (path: string) => {
  selectedStagedPath.value = path;

  const file = stagedFiles.value.find(f => f.path === path);
  if (!file) return;

  // 创建文件节点
  const fileNode: FileNode = {
    name: file.name,
    path: file.path,
    type: 'file',
    status: file.status,
    children: []
  };

  // 加载暂存区文件的 diff
  await loadStagedFileDiff(fileNode);
};

// 加载更改文件的 diff（工作区中的变更）
const loadStagedFileDiff = async (file: FileNode) => {
  try {
    const fileStatus = file.status?.toLowerCase();
    let workContent = '';
    let headContent = '';

    if (fileStatus === 'deleted') {
      // 已删除的文件：工作区内容为空，从 Git 获取删除前的内容
      workContent = '';
      try {
        headContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: 'HEAD'
        });
      } catch (e) {
        headContent = '';
      }
    } else if (fileStatus === 'added') {
      // 新增的文件：HEAD 中不存在，读取工作区内容
      workContent = await invoke<string>('read_file_content', {
        filePath: `${currentPath.value}/${file.path}`
      });
      headContent = '';
    } else {
      // 修改的文件：读取工作区内容和 HEAD 版本
      workContent = await invoke<string>('read_file_content', {
        filePath: `${currentPath.value}/${file.path}`
      });
      try {
        headContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: 'HEAD'
        });
      } catch (e) {
        headContent = '';
      }
    }

    const isBinaryFile = workContent === '[二进制文件]' || headContent === '[二进制文件]';

    if (isBinaryFile) {
      currentFile.value = file;
      leftLines.value = [];
      rightLines.value = [];
      isBinary.value = true;
      diffStats.value = null;
      return;
    }

    // 对于已删除的文件，直接显示完整的旧文件内容
    if (fileStatus === 'deleted') {
      const alignedLeftLines: DiffLine[] = [];
      const alignedRightLines: DiffLine[] = [];
      const oldLines = headContent.split('\n');

      for (let i = 0; i < oldLines.length; i++) {
        alignedLeftLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: i + 1,
          content: oldLines[i],
          changeType: 'removed',
          isDiff: true
        });
      }

      leftLines.value = alignedLeftLines;
      rightLines.value = alignedRightLines;
      isBinary.value = false;
      diffStats.value = { added: 0, removed: oldLines.length, changed: 0 };
      currentFile.value = file;
      return;
    }

    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent: headContent,
      newContent: workContent
    });

    // 构建 diff 行（复用 loadFileDiff 的逻辑）
    const result = buildDiffLines(diffResult);

    leftLines.value = result.leftLines;
    rightLines.value = result.rightLines;
    isBinary.value = false;
    diffStats.value = result.diffStats;
    currentFile.value = file;
  } catch (e) {
    console.error('Failed to load changed file diff:', e);
  }
};

// 构建 diff 行的辅助函数
const buildDiffLines = (diffResult: FileDiff) => {
  const alignedLeftLines: DiffLine[] = [];
  const alignedRightLines: DiffLine[] = [];

  let added = 0, removed = 0, changed = 0;
  let leftLineNum = 1;
  let rightLineNum = 1;

  diffResult.hunks.forEach(hunk => {
    // 添加上下文行 - hunk.old_start 和 hunk.new_start 是 1-based 行号
    // 需要添加从当前位置到 hunk 开始前的所有行
    const contextStart = Math.min(hunk.old_start, hunk.new_start) - 1;
    for (let i = alignedLeftLines.length; i < contextStart; i++) {
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

    let pendingRemoved: { content: string; lineNum: number } | null = null;

    hunk.lines.forEach(line => {
      if (line.change_type === 'removed') {
        pendingRemoved = { content: line.content, lineNum: rightLineNum++ };
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

  // 添加剩余行
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
    diffStats: { added, removed, changed }
  };
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
      const exists = pendingProjects.value.some(p => p.path === selected);
      if (exists) {
        alert('该项目已添加到列表中');
        return;
      }

      let projectName = newProjectName.value.trim();
      console.log('Input projectName:', JSON.stringify(newProjectName.value), 'trimmed:', JSON.stringify(projectName));
      if (!projectName) {
        const parts = selected.split(/[\\/]/);
        projectName = parts[parts.length - 1] || parts[parts.length - 2] || '新项目';
        console.log('Using folder name:', projectName);
      } else {
        console.log('Using input name:', projectName);
      }

      pendingProjects.value.push({ name: projectName, path: selected });
      newProjectName.value = '';
    }
  } catch (e) {
    console.error('Failed to select path:', e);
  }
};

const removeFromPending = (index: number) => {
  pendingProjects.value.splice(index, 1);
  if (editingPendingIndex.value === index) {
    editingPendingIndex.value = -1;
    editingPendingName.value = '';
  }
};

const startEditPendingName = (index: number, name: string) => {
  editingPendingIndex.value = index;
  editingPendingName.value = name;
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
  editingPendingIndex.value = -1;
  editingPendingName.value = '';
};

const confirmAddProjects = async () => {
  // 防抖：如果正在添加中，直接返回
  if (isAddingProjects.value || pendingProjects.value.length === 0) return;

  isAddingProjects.value = true;

  try {
    const newProjects: Project[] = [];
    
    // 第一步：创建项目对象
    for (const item of pendingProjects.value) {
      console.log('Adding project:', item.name, item.path);
      const project: Project = {
        id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
        name: item.name,
        path: item.path
      };
      newProjects.push(project);
      projects.value.push(project);
    }

    saveWorkspaces();

    // 第二步：预加载所有新项目的文件树到缓存（在后台进行）
    console.log('Preloading file tree cache for', newProjects.length, 'projects...');
    const preloadPromises = newProjects.map(async (project) => {
      try {
        // 检查是否已有缓存
        if (!fileTreeCache.value.has(project.path)) {
          console.log('Preloading cache for:', project.path);
          await loadFileTree(project.path);
        } else {
          console.log('Cache already exists for:', project.path);
        }
      } catch (e) {
        console.error('Failed to preload cache for:', project.path, e);
      }
    });
    
    // 等待所有预加载完成
    await Promise.all(preloadPromises);
    console.log('All projects preloaded');

    // 第三步：切换到最后一个添加的项目
    const lastProject = newProjects[newProjects.length - 1];
    await switchProject(lastProject);

    closeAddProjectDialog();
  } finally {
    // 延迟重置防抖状态，防止快速连续点击
    setTimeout(() => {
      isAddingProjects.value = false;
    }, 500);
  }
};

const removeProject = (projectId: string) => {
  const index = projects.value.findIndex(p => p.id === projectId);
  if (index === -1) return;

  const projectPath = projects.value[index].path;

  projects.value.splice(index, 1);
  saveWorkspaces();

  // 清理被删除项目的缓存
  clearFileTreeCache(projectPath);
  clearDiffCache(projectPath);

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
  
  // 清除更改列表，避免显示上一个项目的更改
  stagedFiles.value = [];
  selectedStagedPath.value = '';
  
  await loadFileTree(project.path);
  await invoke('start_file_watcher', { repoPath: project.path });
};

// 保存工作区列表到 localStorage
const saveWorkspaces = () => {
  localStorage.setItem('giter-workspaces', JSON.stringify(workspaces.value));
  localStorage.setItem('giter-current-workspace', currentWorkspaceId.value);
};

// 加载工作区列表
const loadWorkspaces = () => {
  const saved = localStorage.getItem('giter-workspaces');
  if (saved) {
    try {
      workspaces.value = JSON.parse(saved);
      // 加载当前工作区 ID
      const savedCurrent = localStorage.getItem('giter-current-workspace');
      if (savedCurrent && workspaces.value.length > 0) {
        const found = workspaces.value.find(w => w.id === savedCurrent);
        if (found) {
          currentWorkspaceId.value = savedCurrent;
          projects.value = found.projects;
        } else {
          // 默认选择第一个工作区
          currentWorkspaceId.value = workspaces.value[0].id;
          projects.value = workspaces.value[0].projects;
        }
      } else if (workspaces.value.length > 0) {
        currentWorkspaceId.value = workspaces.value[0].id;
        projects.value = workspaces.value[0].projects;
      }
    } catch (e) {
      console.error('Failed to load workspaces:', e);
      // 兼容旧数据
      loadProjects();
    }
  } else {
    // 兼容旧数据格式
    loadProjects();
    
    // 如果没有工作区也没有旧数据，创建默认工作区
    if (workspaces.value.length === 0) {
      const defaultWorkspace: Workspace = {
        id: 'default',
        name: '默认工作区',
        projects: []
      };
      workspaces.value = [defaultWorkspace];
      currentWorkspaceId.value = 'default';
      projects.value = [];
      saveWorkspaces();
    }
  }
};

// 兼容旧版本的单项目列表加载
const loadProjects = () => {
  const saved = localStorage.getItem('giter-projects');
  if (saved) {
    try {
      const projectsData: Project[] = JSON.parse(saved);
      if (projectsData.length > 0) {
        // 迁移到工作区格式
        const defaultWorkspace: Workspace = {
          id: 'default',
          name: '默认工作区',
          projects: projectsData
        };
        workspaces.value = [defaultWorkspace];
        currentWorkspaceId.value = 'default';
        projects.value = projectsData;
        saveWorkspaces();
        // 清除旧数据
        localStorage.removeItem('giter-projects');
      }
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

// 导出项目列表为 JSON 文件
const exportProjects = async () => {
  let filePath: string | null = null;
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');

    filePath = await save({
      filters: [
        { name: 'JSON', extensions: ['json'] }
      ],
      defaultPath: 'giter-projects.json'
    });

    if (filePath) {
      const exportData = {
        version: '1.0',
        exportDate: new Date().toISOString(),
        projects: projects.value
      };
      await writeTextFile(filePath, JSON.stringify(exportData, null, 2));
      alert('项目列表导出成功！');
      return; // 成功导出后直接返回
    }
  } catch (e: any) {
    console.error('Failed to export projects:', e);
    console.log('Error details:', e.toString());
    console.log('File path:', filePath);
    // 用户取消保存对话框，不显示错误
    if (!filePath) {
      return;
    }
    const errorMsg = e.toString();
    if (errorMsg.includes('Operation not permitted') || errorMsg.includes('os error 1')) {
      // 显示权限提示 - 使用自定义弹窗而不是 confirm
      showPermissionDialog.value = true;
    } else {
      alert('导出失败: ' + e);
    }
  }
};

// 从 JSON 文件导入项目列表
const importProjects = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const { readTextFile } = await import('@tauri-apps/plugin-fs');

    const selected = await open({
      filters: [
        { name: 'JSON', extensions: ['json'] }
      ],
      multiple: false
    });

    if (selected && typeof selected === 'string') {
      const content = await readTextFile(selected);
      const importData = JSON.parse(content);

      if (importData.projects && Array.isArray(importData.projects)) {
        // 合并导入的项目，避免重复
        const existingPaths = new Set(projects.value.map(p => p.path));
        const newProjects: Project[] = [];
        let addedCount = 0;

        for (const project of importData.projects) {
          if (!existingPaths.has(project.path)) {
            const newProject: Project = {
              id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
              name: project.name,
              path: project.path
            };
            projects.value.push(newProject);
            newProjects.push(newProject);
            addedCount++;
          }
        }

        saveWorkspaces();

        // 预加载所有新导入项目的缓存
        if (newProjects.length > 0) {
          console.log('Preloading cache for imported projects:', newProjects.length);
          const preloadPromises = newProjects.map(async (project) => {
            try {
              if (!fileTreeCache.value.has(project.path)) {
                console.log('Preloading cache for imported:', project.path);
                await loadFileTree(project.path);
              }
            } catch (e) {
              console.error('Failed to preload cache for imported project:', project.path, e);
            }
          });
          await Promise.all(preloadPromises);
          console.log('All imported projects preloaded');
        }

        alert(`成功导入 ${addedCount} 个项目！`);
      } else {
        alert('无效的项目文件格式');
      }
    }
  } catch (e) {
    console.error('Failed to import projects:', e);
    alert('导入失败：' + e);
  }
};

// 添加工作区
const addWorkspace = () => {
  console.log('Add workspace button clicked');
  promptTitle.value = '新建工作区';
  promptMessage.value = '请输入工作区名称：';
  promptValue.value = '';
  showPromptDialog.value = true;
};

// 处理工作区创建确认
const handleCreateWorkspace = (name: string) => {
  const newWorkspace: Workspace = {
    id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
    name,
    projects: []
  };

  workspaces.value.push(newWorkspace);
  currentWorkspaceId.value = newWorkspace.id;
  projects.value = [];
  saveWorkspaces();
  
  // 关闭对话框
  showPromptDialog.value = false;
  
  console.log('Workspace created:', newWorkspace.name);
};

// 切换工作区
const switchWorkspace = (workspaceId: string) => {
  const workspace = workspaces.value.find(w => w.id === workspaceId);
  if (!workspace) return;

  currentWorkspaceId.value = workspaceId;
  projects.value = workspace.projects;
  currentProjectId.value = '';
  currentPath.value = '';
  fileTree.value = [];
  currentFile.value = null;
  saveWorkspaces();
};

// 删除工作区
const removeWorkspace = (workspaceId: string) => {
  const index = workspaces.value.findIndex(w => w.id === workspaceId);
  if (index === -1) return;

  if (!confirm(`确定要删除工作区 "${workspaces.value[index].name}" 吗？此操作不可恢复。`)) {
    return;
  }

  workspaces.value.splice(index, 1);

  // 如果删除的是当前工作区，切换到第一个工作区
  if (currentWorkspaceId.value === workspaceId) {
    if (workspaces.value.length > 0) {
      currentWorkspaceId.value = workspaces.value[0].id;
      projects.value = workspaces.value[0].projects;
    } else {
      currentWorkspaceId.value = '';
      projects.value = [];
    }
  }

  saveWorkspaces();
};

// 打开系统设置
const openSystemSettings = async () => {
  try {
    await invoke('open_system_settings');
  } catch (e) {
    console.error('Failed to open system settings:', e);
    // 如果命令失败，尝试用浏览器打开设置页面
    window.open('x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles', '_blank');
  }
};

// 加载工作区
const onLoadWorkspace = (workspaceProjects: Project[]) => {
  // 合并工作区项目到现有项目，避免重复
  const existingPaths = new Set(projects.value.map(p => p.path));

  for (const project of workspaceProjects) {
    if (!existingPaths.has(project.path)) {
      projects.value.push({
        id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
        name: project.name,
        path: project.path
      });
    }
  }

  saveWorkspaces();
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
  const changeMap = new Map<string, string>();
  changes.forEach(c => changeMap.set(c.path, c.status));

  const filteredEntries = entries.filter(entry => {
    const name = entry.name;
    if (name.startsWith('.')) return false;
    if (name === 'node_modules') return false;
    if (name === 'target') return false;
    return true;
  });

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
      let children: FileNode[] = [];
      try {
        const subPath = `${basePath}/${relativePath}`;
        const subEntries = await invoke<any[]>('read_directory', { path: subPath });
        children = await buildFileTreeRecursive(subEntries, basePath, changes, relativePath);
      } catch (e) {
        console.error(`Failed to read subdirectory: ${relativePath}`, e);
      }

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
      const status = changeMap.get(relativePath);

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

// 获取缓存的文件树
const getCachedFileTree = (path: string): { tree: FileNode[]; changes: GitStatus[] } | null => {
  const cached = fileTreeCache.value.get(path);
  if (!cached) return null;

  console.log(`Using cached file tree for: ${path}`);
  return { tree: cached.tree, changes: cached.changes };
};

// 设置缓存的文件树
const setCachedFileTree = (path: string, tree: FileNode[], changes: GitStatus[]) => {
  fileTreeCache.value.set(path, {
    tree: JSON.parse(JSON.stringify(tree)), // 深拷贝避免引用问题
    changes: [...changes],
    timestamp: Date.now()
  });
  console.log(`Cached file tree for: ${path}`);
};

// 清除指定项目的缓存
const clearFileTreeCache = (path?: string) => {
  if (path) {
    fileTreeCache.value.delete(path);
    console.log(`Cleared cache for: ${path}`);
  } else {
    fileTreeCache.value.clear();
    console.log('Cleared all file tree caches');
  }
};

const loadFileTree = async (path: string, forceRefresh = false) => {
  try {
    // 先尝试使用缓存（如果不是强制刷新）
    if (!forceRefresh) {
      const cached = getCachedFileTree(path);
      if (cached) {
        fileTree.value = JSON.parse(JSON.stringify(cached.tree)); // 深拷贝避免引用问题
        gitChanges.value = [...cached.changes];

        // 如果有已删除的文件且用户选择显示，则添加到文件树
        if (showDeletedFiles.value) {
          const deletedChanges = cached.changes.filter((c: GitStatus) => c.status === 'Deleted');
          for (const deleted of deletedChanges) {
            addDeletedFileToTree(deleted.path);
          }
        }
        return;
      }
    }

    // 缓存未命中或强制刷新，重新加载
    console.log(`Loading file tree from disk: ${path}`);
    const entries = await invoke<any[]>('read_directory', { path });

    let changes: GitStatus[] = [];
    try {
      changes = await invoke<GitStatus[]>('get_working_tree_changes', { repoPath: path });
    } catch (e) {
      console.log('Not a git repository or error getting changes');
    }

    // 保存变更数据用于检测是否有已删除的文件
    gitChanges.value = changes;

    // 构建文件树（不包含已删除的文件）
    fileTree.value = await buildFileTreeRecursive(entries, path, changes);

    // 保存到缓存（缓存不包含已删除的文件）
    setCachedFileTree(path, fileTree.value, changes);

    // 如果有已删除的文件且用户选择显示，则添加到文件树（在缓存之后）
    if (showDeletedFiles.value) {
      const deletedChanges = changes.filter(c => c.status === 'Deleted');
      for (const deleted of deletedChanges) {
        addDeletedFileToTree(deleted.path);
      }
    }
  } catch (e) {
    console.error('Failed to load file tree:', e);
    alert('加载文件树失败: ' + e);
  }
};

// 添加已删除的文件到文件树
const addDeletedFileToTree = (filePath: string) => {
  const parts = filePath.split('/');
  let currentLevel = fileTree.value;

  for (let i = 0; i < parts.length; i++) {
    const part = parts[i];
    const isFile = i === parts.length - 1;
    const currentPath = parts.slice(0, i + 1).join('/');

    const existingNode = currentLevel.find(node => node.name === part);

    if (existingNode) {
      if (isFile) {
        // 更新已存在节点的状态为 Deleted
        existingNode.status = 'Deleted';
      } else {
        // 展开父目录
        existingNode.expanded = true;
        currentLevel = existingNode.children;
      }
    } else {
      // 创建新节点
      if (isFile) {
        currentLevel.push({
          name: part,
          path: currentPath,
          type: 'file',
          status: 'Deleted',
          children: []
        });
      } else {
        const newDir: FileNode = {
          name: part,
          path: currentPath,
          type: 'directory',
          children: [],
          expanded: true
        };
        currentLevel.push(newDir);
        currentLevel = newDir.children;
      }
    }
  }
};

// 加载更改的文件列表（工作区中已修改但未暂存的文件，类似 VSCode 的"更改"视图）
const loadStagedFiles = async () => {
  if (!currentPath.value) return;

  try {
    // 获取工作区的变更（未暂存的文件）
    const workingChanges = await invoke<GitStatus[]>('get_working_tree_changes', {
      repoPath: currentPath.value
    });

    // 过滤掉未跟踪的文件，只显示已修改、已删除、已重命名的文件
    const changedFiles = workingChanges.filter(change => {
      const status = change.status.toLowerCase();
      return status === 'modified' || status === 'deleted' || status === 'renamed' || status === 'added';
    });

    // 进一步过滤掉文件夹路径（Git 有时会将文件夹报告为变更）
    const fileOnlyChanges = await Promise.all(changedFiles.map(async change => {
      const fullPath = `${currentPath.value}/${change.path}`;
      try {
        // 尝试读取路径，如果是文件夹，会返回目录内容
        const entries = await invoke<any[]>('read_directory', { path: fullPath });
        // 如果能读取成功，说明是文件夹，排除它
        return null;
      } catch (e) {
        // 读取失败，说明是文件（或不存在），保留它
        return change;
      }
    }));

    stagedFiles.value = fileOnlyChanges
      .filter(change => change !== null)
      .map(change => {
        const parts = (change as GitStatus).path.split(/[\\/]/);
        return {
          name: parts[parts.length - 1] || (change as GitStatus).path,
          path: (change as GitStatus).path,
          status: (change as GitStatus).status
        };
      });
  } catch (e) {
    console.error('Failed to load changed files:', e);
    stagedFiles.value = [];
  }
};

// 插件变更回调
const onPluginsChanged = () => {
  // 重新加载当前文件以应用新的语法高亮
  if (currentFile.value) {
    loadFileContent(currentFile.value);
  }
};

// 处理刷新按钮点击 - 按住 Shift 键点击可强制刷新
const handleRefresh = (event?: MouseEvent) => {
  const forceReload = event?.shiftKey === true;
  if (forceReload) {
    console.log('Shift+Refresh: Force reloading file tree...');
  }
  refresh(forceReload);
};

// 轻量级刷新 - 只更新Git状态，不重建文件树
const refresh = async (forceReload = false) => {
  if (!currentPath.value) return;

  // 如果是强制刷新，清除缓存并重新加载文件树
  if (forceReload) {
    console.log('Force refreshing file tree...');
    clearFileTreeCache(currentPath.value);
    await loadFileTree(currentPath.value, true);
    return;
  }

  try {
    // 只获取Git状态变化，不重建整个文件树
    let changes: GitStatus[] = [];
    try {
      changes = await invoke<GitStatus[]>('get_working_tree_changes', { repoPath: currentPath.value });
    } catch (e) {
      console.log('Not a git repository or error getting changes');
    }

    // 更新 gitChanges 用于检测是否有已删除的文件
    gitChanges.value = changes;

    // 更新文件树中的状态（保持展开状态）
    updateFileTreeStatus(fileTree.value, changes);

    // 更新所有标签页的文件状态
    for (const tab of tabs.value) {
      if (tab.projectPath === currentPath.value) {
        await updateTabFileStatus(tab.path);
      }
    }

    // 如果有当前激活的标签页，重新加载其内容到视图
    if (activeTabId.value) {
      const activeTab = tabs.value.find(t => t.id === activeTabId.value);
      if (activeTab && activeTab.projectPath === currentPath.value) {
        // 直接从标签页数据更新视图，不需要重新加载文件
        leftLines.value = activeTab.leftLines;
        rightLines.value = activeTab.rightLines;
        isBinary.value = activeTab.isBinary;
        diffStats.value = activeTab.diffStats;
        // 更新当前文件的状态
        if (currentFile.value) {
          const newStatus = changes.find(c => c.path === activeTab.path)?.status;
          if (newStatus) {
            currentFile.value.status = newStatus;
          }
        }
      }
    }
  } catch (e) {
    console.error('Failed to refresh:', e);
  }
};

// 更新文件树状态（不重建树结构）
const updateFileTreeStatus = (nodes: FileNode[], changes: GitStatus[]) => {
  for (const node of nodes) {
    if (node.type === 'file') {
      const change = changes.find(c => c.path === node.path);
      if (change) {
        node.status = change.status;
      } else {
        node.status = undefined;
      }
    }
    // 文件夹不设置 status，因为 Git 不会跟踪文件夹本身
    // 文件夹的状态通过 has-changes 类由其子文件体现
    if (node.children && node.children.length > 0) {
      updateFileTreeStatus(node.children, changes);
    }
  }
};

// 滚动处理
const handleScroll = (scrollTop: number) => {
  // 可以在这里添加额外的滚动处理逻辑
};

// 获取 diff 缓存键
const getDiffCacheKey = (projectPath: string, filePath: string) => `${projectPath}:${filePath}`;

// 获取 diff 缓存
const getDiffFromCache = (key: string): { leftLines: DiffLine[]; rightLines: DiffLine[]; isBinary: boolean; diffStats: any } | null => {
  const cached = diffCache.value.get(key);
  if (!cached) return null;

  console.log('Using cached diff for:', key);
  return {
    leftLines: JSON.parse(JSON.stringify(cached.leftLines)),
    rightLines: JSON.parse(JSON.stringify(cached.rightLines)),
    isBinary: cached.isBinary,
    diffStats: cached.diffStats
  };
};

// 设置 diff 缓存
const setDiffCache = (key: string, result: { leftLines: DiffLine[]; rightLines: DiffLine[]; isBinary: boolean; diffStats: any }) => {
  diffCache.value.set(key, {
    leftLines: JSON.parse(JSON.stringify(result.leftLines)),
    rightLines: JSON.parse(JSON.stringify(result.rightLines)),
    isBinary: result.isBinary,
    diffStats: result.diffStats,
    timestamp: Date.now()
  });
};

// 清除指定项目的 diff 缓存
const clearDiffCache = (projectPath?: string) => {
  if (projectPath) {
    for (const key of diffCache.value.keys()) {
      if (key.startsWith(projectPath + ':')) {
        diffCache.value.delete(key);
      }
    }
    console.log('Cleared diff cache for project:', projectPath);
  } else {
    diffCache.value.clear();
    console.log('Cleared all diff caches');
  }
};

// 加载文件 diff 内容
const loadFileDiff = async (file: FileNode, forceRefresh = false): Promise<{ leftLines: DiffLine[]; rightLines: DiffLine[]; isBinary: boolean; diffStats: any } | null> => {
  const cacheKey = getDiffCacheKey(currentPath.value, file.path);

  // 尝试使用缓存
  if (!forceRefresh) {
    const cached = getDiffFromCache(cacheKey);
    if (cached) {
      return cached;
    }
  }

  try {
    const fileStatus = file.status?.toLowerCase();
    let workContent = '';
    let indexContent = '';

    if (fileStatus === 'deleted') {
      // 已删除的文件：工作区内容为空，从 Git 获取删除前的内容
      workContent = '';
      try {
        indexContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: 'HEAD'
        });
      } catch (e) {
        indexContent = '';
      }
    } else if (fileStatus === 'added') {
      // 新增的文件：索引内容为空，读取工作区内容
      workContent = await invoke<string>('read_file_content', {
        filePath: `${currentPath.value}/${file.path}`
      });
      indexContent = '';
    } else {
      // 修改的文件：读取工作区内容和 Git 中的内容
      workContent = await invoke<string>('read_file_content', {
        filePath: `${currentPath.value}/${file.path}`
      });
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
      const result = { leftLines: [], rightLines: [], isBinary: true, diffStats: null };
      setDiffCache(cacheKey, result);
      return result;
    }

    // 对于已删除的文件，直接显示完整的旧文件内容，所有行标记为 removed
    if (fileStatus === 'deleted') {
      const alignedLeftLines: DiffLine[] = [];
      const alignedRightLines: DiffLine[] = [];
      const oldLines = indexContent.split('\n');

      for (let i = 0; i < oldLines.length; i++) {
        alignedLeftLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: i + 1,
          content: oldLines[i],
          changeType: 'removed',
          isDiff: true
        });
      }

      const result = {
        leftLines: alignedLeftLines,
        rightLines: alignedRightLines,
        isBinary: false,
        diffStats: { added: 0, removed: oldLines.length, changed: 0 }
      };
      setDiffCache(cacheKey, result);
      return result;
    }

    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent: indexContent,
      newContent: workContent
    });

    const alignedLeftLines: DiffLine[] = [];
    const alignedRightLines: DiffLine[] = [];

    let added = 0, removed = 0, changed = 0;
    let leftLineNum = 1;
    let rightLineNum = 1;

    diffResult.hunks.forEach(hunk => {
        // 添加上下文行 - hunk.old_start 和 hunk.new_start 是 1-based 行号
        const contextStart = Math.min(hunk.old_start, hunk.new_start) - 1;
        for (let i = alignedLeftLines.length; i < contextStart; i++) {
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

      let pendingRemoved: { content: string; lineNum: number } | null = null;

      hunk.lines.forEach(line => {
        if (line.change_type === 'removed') {
          pendingRemoved = { content: line.content, lineNum: rightLineNum++ };
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

    const result = {
      leftLines: alignedLeftLines,
      rightLines: alignedRightLines,
      isBinary: false,
      diffStats: { added, removed, changed }
    };

    // 保存到缓存
    setDiffCache(cacheKey, result);

    return result;
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

  const existingTab = tabs.value.find(tab => tab.path === path && tab.projectPath === currentPath.value);
  if (existingTab) {
    await activateTab(existingTab.id);
    return;
  }

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

  const diffResult = await loadFileDiff(file);
  if (diffResult) {
    newTab.leftLines = diffResult.leftLines;
    newTab.rightLines = diffResult.rightLines;
    newTab.isBinary = diffResult.isBinary;
    newTab.diffStats = diffResult.diffStats;
  }

  tabs.value.push(newTab);
  activeTabId.value = newTab.id;

  updateCurrentViewFromTab(newTab);
};

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

const activateTab = async (tabId: string) => {
  const tab = tabs.value.find(t => t.id === tabId);
  if (!tab) return;

  activeTabId.value = tabId;

  if (tab.projectPath !== currentPath.value) {
    currentPath.value = tab.projectPath;
    await loadFileTree(tab.projectPath);
    await invoke('start_file_watcher', { repoPath: tab.projectPath });
  }

  updateCurrentViewFromTab(tab);
};

const closeTab = (tabId: string) => {
  const index = tabs.value.findIndex(t => t.id === tabId);
  if (index === -1) return;

  tabs.value.splice(index, 1);

  if (activeTabId.value === tabId) {
    if (tabs.value.length > 0) {
      const newIndex = Math.min(index, tabs.value.length - 1);
      const newTab = tabs.value[newIndex];
      activateTab(newTab.id);
    } else {
      activeTabId.value = '';
      currentFile.value = null;
      leftLines.value = [];
      rightLines.value = [];
      isBinary.value = false;
      diffStats.value = null;
    }
  }
};

const closeAllTabs = () => {
  tabs.value = [];
  activeTabId.value = '';
  currentFile.value = null;
  leftLines.value = [];
  rightLines.value = [];
  isBinary.value = false;
  diffStats.value = null;
};

const closeOtherTabs = (tabId: string) => {
  const tab = tabs.value.find(t => t.id === tabId);
  if (!tab) return;

  tabs.value = [tab];
  activeTabId.value = tabId;
  updateCurrentViewFromTab(tab);
};

const closeTabsToRight = (tabId: string) => {
  const index = tabs.value.findIndex(t => t.id === tabId);
  if (index === -1) return;

  tabs.value = tabs.value.slice(0, index + 1);

  if (!tabs.value.find(t => t.id === activeTabId.value)) {
    const lastTab = tabs.value[tabs.value.length - 1];
    if (lastTab) {
      activateTab(lastTab.id);
    }
  }
};

const updateTabFileStatus = async (filePath: string) => {
  const tab = tabs.value.find(t => t.path === filePath && t.projectPath === currentPath.value);
  if (!tab) return;

  // 从文件树中查找文件状态
  const findFileStatus = (nodes: FileNode[]): string | undefined => {
    for (const node of nodes) {
      if (node.path === filePath && node.type === 'file') {
        return node.status;
      }
      if (node.children) {
        const status = findFileStatus(node.children);
        if (status) return status;
      }
    }
    return undefined;
  };

  const fileStatus = findFileStatus(fileTree.value);

  const fileNode: FileNode = {
    name: tab.name,
    path: tab.path,
    type: 'file',
    status: fileStatus || '',
    children: []
  };

  const diffResult = await loadFileDiff(fileNode);
  if (diffResult) {
    tab.leftLines = diffResult.leftLines;
    tab.rightLines = diffResult.rightLines;
    tab.isBinary = diffResult.isBinary;
    tab.diffStats = diffResult.diffStats;
    tab.isModified = true;

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

// 文件比对对话框
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

    const newLeftLines: DiffLine[] = diffResult.new_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    const newRightLines: DiffLine[] = diffResult.old_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    diffResult.hunks.forEach(hunk => {
      hunk.lines.forEach(line => {
        if (line.change_type === 'added') {
          const leftLine = newLeftLines.find(l => l.lineNum === line.line_number);
          if (leftLine) {
            leftLine.changeType = 'added';
            leftLine.isDiff = true;
          }
        } else if (line.change_type === 'removed') {
          const rightLine = newRightLines.find(l => l.lineNum === line.line_number);
          if (rightLine) {
            rightLine.changeType = 'removed';
            rightLine.isDiff = true;
          }
        }
      });
    });

    leftLines.value = newLeftLines;
    rightLines.value = newRightLines;

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

    const newLeftLines: DiffLine[] = diffResult.new_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    const newRightLines: DiffLine[] = diffResult.old_content.map((content, idx) => ({
      lineNum: idx + 1,
      content,
      changeType: 'unchanged',
      isDiff: false
    }));

    diffResult.hunks.forEach(hunk => {
      hunk.lines.forEach(line => {
        if (line.change_type === 'added') {
          const leftLine = newLeftLines.find(l => l.lineNum === line.line_number);
          if (leftLine) {
            leftLine.changeType = 'added';
            leftLine.isDiff = true;
          }
        } else if (line.change_type === 'removed') {
          const rightLine = newRightLines.find(l => l.lineNum === line.line_number);
          if (rightLine) {
            rightLine.changeType = 'removed';
            rightLine.isDiff = true;
          }
        }
      });
    });

    leftLines.value = newLeftLines;
    rightLines.value = newRightLines;

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

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: var(--bg-primary);
  color: var(--text-primary);
}

.main-container {
  flex: 1;
  display: flex;
  overflow: hidden;
}

/* 权限提示弹窗样式 */
.permission-overlay {
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

.permission-dialog {
  background-color: var(--bg-primary);
  border-radius: 8px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  width: 90%;
  max-width: 500px;
  max-height: 90vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.permission-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.permission-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--text-primary);
}

.permission-header .close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.permission-header .close-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.permission-content {
  padding: 20px;
  overflow-y: auto;
  flex: 1;
}

.permission-content p {
  margin: 0 0 12px 0;
  color: var(--text-primary);
  line-height: 1.6;
}

.permission-content ol {
  margin: 12px 0;
  padding-left: 24px;
  color: var(--text-primary);
  line-height: 1.8;
}

.permission-content li {
  margin-bottom: 8px;
}

.permission-actions {
  display: flex;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  justify-content: flex-end;
}

.permission-actions .btn {
  padding: 8px 16px;
  border-radius: 6px;
  border: 1px solid transparent;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.permission-actions .btn-primary {
  background-color: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.permission-actions .btn-primary:hover {
  opacity: 0.9;
}

.permission-actions .btn-secondary {
  background-color: transparent;
  color: var(--text-secondary);
  border-color: var(--border-color);
}

.permission-actions .btn-secondary:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}
</style>