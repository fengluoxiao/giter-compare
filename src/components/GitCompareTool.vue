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
      @refresh="refresh"
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
        :projects="projects"
        :current-project-id="currentProjectId"
        :is-collapsed="isProjectSidebarCollapsed"
        :width="projectSidebarWidth"
        @add-project="showAddProjectDialog"
        @toggle-collapse="toggleProjectSidebar"
        @switch-project="switchProject"
        @remove-project="removeProject"
        @start-resize="startResizeProjectSidebar"
        @export-projects="exportProjects"
        @import-projects="importProjects"
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
import { FileCompareDialog, TextCompareDialog, AddProjectDialog, PluginManagerDialog, WorkspaceManagerDialog } from './dialogs';

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

// 暂存区状态
const stagedFiles = ref<{ name: string; path: string; status?: string }[]>([]);
const selectedStagedPath = ref<string>('');

// 标签页支持
const tabs = ref<Tab[]>([]);
const activeTabId = ref<string>('');

// 多项目支持
const projects = ref<Project[]>([]);
const currentProjectId = ref<string>('');
const newProjectName = ref('');
const newProjectPath = ref('');
const pendingProjects = ref<{ name: string; path: string }[]>([]);
const editingPendingIndex = ref<number>(-1);
const editingPendingName = ref('');

// 侧边栏宽度和折叠状态
const projectSidebarWidth = ref(200);
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

  loadProjects();

  unlistenFileChange = await listen('file-changed', () => {
    if (currentPath.value) {
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
  if (pendingProjects.value.length === 0) return;

  for (const item of pendingProjects.value) {
    console.log('Adding project:', item.name, item.path);
    const project: Project = {
      id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
      name: item.name,
      path: item.path
    };
    projects.value.push(project);
  }

  saveProjects();

  const lastProject = projects.value[projects.value.length - 1];
  await switchProject(lastProject);

  closeAddProjectDialog();
};

const removeProject = (projectId: string) => {
  const index = projects.value.findIndex(p => p.id === projectId);
  if (index === -1) return;

  projects.value.splice(index, 1);
  saveProjects();

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

// 导出项目列表为 JSON 文件
const exportProjects = async () => {
  try {
    const { save } = await import('@tauri-apps/plugin-dialog');
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');

    const filePath = await save({
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
    }
  } catch (e) {
    console.error('Failed to export projects:', e);
    alert('导出失败: ' + e);
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
        let addedCount = 0;

        for (const project of importData.projects) {
          if (!existingPaths.has(project.path)) {
            projects.value.push({
              id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
              name: project.name,
              path: project.path
            });
            addedCount++;
          }
        }

        saveProjects();
        alert(`成功导入 ${addedCount} 个项目！`);
      } else {
        alert('无效的项目文件格式');
      }
    }
  } catch (e) {
    console.error('Failed to import projects:', e);
    alert('导入失败: ' + e);
  }
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

  const savedProjectWidth = localStorage.getItem('giter-project-sidebar-width');
  const savedFileWidth = localStorage.getItem('giter-file-sidebar-width');
  const savedProjectCollapsed = localStorage.getItem('giter-project-sidebar-collapsed');
  const savedFileCollapsed = localStorage.getItem('giter-file-sidebar-collapsed');

  if (savedProjectWidth) projectSidebarWidth.value = parseInt(savedProjectWidth);
  if (savedFileWidth) fileSidebarWidth.value = parseInt(savedFileWidth);
  if (savedProjectCollapsed) isProjectSidebarCollapsed.value = savedProjectCollapsed === 'true';
  if (savedFileCollapsed) isFileSidebarCollapsed.value = savedFileCollapsed === 'true';
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

  saveProjects();
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

const loadFileTree = async (path: string) => {
  try {
    const entries = await invoke<any[]>('read_directory', { path });

    let changes: GitStatus[] = [];
    try {
      changes = await invoke<GitStatus[]>('get_working_tree_changes', { repoPath: path });
    } catch (e) {
      console.log('Not a git repository or error getting changes');
    }

    // 保存变更数据用于检测是否有已删除的文件
    gitChanges.value = changes;

    fileTree.value = await buildFileTreeRecursive(entries, path, changes);

    // 如果有已删除的文件且用户选择显示，则添加到文件树
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

    stagedFiles.value = changedFiles.map(change => {
      const parts = change.path.split(/[\\/]/);
      return {
        name: parts[parts.length - 1] || change.path,
        path: change.path,
        status: change.status
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

// 轻量级刷新 - 只更新Git状态，不重建文件树
const refresh = async () => {
  if (!currentPath.value) return;

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
    if (node.children && node.children.length > 0) {
      updateFileTreeStatus(node.children, changes);
    }
  }
};

// 滚动处理
const handleScroll = (scrollTop: number) => {
  // 可以在这里添加额外的滚动处理逻辑
};

// 加载文件 diff 内容
const loadFileDiff = async (file: FileNode): Promise<{ leftLines: DiffLine[]; rightLines: DiffLine[]; isBinary: boolean; diffStats: any } | null> => {
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
      return { leftLines: [], rightLines: [], isBinary: true, diffStats: null };
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

      return {
        leftLines: alignedLeftLines,
        rightLines: alignedRightLines,
        isBinary: false,
        diffStats: { added: 0, removed: oldLines.length, changed: 0 }
      };
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
</style>