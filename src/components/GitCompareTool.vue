<template>
  <div class="app">
    <!-- 网页工具栏（仅在非 macOS 或原生工具栏不可用时显示） -->
    <Toolbar
      v-if="!useNativeToolbar"
      :theme="theme"
      :has-prev="hasPrev"
      :has-next="hasNext"
      @compare-file="showCompareFileDialog"
      @compare-text="showTextCompare = true"
      @toggle-theme="toggleTheme"
      @navigate-prev="navigatePrev"
      @navigate-next="navigateNext"
      @refresh="handleRefresh"
      @app-settings="showAppSettings = true"
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

    <!-- 全局搜索对话框 -->
    <GlobalSearchDialog
      ref="globalSearchDialog"
      @open-file="handleGlobalSearchOpenFile"
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
        @project-settings="openProjectSettings"
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
        @file-contextmenu="handleFileContextMenu"
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
        :old-version="projectSettings.leftVersion"
        :new-version="projectSettings.rightVersion"
        :commit-list="commitList"
        :compare-branches="projectSettings.compareBranches"
        :old-branch="projectSettings.leftBranch"
        :new-branch="projectSettings.rightBranch"
        :branch-list="branchList"
        :left-blame-info="leftBlameInfo"
        :right-blame-info="rightBlameInfo"
        @scroll="handleScroll"
        @change-old-version="onDiffOldVersionChange"
        @change-new-version="onDiffNewVersionChange"
        @change-old-branch="onDiffOldBranchChange"
        @change-new-branch="onDiffNewBranchChange"
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

    <!-- 项目设置弹窗 -->
    <div v-if="showProjectSettings" class="permission-overlay" @click.self="showProjectSettings = false">
      <div class="project-settings-dialog">
        <div class="settings-header">
          <h3>项目设置</h3>
          <button class="close-btn" @click="showProjectSettings = false">×</button>
        </div>
        <div class="settings-content">
          <!-- 项目信息 -->
          <div class="settings-section">
            <h4>项目信息</h4>
            <div class="settings-row">
              <label class="settings-label">项目名称</label>
              <input type="text" v-model="projectSettings.name" class="settings-input" />
            </div>
            <div class="settings-row">
              <label class="settings-label">项目路径</label>
              <input type="text" v-model="projectSettings.path" class="settings-input" readonly />
            </div>
          </div>

          <!-- Git 设置 -->
          <div class="settings-section">
            <h4>Git 设置</h4>
            <div class="settings-row checkbox">
              <input type="checkbox" id="autoFetch" v-model="projectSettings.autoFetch" />
              <label for="autoFetch">自动获取远程更新</label>
            </div>
            <div class="settings-row checkbox">
              <input type="checkbox" id="showUntracked" v-model="projectSettings.showUntracked" />
              <label for="showUntracked">显示未跟踪文件</label>
            </div>
            <div class="settings-row">
              <label class="settings-label">所选分支</label>
              <select v-model="projectSettings.defaultBranch" class="settings-input settings-select">
                <option v-for="branch in branchList" :key="branch" :value="branch">{{ branch }}</option>
              </select>
            </div>
          </div>

          <!-- 比对版本 -->
          <div class="settings-section">
            <h4>比对版本</h4>
            <div class="settings-row checkbox">
              <input type="checkbox" id="compareBranches" v-model="projectSettings.compareBranches" @change="onCompareBranchesChange" />
              <label for="compareBranches">比对不同分支</label>
            </div>
            <!-- 分支比对模式 -->
            <template v-if="projectSettings.compareBranches">
              <div class="settings-row">
                <label class="settings-label">旧分支</label>
                <select v-model="projectSettings.leftBranch" class="settings-input settings-select" @change="onBranchChange">
                  <option v-for="branch in branchList" :key="branch" :value="branch">{{ branch }}</option>
                </select>
              </div>
              <div class="settings-row">
                <label class="settings-label">新分支</label>
                <select v-model="projectSettings.rightBranch" class="settings-input settings-select" @change="onBranchChange">
                  <option v-for="branch in branchList" :key="branch" :value="branch">{{ branch }}</option>
                </select>
              </div>
            </template>
            <!-- Commit 比对模式 -->
            <template v-else>
              <div class="settings-row">
                <label class="settings-label">旧版本</label>
                <select v-model="projectSettings.leftVersion" class="settings-input settings-select" @change="onSettingsOldVersionChange">
                  <option v-for="commit in commitList" :key="commit.hash" :value="commit.hash">
                    {{ commit.short_hash }} - {{ commit.message }}
                  </option>
                </select>
              </div>
              <div class="settings-row">
                <label class="settings-label">新版本</label>
                <select v-model="projectSettings.rightVersion" class="settings-input settings-select" @change="onSettingsNewVersionChange">
                  <option value="WORKING">工作区 (最新未提交)</option>
                  <option v-for="commit in availableNewVersions" :key="commit.hash" :value="commit.hash">
                    {{ commit.short_hash }} - {{ commit.message }}
                  </option>
                </select>
              </div>
            </template>
          </div>

          <!-- 对比设置 -->
          <div class="settings-section">
            <h4>对比设置</h4>
            <div class="settings-row checkbox">
              <input type="checkbox" id="ignoreWhitespace" v-model="projectSettings.ignoreWhitespace" />
              <label for="ignoreWhitespace">忽略空白差异</label>
            </div>
            <div class="settings-row checkbox">
              <input type="checkbox" id="caseSensitive" v-model="projectSettings.caseSensitive" />
              <label for="caseSensitive">大小写敏感</label>
            </div>
            <div class="settings-row checkbox">
              <input type="checkbox" id="ignoreLineEnding" v-model="projectSettings.ignoreLineEnding" />
              <label for="ignoreLineEnding">忽略行尾差异 (CRLF/LF)</label>
            </div>
          </div>

          <!-- 文件过滤 -->
          <div class="settings-section">
            <h4>文件过滤</h4>
            <div class="settings-row">
              <label class="settings-label">忽略文件模式（每行一个）</label>
              <textarea v-model="projectSettings.ignorePatterns" class="settings-textarea" rows="3" placeholder="*.log&#10;node_modules/&#10;.DS_Store"></textarea>
            </div>
            <div class="settings-row checkbox">
              <input type="checkbox" id="showHidden" v-model="projectSettings.showHiddenFiles" />
              <label for="showHidden">显示隐藏文件</label>
            </div>
          </div>

          <!-- 通知设置 -->
          <div class="settings-section">
            <h4>通知设置</h4>
            <div class="settings-row checkbox">
              <input type="checkbox" id="notifyChanges" v-model="projectSettings.notifyChanges" />
              <label for="notifyChanges">文件变更时通知</label>
            </div>
            <div class="settings-row checkbox">
              <input type="checkbox" id="notifySync" v-model="projectSettings.notifySync" />
              <label for="notifySync">同步完成时通知</label>
            </div>
          </div>

        </div>
        <div class="settings-actions">
          <button class="btn btn-secondary" @click="showProjectSettings = false">取消</button>
          <button class="btn btn-primary" @click="saveProjectSettings">保存设置</button>
        </div>
      </div>
    </div>

    <!-- 文件历史弹窗 -->
    <div v-if="showFileHistory" class="permission-overlay" @click.self="showFileHistory = false">
      <div class="project-settings-dialog file-history-dialog">
        <div class="settings-header">
          <h3>📜 {{ fileHistoryFileName }} - 历史变更</h3>
          <button class="close-btn" @click="showFileHistory = false">×</button>
        </div>
        <div class="settings-content file-history-content">
          <div v-if="fileHistoryLoading" class="history-loading">加载中...</div>
          <div v-else-if="fileHistoryList.length === 0" class="history-empty">暂无历史记录</div>
          <div v-else class="history-list">
            <div
              v-for="(commit, index) in fileHistoryList"
              :key="commit.hash"
              class="history-item"
              :class="{ 'history-item-first': index === 0 }"
              @click="viewFileAtCommit(commit.hash)"
            >
              <div class="history-commit-header">
                <span class="history-hash">{{ commit.short_hash }}</span>
                <span class="history-date">{{ commit.date }}</span>
              </div>
              <div class="history-message">{{ commit.message }}</div>
              <div class="history-author">{{ commit.author }}</div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 历史文件内容弹窗 -->
    <div v-if="showHistoryFileContent" class="permission-overlay" @click.self="closeHistoryFileContent">
      <div class="project-settings-dialog file-content-dialog">
        <div class="settings-header">
          <h3>📄 {{ fileHistoryFileName }}</h3>
          <button class="close-btn" @click="closeHistoryFileContent">×</button>
        </div>
        <div class="file-content-commit-info">
          <span>{{ historyFileContentCommit }}</span>
          <label class="diff-toggle">
            <input
              type="checkbox"
              v-model="showOnlyDiff"
              @change="onShowOnlyDiffChange"
            />
            <span class="diff-toggle-label">只显示变更</span>
          </label>
        </div>
        <div class="file-content-body">
          <div v-if="historyFileContentLoading" class="history-loading">加载中...</div>
          <ShikiDiffLines
            v-else
            :lines="historyFileLines"
            :filename="fileHistoryFileName"
            :theme="theme === 'dark' ? 'dark' : 'light'"
          />
        </div>
      </div>
    </div>

    <!-- 软件设置弹窗 -->
    <div v-if="showAppSettings" class="settings-dialog-overlay" @click="showAppSettings = false">
      <div class="settings-dialog app-settings-dialog" @click.stop>
        <div class="settings-header">
          <h3>软件设置</h3>
          <button class="close-btn" @click="showAppSettings = false">
            <svg viewBox="0 0 24 24" width="16" height="16">
              <path fill="currentColor" d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
            </svg>
          </button>
        </div>
        <div class="app-settings-container">
          <!-- 左侧标签页 -->
          <div class="app-settings-tabs">
            <div
              v-for="tab in appSettingTabs"
              :key="tab.key"
              class="app-settings-tab"
              :class="{ active: activeAppSettingTab === tab.key }"
              @click="activeAppSettingTab = tab.key"
            >
              <span class="tab-icon">{{ tab.icon }}</span>
              <span class="tab-label">{{ tab.label }}</span>
            </div>
          </div>
          <!-- 右侧内容区 -->
          <div class="app-settings-content">
            <!-- 外观设置 -->
            <div v-if="activeAppSettingTab === 'appearance'" class="app-settings-panel">
              <h4>外观设置</h4>
              <div class="settings-row">
                <label class="settings-label">主题模式</label>
                <div class="theme-options">
                  <button
                    class="theme-option"
                    :class="{ active: theme === 'light' }"
                    @click="setTheme('light')"
                  >
                    <span class="theme-preview light-preview"></span>
                    <span>浅色</span>
                  </button>
                  <button
                    class="theme-option"
                    :class="{ active: theme === 'dark' }"
                    @click="setTheme('dark')"
                  >
                    <span class="theme-preview dark-preview"></span>
                    <span>深色</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- 工作区管理 -->
            <div v-if="activeAppSettingTab === 'workspace'" class="app-settings-panel">
              <h4>工作区管理</h4>
              <!-- 导入项目 -->
              <div class="settings-subsection">
                <h5>导入项目</h5>
                <button class="btn btn-secondary" @click="importProjectsFromFolder">
                  📁 从文件夹导入项目
                </button>
              </div>
              <!-- 保存当前工作区 -->
              <div class="settings-subsection">
                <h5>保存当前工作区</h5>
                <div v-if="projects.length > 0" class="current-projects-mini">
                  <div class="projects-header">当前项目列表 ({{ projects.length }}个)</div>
                  <div class="projects-list-mini">
                    <div v-for="project in projects" :key="project.id" class="project-item-mini">
                      <span class="project-name">{{ project.name }}</span>
                      <span class="project-path">{{ project.path }}</span>
                    </div>
                  </div>
                </div>
                <div v-else class="no-projects">当前没有项目</div>
                <div class="input-group">
                  <input
                    v-model="newWorkspaceName"
                    type="text"
                    placeholder="输入工作区名称"
                    class="settings-input"
                    @keyup.enter="saveCurrentWorkspaceFromSettings"
                  />
                  <button
                    class="btn btn-primary"
                    @click="saveCurrentWorkspaceFromSettings"
                    :disabled="!newWorkspaceName.trim()"
                  >
                    保存
                  </button>
                </div>
              </div>
              <!-- 已保存的工作区 -->
              <div class="settings-subsection">
                <h5>已保存的工作区</h5>
                <div v-if="savedWorkspaces.length === 0" class="empty-state">暂无保存的工作区</div>
                <div v-else class="workspace-list-mini">
                  <div
                    v-for="workspace in savedWorkspaces"
                    :key="workspace.id"
                    class="workspace-item-mini"
                  >
                    <div class="workspace-info">
                      <span class="workspace-name">{{ workspace.name }}</span>
                      <span class="workspace-meta">{{ workspace.projects.length }} 个项目 · {{ formatDate(workspace.createdAt) }}</span>
                    </div>
                    <div class="workspace-actions">
                      <button class="btn btn-icon" @click="loadWorkspaceFromSettings(workspace)" title="加载">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                          <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                        </svg>
                      </button>
                      <button class="btn btn-icon btn-danger" @click="deleteWorkspaceFromSettings(workspace.id)" title="删除">
                        <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                          <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 插件管理 -->
            <div v-if="activeAppSettingTab === 'plugins'" class="app-settings-panel">
              <h4>语法高亮插件管理</h4>
              <div class="builtin-notice">
                <span class="notice-icon">ℹ️</span>
                <span>已内置 Vue 语法高亮支持</span>
              </div>
              <!-- 已安装插件 -->
              <div class="settings-subsection">
                <h5>已安装插件</h5>
                <div v-if="pluginLoading" class="loading-state">加载中...</div>
                <div v-else-if="installedPlugins.length === 0" class="empty-state">暂无安装的插件</div>
                <div v-else class="plugin-list-mini">
                  <div v-for="plugin in installedPlugins" :key="plugin.id" class="plugin-item-mini">
                    <div class="plugin-info">
                      <div class="plugin-header">
                        <span class="plugin-name">{{ plugin.name }}</span>
                        <span class="plugin-version">v{{ plugin.version }}</span>
                      </div>
                      <div class="plugin-description">{{ plugin.description }}</div>
                      <div class="plugin-grammars">
                        <span v-for="grammar in plugin.grammars" :key="grammar.language" class="grammar-tag">
                          {{ grammar.language }}
                        </span>
                      </div>
                    </div>
                    <button class="btn btn-danger btn-sm" @click="removePluginFromSettings(plugin.id)">删除</button>
                  </div>
                </div>
              </div>
              <!-- 导入插件 -->
              <div class="settings-subsection">
                <h5>导入新插件</h5>
                <button class="btn btn-primary" @click="importPluginFromSettings" :disabled="pluginImporting">
                  {{ pluginImporting ? '导入中...' : '📂 选择插件文件夹' }}
                </button>
                <span class="import-hint">选择 VSCode 语法高亮插件文件夹</span>
                <div v-if="pluginImportError" class="import-error">{{ pluginImportError }}</div>
              </div>
            </div>
          </div>
        </div>
        <div class="settings-actions">
          <button class="btn btn-primary" @click="showAppSettings = false">关闭</button>
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

    <!-- 底部状态栏 -->
    <div class="status-bar">
      <div class="status-bar-left">
        <div class="status-item" title="当前分支">
          <span class="status-icon">🌿</span>
          <span class="status-text">{{ currentBranch || '未选择项目' }}</span>
        </div>
        <div class="status-item" v-if="currentProject" title="当前项目">
          <span class="status-icon">📁</span>
          <span class="status-text">{{ currentProject.name }}</span>
        </div>
      </div>
      <div class="status-bar-right">
        <div class="status-item" title="文件编码">
          <span class="status-text">UTF-8</span>
        </div>
        <div class="status-item" v-if="currentFile" title="换行符">
          <span class="status-text">LF</span>
        </div>
        <div class="status-item" v-if="currentFile" title="文件类型">
          <span class="status-text">{{ getFileExtension(currentFile.name) || 'TEXT' }}</span>
        </div>
        <div class="status-item" v-if="diffStats" title="变更统计">
          <span class="status-add">+{{ diffStats.added }}</span>
          <span class="status-del">-{{ diffStats.removed }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, nextTick } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import Toolbar from './Toolbar.vue';
import ProjectSidebar from './ProjectSidebar.vue';
import FileTreeSidebar from './FileTreeSidebar.vue';
import DiffViewer from './DiffViewer.vue';
import ShikiDiffLines from './ShikiDiffLines.vue';
import TabBar, { type Tab } from './TabBar.vue';
import { FileCompareDialog, TextCompareDialog, AddProjectDialog, PromptDialog, GlobalSearchDialog } from './dialogs';

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
  createdAt?: string;
}

// 主题和视图状态
const theme = ref('light');
const viewMode = ref<'working' | 'staged'>('working');
const showAllFiles = ref(true);
const showDeletedFiles = ref(false);

// 文件历史弹窗状态
const showFileHistory = ref(false);
const fileHistoryFileName = ref('');
const fileHistoryFilePath = ref('');
const fileHistoryLoading = ref(false);
const fileHistoryList = ref<CommitInfo[]>([]);

// 查看历史文件内容弹窗状态
const showHistoryFileContent = ref(false);
const historyFileContent = ref('');
const historyFileContentCommit = ref('');
const historyFileContentLoading = ref(false);
const currentHistoryCommit = ref(''); // 当前查看的 commit hash
const showOnlyDiff = ref(false); // 只显示该版本更新的代码

// 关闭历史文件内容弹窗
const closeHistoryFileContent = () => {
  showHistoryFileContent.value = false;
  showOnlyDiff.value = false; // 关闭窗口时重置开关状态
};

// 历史文件内容行数据（用于语法高亮）
const historyFileLines = computed(() => {
  if (!historyFileContent.value) return [];
  return historyFileContent.value.split('\n').map((line, index) => {
    // 根据前缀判断 changeType
    let changeType = 'unchanged';
    if (line.startsWith('+ ')) {
      changeType = 'added';
    } else if (line.startsWith('- ')) {
      changeType = 'removed';
    }
    return {
      lineNum: index + 1,
      content: line,
      changeType: changeType,
      isDiff: changeType !== 'unchanged'
    };
  });
});

// 是否使用原生工具栏（macOS 平台）
const useNativeToolbar = ref(false);

// 对话框状态
const showCompareFile = ref(false);
const showTextCompare = ref(false);
const showAddProject = ref(false);
const showPermissionDialog = ref(false);
const showProjectSettings = ref(false);
const showAppSettings = ref(false);
const activeAppSettingTab = ref('appearance');
const appSettingTabs = [
  { key: 'appearance', label: '外观', icon: '🎨' },
  { key: 'workspace', label: '工作区', icon: '💼' },
  { key: 'plugins', label: '插件', icon: '🔌' },
];

// 工作区管理状态（软件设置内）
const newWorkspaceName = ref('');
const savedWorkspaces = ref<Workspace[]>([]);

// 插件管理状态（软件设置内）
interface PluginInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  grammars: { language: string; scope_name: string; path: string }[];
}
const installedPlugins = ref<PluginInfo[]>([]);
const pluginLoading = ref(false);
const pluginImporting = ref(false);
const pluginImportError = ref('');

const showPromptDialog = ref(false);

// 项目设置
interface ProjectSettings {
  name: string;
  path: string;
  autoFetch: boolean;
  showUntracked: boolean;
  defaultBranch: string;
  leftVersion: string;
  rightVersion: string;
  compareBranches: boolean;
  leftBranch: string;
  rightBranch: string;
  ignoreWhitespace: boolean;
  caseSensitive: boolean;
  ignoreLineEnding: boolean;
  ignorePatterns: string;
  showHiddenFiles: boolean;
  notifyChanges: boolean;
  notifySync: boolean;
}

const projectSettings = ref<ProjectSettings>({
  name: '',
  path: '',
  autoFetch: true,
  showUntracked: true,
  defaultBranch: 'main',
  leftVersion: 'HEAD',
  rightVersion: 'WORKING',
  compareBranches: false,
  leftBranch: 'main',
  rightBranch: 'main',
  ignoreWhitespace: false,
  caseSensitive: true,
  ignoreLineEnding: true,
  ignorePatterns: '*.log\nnode_modules/\n.DS_Store',
  showHiddenFiles: false,
  notifyChanges: true,
  notifySync: false,
});

// 分支列表
const branchList = ref<string[]>([]);

// Commit 列表
interface CommitInfo {
  hash: string;
  short_hash: string;
  message: string;
  author: string;
  date: string;
}
const commitList = ref<CommitInfo[]>([]);

// 内存中存储每个项目的版本设置（不持久化，重启后恢复默认）
const projectVersionSettings = new Map<string, {
  leftVersion: string;
  rightVersion: string;
  compareBranches: boolean;
  leftBranch: string;
  rightBranch: string;
}>();

// 根据旧版本选择，计算可用的新版本列表
const availableNewVersions = computed(() => {
  const oldHash = projectSettings.value.leftVersion;
  if (!oldHash || oldHash === 'WORKING') {
    return commitList.value;
  }
  const oldIndex = commitList.value.findIndex(c => c.hash === oldHash);
  if (oldIndex === -1) {
    return commitList.value;
  }
  // 只返回比旧版本更新的 commit（索引更小）
  return commitList.value.slice(0, oldIndex);
});

// Blame 信息
interface BlameInfo {
  line_number: number;
  commit_hash: string;
  short_hash: string;
  author: string;
  email: string;
  timestamp: number;
  summary: string;
}

const leftBlameInfo = ref<BlameInfo[]>([]);
const rightBlameInfo = ref<BlameInfo[]>([]);

// 加载 blame 信息
const loadBlameInfo = async (filePath: string) => {
  if (!currentPath.value) return;

  try {
    // 加载左侧 blame
    const oldVersion = projectSettings.value.leftVersion;
    console.log('Loading blame for left version:', oldVersion, 'file:', filePath);
    if (oldVersion && oldVersion !== 'WORKING') {
      leftBlameInfo.value = await invoke<BlameInfo[]>('get_file_blame', {
        repoPath: currentPath.value,
        filePath,
        revision: oldVersion
      });
      console.log('Left blame loaded:', leftBlameInfo.value.length, 'lines');
    } else {
      leftBlameInfo.value = [];
      console.log('Left version is WORKING, skipping blame');
    }

    // 加载右侧 blame
    const newVersion = projectSettings.value.rightVersion;
    console.log('Loading blame for right version:', newVersion, 'file:', filePath);
    if (newVersion && newVersion !== 'WORKING') {
      rightBlameInfo.value = await invoke<BlameInfo[]>('get_file_blame', {
        repoPath: currentPath.value,
        filePath,
        revision: newVersion
      });
      console.log('Right blame loaded:', rightBlameInfo.value.length, 'lines');
    } else {
      rightBlameInfo.value = [];
      console.log('Right version is WORKING, skipping blame');
    }
  } catch (e) {
    console.error('Failed to load blame info:', e);
    leftBlameInfo.value = [];
    rightBlameInfo.value = [];
  }
};

// 旧版本变更处理
const onOldVersionChange = () => {
  const oldHash = projectSettings.value.leftVersion;
  const currentRight = projectSettings.value.rightVersion;

  // 如果当前新版本不在可用列表中，重置为 WORKING
  if (oldHash && oldHash !== 'WORKING') {
    const oldIndex = commitList.value.findIndex(c => c.hash === oldHash);
    if (oldIndex !== -1) {
      const rightIndex = commitList.value.findIndex(c => c.hash === currentRight);
      // 如果右边选的版本不在左边版本之前（即不比它新），则重置
      if (rightIndex === -1 || rightIndex >= oldIndex) {
        projectSettings.value.rightVersion = 'WORKING';
      }
    }
  }
};

// 项目设置弹窗 - 旧版本变更处理
const onSettingsOldVersionChange = async () => {
  onOldVersionChange();
  await saveAndRefreshVersions();
};

// 项目设置弹窗 - 新版本变更处理
const onSettingsNewVersionChange = async () => {
  await saveAndRefreshVersions();
};

// 分支比对开关切换处理
const onCompareBranchesChange = async () => {
  if (projectSettings.value.compareBranches) {
    // 切换到分支比对模式：使用分支最新 commit
    projectSettings.value.leftVersion = projectSettings.value.leftBranch;
    projectSettings.value.rightVersion = projectSettings.value.rightBranch;
  } else {
    // 切换回 commit 比对模式：恢复默认
    projectSettings.value.leftVersion = 'HEAD';
    projectSettings.value.rightVersion = 'WORKING';
  }
  await saveAndRefreshVersions();
};

// 分支选择变更处理
const onBranchChange = async () => {
  if (projectSettings.value.compareBranches) {
    // 分支比对模式下，直接使用分支名作为版本
    projectSettings.value.leftVersion = projectSettings.value.leftBranch;
    projectSettings.value.rightVersion = projectSettings.value.rightBranch;
    await saveAndRefreshVersions();
  }
};

// DiffViewer 版本变更处理
const onDiffOldVersionChange = async (version: string) => {
  projectSettings.value.leftVersion = version;
  onOldVersionChange();
  await saveAndRefreshVersions();
};

const onDiffNewVersionChange = async (version: string) => {
  projectSettings.value.rightVersion = version;
  await saveAndRefreshVersions();
};

// DiffViewer 分支变更处理
const onDiffOldBranchChange = async (branch: string) => {
  projectSettings.value.leftBranch = branch;
  projectSettings.value.leftVersion = branch;
  await saveAndRefreshVersions();
};

const onDiffNewBranchChange = async (branch: string) => {
  projectSettings.value.rightBranch = branch;
  projectSettings.value.rightVersion = branch;
  await saveAndRefreshVersions();
};

// 保存版本设置并刷新
const saveAndRefreshVersions = async () => {
  // 保存到内存 Map（不持久化，重启后恢复默认）
  if (projectSettings.value.path) {
    projectVersionSettings.set(projectSettings.value.path, {
      leftVersion: projectSettings.value.leftVersion,
      rightVersion: projectSettings.value.rightVersion,
      compareBranches: projectSettings.value.compareBranches,
      leftBranch: projectSettings.value.leftBranch,
      rightBranch: projectSettings.value.rightBranch
    });
  }

  // 清除当前项目的 diff 缓存
  const projectPath = currentPath.value;
  for (const cacheKey of diffCache.value.keys()) {
    if (cacheKey.startsWith(`${projectPath}:`)) {
      diffCache.value.delete(cacheKey);
    }
  }

  // 刷新当前显示的文件并更新视图
  if (currentFile.value) {
    const diffResult = await loadFileDiff(currentFile.value, true);
    if (diffResult) {
      leftLines.value = diffResult.leftLines;
      rightLines.value = diffResult.rightLines;
      isBinary.value = diffResult.isBinary;
      diffStats.value = diffResult.diffStats;

      // 更新当前激活的标签页
      if (activeTabId.value) {
        const activeTab = tabs.value.find(t => t.id === activeTabId.value);
        if (activeTab) {
          activeTab.leftLines = diffResult.leftLines;
          activeTab.rightLines = diffResult.rightLines;
          activeTab.isBinary = diffResult.isBinary;
          activeTab.diffStats = diffResult.diffStats;
        }
      }
    }
  }

  // 刷新更改列表
  await loadStagedFiles();
};

// 打开项目设置
const openProjectSettings = async (project: Project) => {
  projectSettings.value.name = project.name;
  projectSettings.value.path = project.path;
  showProjectSettings.value = true;

  // 加载保存的比对版本设置
  loadCompareVersions(project.path);

  // 获取分支列表
  try {
    const branches = await invoke<string[]>('get_git_branches', { repoPath: project.path });
    branchList.value = branches;
    if (branches.length > 0 && !branches.includes(projectSettings.value.defaultBranch)) {
      projectSettings.value.defaultBranch = branches[0];
    }
  } catch (error) {
    console.error('获取分支列表失败:', error);
    branchList.value = [];
  }

  // 获取 commit 历史
  try {
    const commits = await invoke<CommitInfo[]>('get_commit_history', { repoPath: project.path, limit: 30 });
    commitList.value = commits;
  } catch (error) {
    console.error('获取 commit 历史失败:', error);
    commitList.value = [];
  }
};

// 保存项目设置
const saveProjectSettings = async () => {
  console.log('保存项目设置:', projectSettings.value);
  // 保存比对版本设置到内存 Map（不持久化，重启后恢复默认）
  if (projectSettings.value.path) {
    projectVersionSettings.set(projectSettings.value.path, {
      leftVersion: projectSettings.value.leftVersion,
      rightVersion: projectSettings.value.rightVersion,
      compareBranches: projectSettings.value.compareBranches,
      leftBranch: projectSettings.value.leftBranch,
      rightBranch: projectSettings.value.rightBranch
    });
  }
  showProjectSettings.value = false;

  // 清除当前项目的 diff 缓存
  const projectPath = currentPath.value;
  for (const cacheKey of diffCache.value.keys()) {
    if (cacheKey.startsWith(`${projectPath}:`)) {
      diffCache.value.delete(cacheKey);
    }
  }

  // 刷新当前显示的文件
  if (currentFile.value) {
    await loadFileDiff(currentFile.value);
  }

  // 刷新更改列表
  await loadStagedFiles();
};

// 加载比对版本设置（从内存 Map 加载，不持久化）
const loadCompareVersions = (projectPath: string) => {
  try {
    const saved = projectVersionSettings.get(projectPath);
    if (saved) {
      projectSettings.value.leftVersion = saved.leftVersion || '';
      projectSettings.value.rightVersion = saved.rightVersion || 'WORKING';
      projectSettings.value.compareBranches = saved.compareBranches || false;
      projectSettings.value.leftBranch = saved.leftBranch || projectSettings.value.defaultBranch || 'main';
      projectSettings.value.rightBranch = saved.rightBranch || projectSettings.value.defaultBranch || 'main';
    } else {
      // 没有保存的设置，恢复默认值（空字符串表示使用工作区模式）
      projectSettings.value.leftVersion = '';
      projectSettings.value.rightVersion = 'WORKING';
      projectSettings.value.compareBranches = false;
      projectSettings.value.leftBranch = projectSettings.value.defaultBranch || 'main';
      projectSettings.value.rightBranch = projectSettings.value.defaultBranch || 'main';
    }
  } catch (e) {
    console.error('加载比对版本设置失败:', e);
  }
};

// 全局搜索对话框
const globalSearchDialog = ref<InstanceType<typeof GlobalSearchDialog> | null>(null);
const promptTitle = ref('');
const promptMessage = ref('');
const promptValue = ref('');

// 监听软件设置弹窗打开，加载数据
watch(showAppSettings, (newVal) => {
  if (newVal) {
    loadSavedWorkspaces();
    loadPluginsFromSettings();
  }
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

// 当前分支
const currentBranch = ref('');

// 获取当前项目
const currentProject = computed(() => {
  return projects.value.find(p => p.id === currentProjectId.value) || null;
});

// 获取文件扩展名
const getFileExtension = (filename: string): string => {
  const parts = filename.split('.');
  return parts.length > 1 ? parts[parts.length - 1].toUpperCase() : '';
};

// 获取当前分支
const fetchCurrentBranch = async () => {
  const project = currentProject.value;
  if (!project) {
    currentBranch.value = '';
    return;
  }
  try {
    const branch = await invoke<string>('get_current_branch', { repoPath: project.path });
    currentBranch.value = branch;
  } catch (error) {
    console.error('获取当前分支失败:', error);
    currentBranch.value = '';
  }
};

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
let unlistenToolbarClick: (() => void) | null = null;

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

  // 等待 Tauri 完全准备好后再加载工作区和项目
  setTimeout(async () => {
    await loadWorkspaces();
  }, 500);

  // 设置初始窗口标题
  getCurrentWindow().setTitle('Giter Compare');

  // 添加全局快捷键监听
  window.addEventListener('keydown', handleGlobalKeyDown);

  // 在 refactor/components 分支上禁用原生工具栏，始终使用网页工具栏
  useNativeToolbar.value = false;
  console.log('原生工具栏已禁用（refactor/components 分支）');

  unlistenFileChange = await listen('file-changed', (event: any) => {
    console.log('File changed event received:', event.payload);
    // 检查是否是结构变化（新增/删除文件或文件夹）
    const payload = event.payload;
    const isStructuralChange = payload?.is_structural_change === true;
    const changedFilePath = payload?.path;
    const eventRepoPath = payload?.repo_path;

    // 只有当事件来自当前项目时才处理
    if (eventRepoPath && eventRepoPath !== currentPath.value) {
      console.log('Event from different project, ignoring');
      return;
    }

    if (isStructuralChange && currentPath.value) {
      // 文件结构变化，清除当前项目的文件树缓存
      clearFileTreeCache(currentPath.value);
    }

    // 清除变更文件的 diff 缓存
    if (changedFilePath && currentPath.value) {
      const diffCacheKey = getDiffCacheKey(currentPath.value, changedFilePath);
      if (diffCache.value.has(diffCacheKey)) {
        diffCache.value.delete(diffCacheKey);
        console.log('Cleared diff cache for changed file:', changedFilePath);
      }
    }

    // 无论是否有 currentPath，都尝试刷新
    refresh();
  });

  // 启动轮询检查原生工具栏按钮点击
  const pollToolbarButtons = async () => {
    try {
      const buttonId = await invoke('poll_toolbar_button_click') as string | null;
      if (buttonId) {
        console.log('原生工具栏按钮点击:', buttonId);
        switch (buttonId) {
          case 'settings':
            showAppSettings.value = true;
            break;
          case 'prev':
            navigatePrev();
            break;
          case 'next':
            navigateNext();
            break;
          case 'refresh':
            refresh();
            break;
          default:
            console.log('未知的工具栏按钮:', buttonId);
        }
      }
    } catch (error) {
      console.error('轮询工具栏按钮失败:', error);
    }
  };

  // 每 100ms 轮询一次
  const pollInterval = setInterval(pollToolbarButtons, 100);

  // 保存清理函数
  unlistenToolbarClick = () => {
    clearInterval(pollInterval);
  };
});

onUnmounted(() => {
  if (unlistenFileChange) {
    unlistenFileChange();
  }
  if (unlistenToolbarClick) {
    unlistenToolbarClick();
  }
  // 移除全局快捷键监听
  window.removeEventListener('keydown', handleGlobalKeyDown);
});

// 主题切换
const toggleTheme = () => {
  theme.value = theme.value === 'dark' ? 'light' : 'dark';
  localStorage.setItem('theme', theme.value);
  // 设置 body 的 data-theme 属性，让对话框也能继承主题
  document.body.setAttribute('data-theme', theme.value);
};

const setTheme = (newTheme: string) => {
  theme.value = newTheme;
  localStorage.setItem('theme', theme.value);
  document.body.setAttribute('data-theme', theme.value);
};

// ========== 软件设置 - 工作区管理函数 ==========

const loadSavedWorkspaces = async () => {
  try {
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    const { appDataDir, join } = await import('@tauri-apps/api/path');
    const appData = await appDataDir();
    const filePath = await join(appData, 'workspaces.json');
    const content = await readTextFile(filePath);
    savedWorkspaces.value = JSON.parse(content);
  } catch (e) {
    savedWorkspaces.value = [];
  }
};

const saveWorkspacesToFile = async () => {
  try {
    const { writeTextFile, mkdir } = await import('@tauri-apps/plugin-fs');
    const { appDataDir, join } = await import('@tauri-apps/api/path');
    const { BaseDirectory } = await import('@tauri-apps/plugin-fs');
    await mkdir('', { baseDir: BaseDirectory.AppData, recursive: true });
    const appData = await appDataDir();
    const filePath = await join(appData, 'workspaces.json');
    await writeTextFile(filePath, JSON.stringify(savedWorkspaces.value, null, 2));
  } catch (e) {
    console.error('Failed to save workspaces:', e);
  }
};

const importProjectsFromFolder = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: true,
      title: '选择 Git 项目文件夹'
    });
    if (selected && Array.isArray(selected)) {
      const existingPaths = new Set(projects.value.map(p => p.path));
      const newProjects: Project[] = [];
      for (const path of selected) {
        if (!existingPaths.has(path)) {
          const parts = path.split(/[\\/]/);
          newProjects.push({
            id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
            name: parts[parts.length - 1] || parts[parts.length - 2] || '新项目',
            path
          });
        }
      }
      if (newProjects.length > 0) {
        projects.value.push(...newProjects);
        saveWorkspaces();
      }
    }
  } catch (e) {
    console.error('Failed to import from folder:', e);
  }
};

const saveCurrentWorkspaceFromSettings = async () => {
  const name = newWorkspaceName.value.trim();
  if (!name) return;
  const workspace: Workspace = {
    id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
    name,
    projects: [...projects.value],
    createdAt: new Date().toISOString()
  };
  savedWorkspaces.value.unshift(workspace);
  await saveWorkspacesToFile();
  newWorkspaceName.value = '';
};

const loadWorkspaceFromSettings = (workspace: Workspace) => {
  const existingPaths = new Set(projects.value.map(p => p.path));
  for (const project of workspace.projects) {
    if (!existingPaths.has(project.path)) {
      projects.value.push({ ...project });
    }
  }
  saveWorkspaces();
};

const deleteWorkspaceFromSettings = async (id: string) => {
  const index = savedWorkspaces.value.findIndex(w => w.id === id);
  if (index > -1) {
    savedWorkspaces.value.splice(index, 1);
    await saveWorkspacesToFile();
  }
};

const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
};

// ========== 软件设置 - 插件管理函数 ==========

const loadPluginsFromSettings = async () => {
  pluginLoading.value = true;
  try {
    const result = await invoke<PluginInfo[]>('get_installed_plugins');
    installedPlugins.value = result;
  } catch (e) {
    console.error('Failed to load plugins:', e);
  } finally {
    pluginLoading.value = false;
  }
};

const importPluginFromSettings = async () => {
  pluginImportError.value = '';
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 VSCode 语法高亮插件文件夹'
    });
    if (selected && typeof selected === 'string') {
      pluginImporting.value = true;
      try {
        await invoke<PluginInfo>('import_vscode_plugin', {
          pluginPath: selected
        });
        await loadPluginsFromSettings();
        onPluginsChanged();
      } catch (e) {
        pluginImportError.value = `导入失败: ${e}`;
      } finally {
        pluginImporting.value = false;
      }
    }
  } catch (e) {
    pluginImportError.value = `选择文件夹失败: ${e}`;
  }
};

const removePluginFromSettings = async (pluginId: string) => {
  if (!confirm('确定要删除这个插件吗？')) return;
  try {
    await invoke('remove_plugin', { pluginId });
    await loadPluginsFromSettings();
    onPluginsChanged();
  } catch (e) {
    alert(`删除失败: ${e}`);
  }
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
    let leftContent = '';
    let rightContent = '';

    // 获取旧版本内容（左边）
    const oldVersion = projectSettings.value.leftVersion;
    if (oldVersion && oldVersion !== 'WORKING') {
      try {
        leftContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: oldVersion
        });
      } catch (e) {
        leftContent = '';
      }
    } else {
      // 工作区
      if (fileStatus === 'deleted') {
        leftContent = '';
      } else {
        try {
          leftContent = await invoke<string>('read_file_content', {
            filePath: `${currentPath.value}/${file.path}`
          });
        } catch (e) {
          leftContent = '';
        }
      }
    }

    // 获取新版本内容（右边）
    const newVersion = projectSettings.value.rightVersion;
    if (newVersion && newVersion !== 'WORKING') {
      try {
        rightContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: newVersion
        });
      } catch (e) {
        rightContent = '';
      }
    } else {
      // 工作区
      if (fileStatus === 'deleted') {
        rightContent = '';
      } else {
        try {
          rightContent = await invoke<string>('read_file_content', {
            filePath: `${currentPath.value}/${file.path}`
          });
        } catch (e) {
          rightContent = '';
        }
      }
    }

    const isBinaryFile = leftContent === '[二进制文件]' || rightContent === '[二进制文件]';

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
      const oldLines = leftContent.split('\n');

      for (let i = 0; i < oldLines.length; i++) {
        alignedLeftLines.push({
          lineNum: i + 1,
          content: oldLines[i],
          changeType: 'removed',
          isDiff: true
        });
        alignedRightLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
      }

      leftLines.value = alignedLeftLines;
      rightLines.value = alignedRightLines;
      isBinary.value = false;
      diffStats.value = { added: 0, removed: oldLines.length, changed: 0 };
      currentFile.value = file;
      return;
    }

    // 对于新增的文件，直接显示完整的新文件内容
    if (fileStatus === 'added') {
      const alignedLeftLines: DiffLine[] = [];
      const alignedRightLines: DiffLine[] = [];
      const newLines = rightContent.split('\n');

      for (let i = 0; i < newLines.length; i++) {
        alignedLeftLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: i + 1,
          content: newLines[i],
          changeType: 'added',
          isDiff: true
        });
      }

      leftLines.value = alignedLeftLines;
      rightLines.value = alignedRightLines;
      isBinary.value = false;
      diffStats.value = { added: newLines.length, removed: 0, changed: 0 };
      currentFile.value = file;
      return;
    }

    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent: leftContent,
      newContent: rightContent
    });

    // 构建 diff 行（复用 loadFileDiff 的逻辑）
    const result = buildDiffLines(diffResult);

    leftLines.value = result.leftLines;
    rightLines.value = result.rightLines;
    isBinary.value = false;
    diffStats.value = result.diffStats;
    currentFile.value = file;

    // 加载 blame 信息
    await loadBlameInfo(file.path);
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
    localStorage.removeItem('giter-current-project');
  }
};

const switchProject = async (project: Project) => {
  currentProjectId.value = project.id;
  currentPath.value = project.path;

  // 保存当前选择的项目ID
  localStorage.setItem('giter-current-project', project.id);

  // 清除更改列表，避免显示上一个项目的更改
  stagedFiles.value = [];
  selectedStagedPath.value = '';
  
  // 加载保存的版本设置
  loadCompareVersions(project.path);
  
  // 加载 commit 历史
  try {
    const commits = await invoke<CommitInfo[]>('get_commit_history', { repoPath: project.path, limit: 30 });
    commitList.value = commits;
  } catch (error) {
    console.error('获取 commit 历史失败:', error);
    commitList.value = [];
  }
  
  await loadFileTree(project.path);
  await loadStagedFiles(); // 重新加载当前项目的更改列表
  await invoke('start_file_watcher', { repoPath: project.path });
  
  // 获取当前分支
  await fetchCurrentBranch();
  
  // 更新窗口标题
  updateWindowTitle(project);
};

// 更新窗口标题
const updateWindowTitle = (project: Project) => {
  const projectName = project.name || project.path.split('/').pop() || 'Giter Compare';
  const newTitle = `${projectName} - Giter Compare`;
  console.log('Updating window title to:', newTitle);
  const win = getCurrentWindow();
  console.log('Window object:', win);
  win.setTitle(newTitle);
};

// 保存工作区列表到 localStorage
const saveWorkspaces = () => {
  localStorage.setItem('giter-workspaces', JSON.stringify(workspaces.value));
  localStorage.setItem('giter-current-workspace', currentWorkspaceId.value);
};

// 加载工作区列表
const loadWorkspaces = async () => {
  console.log('Loading workspaces...');
  const saved = localStorage.getItem('giter-workspaces');
  if (saved) {
    try {
      workspaces.value = JSON.parse(saved);
      console.log('Workspaces loaded:', workspaces.value.length);
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
      console.log('Projects loaded:', projects.value.length);

      // 自动加载上次选择的项目或第一个项目
      const savedCurrentProject = localStorage.getItem('giter-current-project');
      console.log('Saved current project:', savedCurrentProject);
      if (projects.value.length > 0) {
        const projectToLoad = savedCurrentProject
          ? projects.value.find(p => p.id === savedCurrentProject)
          : projects.value[0];
        console.log('Project to load:', projectToLoad?.name || 'none');
        if (projectToLoad) {
          console.log('Auto-switching to project:', projectToLoad.name);
          await switchProject(projectToLoad);
        }
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
  localStorage.removeItem('giter-current-project');
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

// 加载更改的文件列表（工作区中已修改但未暂存的文件，或两个版本之间的差异文件）
const loadStagedFiles = async () => {
  if (!currentPath.value) return;

  try {
    let changedFiles: GitStatus[] = [];
    
    // 检查是否设置了版本对比
    const oldVersion = projectSettings.value.leftVersion;
    const newVersion = projectSettings.value.rightVersion;
    
    // 只有当两个版本都是 commit hash（不是 WORKING）时，才使用版本对比
    const isOldVersionCommit = oldVersion && oldVersion !== 'WORKING';
    const isNewVersionCommit = newVersion && newVersion !== 'WORKING';
    
    if (isOldVersionCommit || isNewVersionCommit) {
      // 使用版本对比获取差异文件
      console.log('Using version diff:', oldVersion, '->', newVersion);
      changedFiles = await invoke<GitStatus[]>('get_diff_between_versions', {
        repoPath: currentPath.value,
        oldVersion: oldVersion || 'HEAD',
        newVersion: newVersion || 'WORKING'
      });
    } else {
      // 获取工作区的变更（未暂存的文件）
      console.log('Using working tree changes');
      changedFiles = await invoke<GitStatus[]>('get_working_tree_changes', {
        repoPath: currentPath.value
      });
    }

    // 过滤掉未跟踪的文件，只显示已修改、已删除、已重命名的文件
    const filteredFiles = changedFiles.filter(change => {
      const status = change.status.toLowerCase();
      return status === 'modified' || status === 'deleted' || status === 'renamed' || status === 'added';
    });

    // 进一步过滤掉文件夹路径（Git 有时会将文件夹报告为变更）
    const fileOnlyChanges = await Promise.all(filteredFiles.map(async change => {
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

    const finalChanges = fileOnlyChanges
      .filter(change => change !== null)
      .map(change => {
        const parts = (change as GitStatus).path.split(/[\\/]/);
        return {
          name: parts[parts.length - 1] || (change as GitStatus).path,
          path: (change as GitStatus).path,
          status: (change as GitStatus).status
        };
      });
    
    stagedFiles.value = finalChanges;
    
    // 同时更新 gitChanges 和文件树状态
    gitChanges.value = finalChanges;
    updateFileTreeStatus(fileTree.value, finalChanges);
    
  } catch (e) {
    console.error('Failed to load changed files:', e);
    stagedFiles.value = [];
    gitChanges.value = [];
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

// 获取 diff 缓存键（包含版本信息）
const getDiffCacheKey = (projectPath: string, filePath: string) => {
  const oldV = projectSettings.value.leftVersion || 'WORKING';
  const newV = projectSettings.value.rightVersion || 'WORKING';
  return `${projectPath}:${filePath}:${oldV}:${newV}`;
};

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
    let leftContent = '';
    let rightContent = '';

    // 获取旧版本内容（左边）
    const oldVersion = projectSettings.value.leftVersion;
    if (oldVersion && oldVersion !== 'WORKING') {
      try {
        leftContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: oldVersion
        });
      } catch (e) {
        leftContent = '';
      }
    } else {
      // 工作区
      if (fileStatus === 'deleted') {
        leftContent = '';
      } else {
        try {
          leftContent = await invoke<string>('read_file_content', {
            filePath: `${currentPath.value}/${file.path}`
          });
        } catch (e) {
          leftContent = '';
        }
      }
    }

    // 获取新版本内容（右边）
    const newVersion = projectSettings.value.rightVersion;
    if (newVersion && newVersion !== 'WORKING') {
      try {
        rightContent = await invoke<string>('get_file_content_at_revision', {
          repoPath: currentPath.value,
          filePath: file.path,
          revision: newVersion
        });
      } catch (e) {
        rightContent = '';
      }
    } else {
      // 工作区
      if (fileStatus === 'deleted') {
        rightContent = '';
      } else {
        try {
          rightContent = await invoke<string>('read_file_content', {
            filePath: `${currentPath.value}/${file.path}`
          });
        } catch (e) {
          rightContent = '';
        }
      }
    }

    const isBinaryFile = leftContent === '[二进制文件]' || rightContent === '[二进制文件]';

    if (isBinaryFile) {
      const result = { leftLines: [], rightLines: [], isBinary: true, diffStats: null };
      setDiffCache(cacheKey, result);
      return result;
    }

    // 对于已删除的文件，直接显示完整的旧文件内容，所有行标记为 removed
    if (fileStatus === 'deleted') {
      const alignedLeftLines: DiffLine[] = [];
      const alignedRightLines: DiffLine[] = [];
      const oldLines = leftContent.split('\n');

      for (let i = 0; i < oldLines.length; i++) {
        alignedLeftLines.push({
          lineNum: i + 1,
          content: oldLines[i],
          changeType: 'removed',
          isDiff: true
        });
        alignedRightLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
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

    // 对于新增的文件，直接显示完整的新文件内容，所有行标记为 added
    if (fileStatus === 'added') {
      const alignedLeftLines: DiffLine[] = [];
      const alignedRightLines: DiffLine[] = [];
      const newLines = rightContent.split('\n');

      for (let i = 0; i < newLines.length; i++) {
        alignedLeftLines.push({
          lineNum: 0,
          content: '',
          changeType: 'empty',
          isDiff: false
        });
        alignedRightLines.push({
          lineNum: i + 1,
          content: newLines[i],
          changeType: 'added',
          isDiff: true
        });
      }

      const result = {
        leftLines: alignedLeftLines,
        rightLines: alignedRightLines,
        isBinary: false,
        diffStats: { added: newLines.length, removed: 0, changed: 0 }
      };
      setDiffCache(cacheKey, result);
      return result;
    }

    const diffResult = await invoke<FileDiff>('compare_strings', {
      oldContent: leftContent,
      newContent: rightContent
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

    // 加载 blame 信息
    await loadBlameInfo(file.path);

    return result;
  } catch (e) {
    console.error('Failed to load diff:', e);
    return null;
  }
};

// 查看文件历史变更
const viewFileHistory = async (filePath: string) => {
  console.log('viewFileHistory called:', filePath);
  if (!currentPath.value) {
    console.log('currentPath is empty');
    return;
  }
  
  fileHistoryFilePath.value = filePath;
  fileHistoryFileName.value = filePath.split('/').pop() || filePath;
  showFileHistory.value = true;
  fileHistoryLoading.value = true;
  fileHistoryList.value = [];
  
  console.log('showFileHistory set to true');
  
  try {
    console.log('calling get_file_history:', currentPath.value, filePath);
    const commits = await invoke<CommitInfo[]>('get_file_history', {
      repoPath: currentPath.value,
      filePath: filePath,
      limit: 50
    });
    console.log('get_file_history result:', commits.length);
    fileHistoryList.value = commits;
  } catch (error) {
    console.error('获取文件历史失败:', error);
    fileHistoryList.value = [];
  } finally {
    fileHistoryLoading.value = false;
  }
};

// 处理"只显示变更"开关变化
const onShowOnlyDiffChange = () => {
  if (currentHistoryCommit.value) {
    viewFileAtCommit(currentHistoryCommit.value);
  }
};

// 查看指定版本的文件内容
const viewFileAtCommit = async (commitHash: string) => {
  if (!currentPath.value || !fileHistoryFilePath.value) return;

  // 获取 commit 信息
  const commit = fileHistoryList.value.find(c => c.hash === commitHash);
  if (!commit) return;

  currentHistoryCommit.value = commitHash;
  historyFileContentCommit.value = `${commit.short_hash} - ${commit.message}`;
  showHistoryFileContent.value = true;
  historyFileContentLoading.value = true;
  historyFileContent.value = '';

  try {
    if (showOnlyDiff.value) {
      // 只显示该版本更新的代码（diff）
      const parentHash = await invoke<string>('get_commit_parent', {
        repoPath: currentPath.value,
        commitHash: commitHash
      });

      const diffResult = await invoke<FileDiff>('get_diff_between_commits', {
        repoPath: currentPath.value,
        oldCommit: parentHash || '4b825dc642cb6eb9a060e54bf8d69288fbee4904', // empty tree hash
        newCommit: commitHash,
        filePath: fileHistoryFilePath.value
      });

      // 只显示有变更的行
      const diffLines: string[] = [];
      diffResult.hunks.forEach(hunk => {
        hunk.lines.forEach(line => {
          if (line.change_type !== 'unchanged') {
            const prefix = line.change_type === 'added' ? '+' : line.change_type === 'removed' ? '-' : ' ';
            diffLines.push(`${prefix} ${line.content}`);
          }
        });
      });

      historyFileContent.value = diffLines.join('\n') || '该版本没有代码变更';
    } else {
      // 获取该版本的完整文件内容
      const content = await invoke<string>('get_file_content_at_revision', {
        repoPath: currentPath.value,
        filePath: fileHistoryFilePath.value,
        revision: commitHash
      });

      historyFileContent.value = content;
    }
  } catch (error: any) {
    console.error('获取文件内容失败:', error);
    historyFileContent.value = '获取文件内容失败: ' + (error?.message || error || '未知错误');
  } finally {
    historyFileContentLoading.value = false;
  }
};

// 处理文件树右击菜单
const handleFileContextMenu = async (action: string, node: FileNode) => {
  console.log('handleFileContextMenu:', action, node.path);
  if (!currentPath.value) {
    console.log('currentPath is empty');
    return;
  }

  switch (action) {
    case 'copyPath':
      {
        const fullPath = await invoke<string>('get_full_path', {
          repoPath: currentPath.value,
          filePath: node.path
        }).catch(() => node.path);
        await navigator.clipboard.writeText(fullPath);
      }
      break;
    case 'copyRelativePath':
      await navigator.clipboard.writeText(node.path);
      break;
    case 'openInFinder':
      {
        const fullPath = await invoke<string>('get_full_path', {
          repoPath: currentPath.value,
          filePath: node.path
        }).catch(() => node.path);
        await invoke('open_in_finder', { path: fullPath });
      }
      break;
    case 'openInEditor':
      {
        const fullPath = await invoke<string>('get_full_path', {
          repoPath: currentPath.value,
          filePath: node.path
        }).catch(() => node.path);
        await invoke('open_in_editor', { path: fullPath });
      }
      break;
    case 'viewHistory':
      // 打开文件历史变更查看
      console.log('calling viewFileHistory for:', node.path);
      await viewFileHistory(node.path);
      break;
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

// 通过路径打开文件
const openFileByPath = async (relativePath: string) => {
  try {
    // 获取文件内容
    const fullPath = `${currentPath.value}/${relativePath}`;
    const content = await invoke<string>('read_file_content', { 
      filePath: fullPath 
    });
    
    // 将内容按行分割
    const lines = content.split('\n').map((line, index) => ({
      lineNum: index + 1,
      content: line,
      changeType: 'same',
      isDiff: false
    }));
    
    // 创建新标签
    const tabId = `file-${Date.now()}`;
    const newTab: Tab = {
      id: tabId,
      name: relativePath.split('/').pop() || relativePath,
      path: relativePath,
      projectPath: currentPath.value || '',
      fileType: 'file',
      isModified: false,
      leftLines: lines,
      rightLines: lines,
      isBinary: false,
      diffStats: { additions: 0, deletions: 0 },
      scrollTop: 0
    };
    
    tabs.value.push(newTab);
    activeTabId.value = tabId;
  } catch (error) {
    console.error('打开文件失败:', error);
    alert('打开文件失败：' + error);
  }
};

// 全局搜索快捷键处理
const handleGlobalKeyDown = (event: KeyboardEvent) => {
  // Ctrl + Alt + F (Windows/Linux) 或 Cmd + Control + F (macOS)
  const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
  if (isMac) {
    if (event.metaKey && event.ctrlKey && event.key === 'f') {
      event.preventDefault();
      if (currentPath.value && globalSearchDialog.value) {
        globalSearchDialog.value.open(currentPath.value);
      }
    }
  } else {
    if (event.ctrlKey && event.altKey && event.key === 'f') {
      event.preventDefault();
      if (currentPath.value && globalSearchDialog.value) {
        globalSearchDialog.value.open(currentPath.value);
      }
    }
  }
};

// 全局搜索打开文件处理
const handleGlobalSearchOpenFile = (path: string, lineNumber?: number, searchText?: string) => {
  // 关闭搜索对话框
  if (globalSearchDialog.value) {
    globalSearchDialog.value.close();
  }
  
  // 获取相对路径
  const relativePath = path.startsWith(currentPath.value) 
    ? path.substring(currentPath.value.length).replace(/^[/\\]/, '')
    : path;
  
  // 在文件树中查找该文件
  const findFileInTree = (nodes: FileNode[], targetPath: string): FileNode | null => {
    for (const node of nodes) {
      if (node.path === targetPath) {
        return node;
      }
      if (node.children) {
        const found = findFileInTree(node.children, targetPath);
        if (found) return found;
      }
    }
    return null;
  };
  
  const fileNode = findFileInTree(fileTree.value, relativePath);
  
  if (fileNode) {
    // 文件在文件树中，激活差异对比
    selectFile(fileNode.path);
    
    // 等待差异对比加载完成后跳转
    nextTick(() => {
      setTimeout(() => {
        console.log('触发跳转到行:', lineNumber, '搜索词:', searchText);
        const event = new CustomEvent('jump-to-line', { 
          detail: { 
            lineNumber,
            searchText
          } 
        });
        window.dispatchEvent(event);
      }, 300);
    });
  } else {
    // 文件不在文件树中（可能是未跟踪的文件），打开文件查看
    openFileByPath(relativePath).then(() => {
      // 跳转到指定行
      setTimeout(() => {
        const event = new CustomEvent('jump-to-line', { detail: lineNumber });
        window.dispatchEvent(event);
      }, 100);
    });
  }
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
  z-index: 10000;
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

/* 软件设置弹窗样式 */
.settings-dialog-overlay {
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

.settings-dialog {
  background-color: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  width: 90%;
  max-width: 500px;
  max-height: 85vh;
  overflow: hidden;
  display: flex;
  flex-direction: column;
}

.settings-body {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.settings-actions {
  padding: 16px 24px;
  border-top: 1px solid #f0f0f0;
  display: flex;
  justify-content: flex-end;
  gap: 12px;
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

/* 项目设置弹窗样式 */
.project-settings-dialog {
  background-color: #ffffff;
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.15);
  width: 560px;
  max-height: 85vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* 文件历史弹窗 */
.file-history-dialog {
  width: 480px;
}

.file-history-content {
  padding: 0;
}

.history-loading,
.history-empty {
  padding: 40px;
  text-align: center;
  color: #888;
  font-size: 14px;
}

.history-list {
  padding: 8px 0;
}

.history-item {
  padding: 12px 20px;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
  transition: background-color 0.15s;
}

.history-item:hover {
  background-color: #f5f7fa;
}

.history-item-first {
  background-color: #f0f7ff;
}

.history-item-first:hover {
  background-color: #e0f0ff;
}

.history-commit-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 6px;
}

.history-hash {
  font-family: 'SF Mono', Monaco, monospace;
  font-size: 12px;
  color: #4a7eff;
  background-color: #eef2ff;
  padding: 2px 8px;
  border-radius: 4px;
}

.history-date {
  font-size: 11px;
  color: #999;
}

.history-message {
  font-size: 13px;
  color: #333;
  font-weight: 500;
  margin-bottom: 4px;
  line-height: 1.4;
}

.history-author {
  font-size: 11px;
  color: #888;
}

/* 文件内容弹窗 */
.file-content-dialog {
  width: 800px;
  max-width: 90vw;
  height: 70vh;
  max-height: 80vh;
}

.file-content-commit-info {
  padding: 8px 20px;
  background-color: #f5f7fa;
  border-bottom: 1px solid #e8e8e8;
  font-size: 12px;
  color: #666;
  font-family: 'SF Mono', Monaco, monospace;
}

.file-content-body {
  flex: 1;
  overflow: auto;
  padding: 0;
  background-color: #fafafa;
}

.file-content-pre {
  margin: 0;
  padding: 16px 20px;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Fira Code', monospace;
  font-size: 13px;
  line-height: 1.6;
  color: #333;
  white-space: pre-wrap;
  word-wrap: break-word;
  background-color: transparent;
}

.file-content-pre code {
  font-family: inherit;
  background-color: transparent;
  padding: 0;
}

/* 只显示变更开关 - 现代 Toggle Switch 样式 */
.diff-toggle {
  display: flex;
  align-items: center;
  gap: 14px;
  cursor: pointer;
  margin-left: auto;
  user-select: none;
  margin-top: 8px;
}

.diff-toggle input[type="checkbox"] {
  appearance: none;
  -webkit-appearance: none;
  width: 36px;
  height: 20px;
  background-color: #d1d5db;
  border-radius: 10px;
  position: relative;
  cursor: pointer;
  transition: background-color 0.2s ease;
  outline: none;
  flex-shrink: 0;
}

.diff-toggle input[type="checkbox"]::before {
  content: '';
  position: absolute;
  width: 16px;
  height: 16px;
  background-color: white;
  border-radius: 50%;
  top: 2px;
  left: 2px;
  transition: transform 0.2s ease;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.diff-toggle input[type="checkbox"]:checked {
  background-color: #4a7eff;
}

.diff-toggle input[type="checkbox"]:checked::before {
  transform: translateX(16px);
}

.diff-toggle input[type="checkbox"]:hover:not(:checked) {
  background-color: #b8bcc4;
}

.diff-toggle-label {
  font-size: 12px;
  color: #555;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  font-weight: 500;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid #f0f0f0;
  background: linear-gradient(135deg, #fafafa 0%, #f5f5f5 100%);
}

.settings-header .close-btn {
  background: none;
  border: none;
  font-size: 28px;
  color: #888;
  cursor: pointer;
  padding: 0;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  transition: all 0.2s;
  line-height: 1;
}

.settings-header .close-btn:hover {
  background-color: #f0f0f0;
  color: #333;
}

.settings-header h3 {
  margin: 0;
  font-size: 17px;
  font-weight: 600;
  color: #1a1a1a;
  letter-spacing: -0.3px;
}

.settings-content {
  padding: 24px;
  overflow-y: auto;
  flex: 1;
}

.settings-section {
  margin-bottom: 28px;
  background: #fafbfc;
  border-radius: 10px;
  padding: 20px;
  border: 1px solid #f0f0f0;
}

.settings-section:last-child {
  margin-bottom: 0;
}

.settings-section h4 {
  margin: 0 0 16px 0;
  font-size: 12px;
  font-weight: 600;
  color: #4a7eff;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.settings-row {
  display: flex;
  align-items: center;
  margin-bottom: 14px;
  gap: 16px;
}

.settings-row:last-child {
  margin-bottom: 0;
}

.settings-row.checkbox {
  gap: 10px;
  padding: 6px 0;
}

.settings-row.checkbox label {
  margin-bottom: 0;
  cursor: pointer;
  font-size: 13px;
  color: #444;
  font-weight: 400;
  user-select: none;
}

.settings-row.checkbox input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
  margin: 0;
  accent-color: #4a7eff;
  border-radius: 4px;
}

.settings-label {
  width: 110px;
  flex-shrink: 0;
  font-size: 13px;
  color: #555;
  font-weight: 500;
  text-align: left;
  line-height: 1.4;
}

.input-group {
  display: flex;
  gap: 8px;
  margin-top: 16px;
  align-items: center;
}

.settings-input {
  flex: 1;
  padding: 9px 14px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  font-size: 13px;
  color: #333;
  background-color: #fff;
  transition: all 0.2s ease;
  box-sizing: border-box;
  min-width: 0;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.04);
}

.settings-input:focus {
  outline: none;
  border-color: #4a7eff;
  box-shadow: 0 0 0 3px rgba(74, 126, 255, 0.12), inset 0 1px 2px rgba(0,0,0,0.04);
}

.settings-input[readonly] {
  background-color: #f8f9fa;
  color: #888;
  border-color: #e8e8e8;
}

.settings-select {
  cursor: pointer;
  appearance: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  padding: 9px 36px 9px 14px;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23666' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
  background-size: 12px;
}

.settings-textarea {
  flex: 1;
  padding: 10px 14px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  font-size: 13px;
  color: #333;
  background-color: #fff;
  resize: vertical;
  font-family: 'SF Mono', Monaco, 'Cascadia Code', monospace;
  box-sizing: border-box;
  min-width: 0;
  line-height: 1.5;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.04);
}

.settings-textarea:focus {
  outline: none;
  border-color: #4a7eff;
  box-shadow: 0 0 0 3px rgba(74, 126, 255, 0.12), inset 0 1px 2px rgba(0,0,0,0.04);
}

.settings-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid #f0f0f0;
  background-color: #fafafa;
}

.settings-actions .btn {
  padding: 9px 20px;
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.settings-actions .btn-secondary {
  background-color: #fff;
  border: 1px solid #ddd;
  color: #555;
}

.settings-actions .btn-secondary:hover {
  background-color: #f5f5f5;
  border-color: #ccc;
}

.settings-actions .btn-primary {
  background: linear-gradient(135deg, #4a7eff 0%, #3a6eef 100%);
  color: #fff;
  box-shadow: 0 2px 8px rgba(74, 126, 255, 0.3);
}

.settings-actions .btn-primary:hover {
  background: linear-gradient(135deg, #3a6eef 0%, #2a5edf 100%);
  box-shadow: 0 4px 12px rgba(74, 126, 255, 0.4);
  transform: translateY(-1px);
}

/* ========== 软件设置弹窗样式 ========== */

.app-settings-dialog {
  width: 90%;
  max-width: 800px;
  max-height: 85vh;
}

.app-settings-container {
  display: flex;
  flex: 1;
  overflow: hidden;
  min-height: 400px;
}

.app-settings-tabs {
  width: 160px;
  background-color: #f8f9fa;
  border-right: 1px solid #e8e8e8;
  padding: 16px 0;
  flex-shrink: 0;
}

.app-settings-tab {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 20px;
  cursor: pointer;
  transition: all 0.2s;
  border-left: 3px solid transparent;
  color: #666;
  font-size: 14px;
}

.app-settings-tab:hover {
  background-color: #f0f1f2;
  color: #333;
}

.app-settings-tab.active {
  background-color: #fff;
  border-left-color: #4a7eff;
  color: #4a7eff;
  font-weight: 500;
}

.tab-icon {
  font-size: 16px;
}

.tab-label {
  font-size: 14px;
}

.app-settings-content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.app-settings-panel h4 {
  margin: 0 0 20px 0;
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
}

.settings-subsection {
  margin-bottom: 24px;
  padding: 16px;
  background: #fafbfc;
  border-radius: 10px;
  border: 1px solid #f0f0f0;
}

.settings-subsection h5 {
  margin: 0 0 12px 0;
  font-size: 13px;
  font-weight: 600;
  color: #4a7eff;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

/* 主题选项 */
.theme-options {
  display: flex;
  gap: 16px;
  margin-top: 8px;
}

.theme-option {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px 24px;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
  background: #fff;
}

.theme-option:hover {
  border-color: #b0b0b0;
}

.theme-option.active {
  border-color: #4a7eff;
  background: #f0f4ff;
}

.theme-preview {
  width: 48px;
  height: 48px;
  border-radius: 8px;
  border: 1px solid #ddd;
}

.light-preview {
  background: linear-gradient(135deg, #ffffff 0%, #f5f5f5 100%);
}

.dark-preview {
  background: linear-gradient(135deg, #2d2d2d 0%, #1a1a1a 100%);
}

/* 工作区列表 */
.current-projects-mini {
  margin-bottom: 12px;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
  overflow: hidden;
}

.projects-header {
  font-size: 12px;
  font-weight: 500;
  color: #666;
  padding: 8px 12px;
  background: #f0f0f0;
  border-bottom: 1px solid #e0e0e0;
}

.projects-list-mini {
  max-height: 150px;
  overflow-y: auto;
  padding: 8px;
}

.project-item-mini {
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 6px 8px;
  border-radius: 4px;
}

.project-item-mini:hover {
  background: #f0f0f0;
}

.project-item-mini .project-name {
  font-size: 13px;
  font-weight: 500;
  color: #333;
}

.project-item-mini .project-path {
  font-size: 11px;
  color: #999;
}

.no-projects {
  padding: 16px;
  text-align: center;
  color: #999;
  font-size: 13px;
}

.workspace-list-mini {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.workspace-item-mini {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
}

.workspace-item-mini:hover {
  border-color: #c0c0c0;
}

.workspace-item-mini .workspace-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.workspace-item-mini .workspace-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.workspace-item-mini .workspace-meta {
  font-size: 12px;
  color: #999;
}

.workspace-item-mini .workspace-actions {
  display: flex;
  gap: 6px;
}

/* 插件列表 */
.builtin-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 12px;
  background: #e8f2ff;
  border-radius: 6px;
  margin-bottom: 16px;
  font-size: 13px;
  color: #4a7eff;
}

.plugin-list-mini {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.plugin-item-mini {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: #fff;
  border: 1px solid #e0e0e0;
  border-radius: 8px;
}

.plugin-item-mini .plugin-info {
  flex: 1;
}

.plugin-item-mini .plugin-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.plugin-item-mini .plugin-name {
  font-size: 14px;
  font-weight: 500;
  color: #333;
}

.plugin-item-mini .plugin-version {
  font-size: 12px;
  color: #999;
}

.plugin-item-mini .plugin-description {
  font-size: 12px;
  color: #666;
  margin-bottom: 6px;
}

.plugin-grammars {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.grammar-tag {
  font-size: 11px;
  padding: 2px 8px;
  background: #f0f0f0;
  border-radius: 4px;
  color: #666;
}

.import-hint {
  margin-left: 12px;
  font-size: 12px;
  color: #999;
}

.import-error {
  margin-top: 8px;
  padding: 8px 12px;
  background: #fff2f0;
  border: 1px solid #ffccc7;
  border-radius: 6px;
  color: #ff4d4f;
  font-size: 13px;
}

.loading-state,
.empty-state {
  padding: 24px;
  text-align: center;
  color: #999;
  font-size: 13px;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
}

/* ========== 软件设置弹窗深色模式适配 ========== */

/* 深色模式下使用 CSS 变量 */
[data-theme="dark"] .app-settings-dialog .settings-header {
  background: linear-gradient(135deg, #2d2d30 0%, #252526 100%);
  border-bottom-color: #3e3e42;
}

[data-theme="dark"] .app-settings-dialog .settings-header h3 {
  color: #cccccc;
}

[data-theme="dark"] .app-settings-dialog .settings-header .close-btn {
  color: #858585;
}

[data-theme="dark"] .app-settings-dialog .settings-header .close-btn:hover {
  background-color: #3e3e42;
  color: #cccccc;
}

[data-theme="dark"] .app-settings-tabs {
  background-color: #252526;
  border-right-color: #3e3e42;
}

[data-theme="dark"] .app-settings-tab {
  color: #858585;
}

[data-theme="dark"] .app-settings-tab:hover {
  background-color: #2a2d2e;
  color: #cccccc;
}

[data-theme="dark"] .app-settings-tab.active {
  background-color: #1e1e1e;
  border-left-color: #007acc;
  color: #007acc;
}

[data-theme="dark"] .app-settings-content {
  background-color: #1e1e1e;
}

[data-theme="dark"] .app-settings-panel h4 {
  color: #cccccc;
}

[data-theme="dark"] .settings-subsection {
  background: #252526;
  border-color: #3e3e42;
}

[data-theme="dark"] .settings-subsection h5 {
  color: #58a6ff;
}

[data-theme="dark"] .theme-option {
  background: #252526;
  border-color: #3e3e42;
  color: #cccccc;
}

[data-theme="dark"] .theme-option:hover {
  border-color: #555;
}

[data-theme="dark"] .theme-option.active {
  border-color: #007acc;
  background: #1e3a5f;
}

[data-theme="dark"] .current-projects-mini {
  border-color: #3e3e42;
}

[data-theme="dark"] .projects-header {
  background: #2d2d30;
  color: #858585;
  border-bottom-color: #3e3e42;
}

[data-theme="dark"] .project-item-mini:hover {
  background: #2a2d2e;
}

[data-theme="dark"] .project-item-mini .project-name {
  color: #cccccc;
}

[data-theme="dark"] .project-item-mini .project-path {
  color: #858585;
}

[data-theme="dark"] .no-projects {
  color: #858585;
}

[data-theme="dark"] .workspace-item-mini {
  background: #252526;
  border-color: #3e3e42;
}

[data-theme="dark"] .workspace-item-mini:hover {
  border-color: #555;
}

[data-theme="dark"] .workspace-item-mini .workspace-name {
  color: #cccccc;
}

[data-theme="dark"] .workspace-item-mini .workspace-meta {
  color: #858585;
}

[data-theme="dark"] .builtin-notice {
  background: #1e3a5f;
  color: #58a6ff;
}

[data-theme="dark"] .plugin-item-mini {
  background: #252526;
  border-color: #3e3e42;
}

[data-theme="dark"] .plugin-item-mini .plugin-name {
  color: #cccccc;
}

[data-theme="dark"] .plugin-item-mini .plugin-version {
  color: #858585;
}

[data-theme="dark"] .plugin-item-mini .plugin-description {
  color: #858585;
}

[data-theme="dark"] .grammar-tag {
  background: #2d2d30;
  color: #858585;
}

[data-theme="dark"] .import-hint {
  color: #858585;
}

[data-theme="dark"] .import-error {
  background: #4d1e1e;
  border-color: #ff6b6b;
  color: #ffa198;
}

[data-theme="dark"] .loading-state,
[data-theme="dark"] .empty-state {
  color: #858585;
}

[data-theme="dark"] .settings-actions {
  background-color: #252526;
  border-top-color: #3e3e42;
}

[data-theme="dark"] .settings-input {
  background-color: #3c3c3c;
  border-color: #3e3e42;
  color: #cccccc;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.2);
}

[data-theme="dark"] .settings-input::placeholder {
  color: #6e6e6e;
}

[data-theme="dark"] .settings-input:focus {
  border-color: #4a7eff;
  box-shadow: 0 0 0 3px rgba(74, 126, 255, 0.2), inset 0 1px 2px rgba(0,0,0,0.2);
}

[data-theme="dark"] .settings-input[readonly] {
  background-color: #2d2d30;
  color: #6e6e6e;
  border-color: #3e3e42;
}

/* ========== 项目设置弹窗深色模式适配 ========== */

[data-theme="dark"] .project-settings-dialog {
  background-color: #252526;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
}

[data-theme="dark"] .project-settings-dialog .settings-header {
  background: linear-gradient(135deg, #2d2d30 0%, #252526 100%);
  border-bottom-color: #3e3e42;
}

[data-theme="dark"] .project-settings-dialog .settings-header h3 {
  color: #cccccc;
}

[data-theme="dark"] .project-settings-dialog .settings-header .close-btn {
  color: #858585;
}

[data-theme="dark"] .project-settings-dialog .settings-header .close-btn:hover {
  background-color: #3e3e42;
  color: #cccccc;
}

[data-theme="dark"] .project-settings-dialog .settings-content {
  background-color: #252526;
}

[data-theme="dark"] .project-settings-dialog .settings-section {
  background: #2d2d30;
  border-color: #3e3e42;
}

[data-theme="dark"] .project-settings-dialog .settings-section h4 {
  color: #4a7eff;
}

[data-theme="dark"] .project-settings-dialog .settings-label {
  color: #cccccc;
}

[data-theme="dark"] .project-settings-dialog .settings-row.checkbox label {
  color: #cccccc;
}

[data-theme="dark"] .project-settings-dialog .settings-select {
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12'%3E%3Cpath fill='%23cccccc' d='M6 8L1 3h10z'/%3E%3C/svg%3E");
  background-color: #3c3c3c;
  border-color: #3e3e42;
  color: #cccccc;
}

[data-theme="dark"] .project-settings-dialog .settings-textarea {
  background-color: #3c3c3c;
  border-color: #3e3e42;
  color: #cccccc;
  box-shadow: inset 0 1px 2px rgba(0,0,0,0.2);
}

[data-theme="dark"] .project-settings-dialog .settings-textarea:focus {
  border-color: #4a7eff;
  box-shadow: 0 0 0 3px rgba(74, 126, 255, 0.2), inset 0 1px 2px rgba(0,0,0,0.2);
}

[data-theme="dark"] .project-settings-dialog .settings-actions {
  background-color: #252526;
  border-top-color: #3e3e42;
}

/* ========== 文件历史弹窗深色模式适配 ========== */

[data-theme="dark"] .history-loading,
[data-theme="dark"] .history-empty {
  color: #858585;
}

[data-theme="dark"] .history-item {
  border-bottom-color: #3e3e42;
}

[data-theme="dark"] .history-item:hover {
  background-color: #2d2d30;
}

[data-theme="dark"] .history-item-first {
  background-color: #1a3a5c;
}

[data-theme="dark"] .history-item-first:hover {
  background-color: #1e4a70;
}

[data-theme="dark"] .history-hash {
  color: #6b9fff;
  background-color: #1a3a5c;
}

[data-theme="dark"] .history-date {
  color: #858585;
}

[data-theme="dark"] .history-message {
  color: #cccccc;
}

[data-theme="dark"] .history-author {
  color: #858585;
}

/* ========== 文件内容弹窗深色模式适配 ========== */

[data-theme="dark"] .file-content-commit-info {
  background-color: #1e1e1e;
  border-bottom-color: #3e3e42;
  color: #858585;
}

[data-theme="dark"] .file-content-body {
  background-color: #1e1e1e;
}

[data-theme="dark"] .file-content-pre {
  color: #cccccc;
}

[data-theme="dark"] .diff-toggle-label {
  color: #a0a0a0;
}

[data-theme="dark"] .diff-toggle input[type="checkbox"] {
  background-color: #4a4a4a;
}

[data-theme="dark"] .diff-toggle input[type="checkbox"]:hover:not(:checked) {
  background-color: #555555;
}

[data-theme="dark"] .diff-toggle input[type="checkbox"]:checked {
  background-color: #4a7eff;
}

/* 底部状态栏 */
.status-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px;
  height: 22px;
  background-color: var(--accent-color, #4a7eff);
  color: #fff;
  font-size: 12px;
  flex-shrink: 0;
}

.status-bar-left,
.status-bar-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 2px 8px;
  border-radius: 3px;
  cursor: default;
  transition: background-color 0.15s;
}

.status-item:hover {
  background-color: rgba(255, 255, 255, 0.15);
}

.status-icon {
  font-size: 11px;
  line-height: 1;
}

.status-text {
  font-size: 11px;
  line-height: 1;
}

.status-add {
  color: #a5d6a7;
  font-size: 11px;
  font-weight: 500;
}

.status-del {
  color: #ef9a9a;
  font-size: 11px;
  font-weight: 500;
  margin-left: 4px;
}
</style>