<template>
  <DialogBase :open="open" title="工作区管理" @close="$emit('close')">
    <div class="workspace-manager">
      <!-- 保存当前工作区 -->
      <div class="section save-section">
        <h4>保存当前工作区</h4>
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
          一键保存当前项目
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
            <div class="workspace-actions">
              <button class="btn btn-icon" @click.stop="loadWorkspace(workspace)" title="加载">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/>
                </svg>
              </button>
              <button class="btn btn-icon btn-delete" @click.stop="deleteWorkspace(workspace.id)" title="删除">
                <svg viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
                  <path d="M6 19c0 1.1.9 2 2 2h8c1.1 0 2-.9 2-2V7H6v12zM19 4h-3.5l-1-1h-5l-1 1H5v2h14V4z"/>
                </svg>
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- 导入文件夹 -->
      <div class="section import-section">
        <h4>导入项目</h4>
        <button class="btn btn-secondary" @click="importFromFolder">
          从文件夹导入项目
        </button>
      </div>
    </div>
  </DialogBase>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
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

const workspaceName = ref('');
const workspaces = ref<Workspace[]>([]);
const selectedWorkspaceId = ref<string>('');

const WORKSPACE_STORAGE_KEY = 'giter-workspaces';

// 加载已保存的工作区
const loadWorkspaces = () => {
  const saved = localStorage.getItem(WORKSPACE_STORAGE_KEY);
  if (saved) {
    try {
      workspaces.value = JSON.parse(saved);
    } catch (e) {
      console.error('Failed to load workspaces:', e);
      workspaces.value = [];
    }
  }
};

// 保存工作区列表
const saveWorkspaces = () => {
  localStorage.setItem(WORKSPACE_STORAGE_KEY, JSON.stringify(workspaces.value));
};

// 保存当前工作区
const saveWorkspace = () => {
  const name = workspaceName.value.trim();
  if (!name) return;

  const workspace: Workspace = {
    id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
    name,
    projects: [...props.currentProjects],
    createdAt: new Date().toISOString()
  };

  workspaces.value.unshift(workspace);
  saveWorkspaces();
  workspaceName.value = '';
};

// 一键保存当前项目
const saveCurrentWorkspace = () => {
  const now = new Date();
  const dateStr = now.toLocaleDateString() + ' ' + now.toLocaleTimeString();
  workspaceName.value = `工作区 ${dateStr}`;
  saveWorkspace();
};

// 加载工作区
const loadWorkspace = (workspace: Workspace) => {
  emit('loadWorkspace', workspace.projects);
  emit('close');
};

// 删除工作区
const deleteWorkspace = (id: string) => {
  const workspace = workspaces.value.find(w => w.id === id);
  if (!workspace) return;
  
  // 确认删除
  if (confirm(`确定要删除工作区 "${workspace.name}" 吗?`)) {
    const index = workspaces.value.findIndex(w => w.id === id);
    if (index > -1) {
      workspaces.value.splice(index, 1);
      saveWorkspaces();
      if (selectedWorkspaceId.value === id) {
        selectedWorkspaceId.value = '';
      }
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

      // 弹出输入框让用户输入工作区名称
      const name = prompt(`已选择 ${newProjects.length} 个项目\n\n请输入工作区名称:`, `工作区 ${new Date().toLocaleDateString()}`);
      
      if (name && name.trim()) {
        // 创建新工作区
        const workspace: Workspace = {
          id: Date.now().toString() + Math.random().toString(36).substr(2, 9),
          name: name.trim(),
          projects: newProjects,
          createdAt: new Date().toISOString()
        };

        workspaces.value.unshift(workspace);
        saveWorkspaces();
      }
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

onMounted(() => {
  loadWorkspaces();
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
</style>
