<template>
  <DialogBase :open="open" title="工作区管理" @close="$emit('close')">
    <!-- 权限提示弹窗 -->
    <div v-if="showPermissionDialog" class="permission-overlay" @click.self="showPermissionDialog = false">
      <div class="permission-dialog">
        <h3>需要磁盘访问权限</h3>
        <p>导出项目列表需要授予应用磁盘访问权限。</p>
        <p>请按照以下步骤操作：</p>
        <ol>
          <li>打开 <strong>系统设置</strong> → <strong>隐私与安全性</strong> → <strong>完全磁盘访问权限</strong></li>
          <li>点击 <strong>+</strong> 按钮</li>
          <li>找到并添加 <strong>git-compare-tool</strong> 应用</li>
          <li>重启应用</li>
        </ol>
        <div class="permission-actions">
          <button class="btn btn-primary" @click="openSystemSettings">打开系统设置</button>
          <button class="btn btn-secondary" @click="showPermissionDialog = false">稍后再说</button>
        </div>
      </div>
    </div>
    <div class="workspace-manager">
      <!-- 导入项目 -->
      <div class="section import-section">
        <h4>导入项目</h4>
        <button class="btn btn-secondary" @click="importFromFolder">
          从文件夹导入项目
        </button>
      </div>

      <!-- 保存当前工作区 -->
      <div class="section save-section">
        <h4>保存当前工作区</h4>

        <!-- 显示当前项目列表 -->
        <div v-if="localProjects.length > 0" class="current-projects">
          <div class="projects-header">当前项目列表 ({{ localProjects.length }}个):</div>
          <div class="projects-list">
            <div v-for="project in localProjects" :key="project.id" class="project-item">
              <span class="project-name">{{ project.name }}</span>
              <span class="project-path">{{ project.path }}</span>
              <button class="btn btn-icon btn-delete-project" @click.stop="removeCurrentProject(project.id)" title="从列表移除">
                <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                  <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
        <div v-else class="no-projects">
          当前没有项目
        </div>

        <div class="input-group">
          <input
            v-model="workspaceName"
            type="text"
            placeholder="输入工作区名称"
            class="workspace-input"
            @keyup.enter="saveWorkspace"
          />
          <button class="btn btn-primary" @click="saveWorkspace" :disabled="!workspaceName.trim()">
            保存
          </button>
        </div>
        <button class="btn btn-secondary btn-save-current" @click="saveCurrentWorkspace">
          导入当前项目列表
        </button>
      </div>

      <!-- 已保存的工作区列表 -->
      <div class="section workspaces-section">
        <h4>已保存的工作区</h4>
        <div v-if="workspaces.length === 0" class="empty-state">
          暂无保存的工作区
        </div>
        <div v-else class="workspace-list">
          <div
            v-for="workspace in workspaces"
            :key="workspace.id"
            class="workspace-item"
            :class="{ active: selectedWorkspaceId === workspace.id }"
            @click="selectWorkspace(workspace.id)"
          >
            <div class="workspace-info">
              <span class="workspace-name">{{ workspace.name }}</span>
              <span class="workspace-date">{{ formatDate(workspace.createdAt) }}</span>
              <span class="workspace-count">{{ workspace.projects.length }} 个项目</span>
            </div>
            <div class="workspace-actions" style="display: flex !important; gap: 8px !important; visibility: visible !important;">
              <button @click.stop="loadWorkspace(workspace)" title="加载" style="width: 32px !important; height: 32px !important; display: flex !important; align-items: center !important; justify-content: center !important; border: 1px solid #ccc !important; background: white !important; border-radius: 4px !important; cursor: pointer !important; visibility: visible !important; opacity: 1 !important;">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                </svg>
              </button>
              <button @click.stop="deleteWorkspace(workspace.id)" title="删除" style="width: 32px !important; height: 32px !important; display: flex !important; align-items: center !important; justify-content: center !important; border: 1px solid #ccc !important; background: white !important; border-radius: 4px !important; cursor: pointer !important; color: #ef4444 !important; visibility: visible !important; opacity: 1 !important;">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </DialogBase>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import DialogBase from './DialogBase.vue';

interface Project {
  id: string;
  name: string;
  path: string;
}

interface Workspace {
  id: string;
  name: string;
  projects: Project[];
  createdAt: string;
}

const props = defineProps<{
  open: boolean;
  currentProjects: Project[];
}>();

const emit = defineEmits<{
  close: [];
  loadWorkspace: [projects: Project[]];
}>();

// 本地项目列表副本，用于编辑
const localProjects = ref<Project[]>([]);

// 监听 props 变化，更新本地副本
watch(() => props.currentProjects, (newProjects) => {
  localProjects.value = [...newProjects];
}, { immediate: true });

const workspaceName = ref('');
const workspaces = ref<Workspace[]>([]);
const selectedWorkspaceId = ref<string>('');
const showPermissionDialog = ref(false);

const WORKSPACE_FILE_NAME = 'workspaces.json';

// 获取工作区文件路径
const getWorkspaceFilePath = async () => {
  const { appDataDir } = await import('@tauri-apps/api/path');
  const { join } = await import('@tauri-apps/api/path');
  const appData = await appDataDir();
  return await join(appData, WORKSPACE_FILE_NAME);
};

// 加载已保存的工作区
const loadWorkspaces = async () => {
  try {
    const { readTextFile } = await import('@tauri-apps/plugin-fs');
    const filePath = await getWorkspaceFilePath();
    const content = await readTextFile(filePath);
    workspaces.value = JSON.parse(content);
  } catch (e) {
    console.log('No saved workspaces found or failed to load:', e);
    workspaces.value = [];
  }
};

// 保存工作区列表
const saveWorkspaces = async () => {
  try {
    const { writeTextFile } = await import('@tauri-apps/plugin-fs');
    const { mkdir } = await import('@tauri-apps/plugin-fs');
    const { appDataDir } = await import('@tauri-apps/api/path');
    const { BaseDirectory } = await import('@tauri-apps/plugin-fs');

    // 确保目录存在
    await mkdir('', { baseDir: BaseDirectory.AppData, recursive: true });

    const filePath = await getWorkspaceFilePath();
    await writeTextFile(filePath, JSON.stringify(workspaces.value, null, 2));
  } catch (e) {
    console.error('Failed to save workspaces:', e);
    // 降级到 localStorage
    localStorage.setItem('giter-workspaces-backup', JSON.stringify(workspaces.value));
  }
};

// 保存当前工作区
const saveWorkspace = async () => {
  const name = workspaceName.value.trim();
  if (!name) return;

  const workspace: Workspace = {
    id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
    name,
    projects: [...localProjects.value],
    createdAt: new Date().toISOString()
  };

  workspaces.value.unshift(workspace);
  await saveWorkspaces();
  workspaceName.value = '';
};

// 导入当前项目列表
const saveCurrentWorkspace = async () => {
  // 重置本地项目列表为当前项目列表
  localProjects.value = [...props.currentProjects];
  
  const now = new Date();
  const dateStr = now.toLocaleDateString() + ' ' + now.toLocaleTimeString();
  const defaultName = `工作区 ${dateStr}`;
  
  // 使用简单的输入方式：直接填充到输入框
  workspaceName.value = defaultName;
  // 聚焦输入框
  setTimeout(() => {
    const input = document.querySelector('.workspace-input') as HTMLInputElement;
    if (input) {
      input.focus();
      input.select();
    }
  }, 100);
};

// 加载工作区
const loadWorkspace = (workspace: Workspace) => {
  emit('loadWorkspace', workspace.projects);
  emit('close');
};

// 删除工作区
const deleteWorkspace = async (id: string) => {
  const workspace = workspaces.value.find(w => w.id === id);
  if (!workspace) return;

  // 直接删除，不使用 confirm
  const index = workspaces.value.findIndex(w => w.id === id);
  if (index > -1) {
    workspaces.value.splice(index, 1);
    await saveWorkspaces();
    if (selectedWorkspaceId.value === id) {
      selectedWorkspaceId.value = '';
    }
  }
};

// 选择工作区
const selectWorkspace = (id: string) => {
  selectedWorkspaceId.value = id;
};

// 从文件夹导入项目
const importFromFolder = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog');
    const selected = await open({
      directory: true,
      multiple: true,
      title: '选择 Git 项目文件夹'
    });

    if (selected && Array.isArray(selected)) {
      const newProjects: Project[] = selected.map(path => {
        const parts = path.split(/[\\/]/);
        return {
          id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          name: parts[parts.length - 1] || parts[parts.length - 2] || '新项目',
          path
        };
      });

      // 添加到本地项目列表
      localProjects.value.push(...newProjects);
    }
  } catch (e) {
    console.error('Failed to import from folder:', e);
    alert('导入失败: ' + e);
  }
};

// 格式化日期
const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr);
  return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
};

// 从当前项目列表中移除项目（仅本地副本）
const removeCurrentProject = (projectId: string) => {
  const index = localProjects.value.findIndex(p => p.id === projectId);
  if (index > -1) {
    localProjects.value.splice(index, 1);
  }
};

// 打开系统设置
const openSystemSettings = async () => {
  try {
    const { open } = await import('@tauri-apps/plugin-shell');
    await open('x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles');
  } catch (e) {
    console.error('Failed to open system settings:', e);
  }
};

onMounted(async () => {
  await loadWorkspaces();
});
</script>

<style scoped>
.workspace-manager {
  display: flex;
  flex-direction: column;
  gap: 20px;
  min-width: 500px;
  max-height: 600px;
  overflow-y: auto;
}

.section {
  border: 1px solid var(--border-color);
  border-radius: 8px;
  padding: 16px;
}

.section h4 {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: var(--text-primary);
  font-weight: 600;
}

.save-section .input-group {
  display: flex;
  gap: 8px;
  margin-bottom: 12px;
}

.workspace-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}

.workspace-input:focus {
  outline: none;
  border-color: var(--accent-color);
}

.btn-save-current {
  width: 100%;
}

.current-projects {
  margin-bottom: 16px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-primary);
}

.projects-header {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-primary);
  padding: 12px 16px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  border-radius: 8px 8px 0 0;
}

.projects-list {
  max-height: 200px;
  overflow-y: auto;
  padding: 8px;
}

.project-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  border-radius: 6px;
  transition: background-color 0.2s;
}

.project-item:hover {
  background-color: var(--bg-hover);
}

.project-item:not(:last-child) {
  border-bottom: 1px solid var(--border-color);
}

.project-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
  flex-shrink: 0;
}

.project-path {
  font-size: 12px;
  color: var(--text-secondary);
  word-break: break-all;
  flex: 1;
}

.btn-delete-project {
  width: 24px;
  height: 24px;
  padding: 4px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  opacity: 0;
  transition: all 0.2s;
}

.project-item:hover .btn-delete-project {
  opacity: 1;
}

.btn-delete-project:hover {
  background-color: var(--error-color-light, rgba(239, 68, 68, 0.1));
  color: var(--error-color, #ef4444);
}

.no-projects {
  text-align: center;
  padding: 24px;
  color: var(--text-secondary);
  font-size: 13px;
  margin-bottom: 16px;
  border: 1px dashed var(--border-color);
  border-radius: 8px;
}

.workspace-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 250px;
  overflow-y: auto;
}

.workspace-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  min-height: 60px;
}

.workspace-item:hover {
  background-color: var(--bg-hover);
}

.workspace-item.active {
  border-color: var(--accent-color);
  background-color: var(--accent-color-light, rgba(59, 130, 246, 0.1));
}

.workspace-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  flex: 1;
  min-width: 0;
}

.workspace-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-primary);
}

.workspace-date,
.workspace-count {
  font-size: 11px;
  color: var(--text-secondary);
}

.workspace-actions {
  display: flex;
  gap: 4px;
  align-items: center;
  flex-shrink: 0;
}

.empty-state {
  text-align: center;
  padding: 24px;
  color: var(--text-secondary);
  font-size: 13px;
}

.btn {
  padding: 8px 16px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
}

.btn:hover {
  background-color: var(--bg-hover);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background-color: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.btn-primary:hover {
  background-color: var(--accent-color-hover, #2563eb);
}

.btn-secondary {
  background-color: var(--bg-secondary);
  border-color: var(--border-color);
}

.btn-secondary:hover {
  background-color: var(--bg-hover);
}

.btn-icon {
  padding: 6px;
  width: 32px;
  height: 32px;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
}

.btn-delete {
  color: var(--error-color, #ef4444);
  border-color: var(--border-color);
  background-color: var(--bg-primary);
}

.btn-delete:hover {
  background-color: var(--error-color-light, rgba(239, 68, 68, 0.1));
  border-color: var(--error-color, #ef4444);
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
  z-index: 1000;
}

.permission-dialog {
  background-color: var(--bg-primary);
  border-radius: 12px;
  padding: 24px;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.permission-dialog h3 {
  margin: 0 0 16px 0;
  font-size: 18px;
  color: var(--text-primary);
}

.permission-dialog p {
  margin: 8px 0;
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.permission-dialog ol {
  margin: 16px 0;
  padding-left: 20px;
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.8;
}

.permission-dialog li {
  margin: 8px 0;
}

.permission-actions {
  display: flex;
  gap: 12px;
  margin-top: 20px;
  justify-content: flex-end;
}
</style>
