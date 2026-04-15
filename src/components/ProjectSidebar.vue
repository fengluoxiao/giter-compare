<template>
  <aside
    class="project-sidebar"
    :class="{ collapsed: isCollapsed }"
    :style="{ width: isCollapsed ? '40px' : width + 'px' }"
  >
    <div class="project-header">
      <h3 v-show="!isCollapsed">项目列表</h3>
      <div class="project-header-buttons">
        <button class="btn btn-icon btn-add" @click="$emit('add-project')" title="添加项目">
          <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
            <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z"/>
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
        :title="project.path"
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
  </aside>
</template>

<script setup lang="ts">
interface Project {
  id: string;
  name: string;
  path: string;
}

defineProps<{
  projects: Project[];
  currentProjectId: string;
  isCollapsed: boolean;
  width: number;
}>();

defineEmits<{
  'add-project': [];
  'toggle-collapse': [];
  'switch-project': [project: Project];
  'remove-project': [projectId: string];
  'start-resize': [event: MouseEvent];
}>();

// 从路径获取文件夹名称
const getFolderName = (path: string): string => {
  const parts = path.split('/');
  return parts[parts.length - 1] || parts[parts.length - 2] || '新项目';
};
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
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
}

.project-header h3 {
  font-size: 14px;
  color: var(--text-primary);
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
</style>
