<template>
  <div class="tab-bar">
    <div class="tab-list">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="tab-item"
        :class="{ active: activeTabId === tab.id, modified: tab.isModified }"
        @click="activateTab(tab.id)"
        @contextmenu.prevent="showContextMenu($event, tab.id)"
      >
        <span class="tab-icon">{{ getFileIcon(tab.fileType) }}</span>
        <span class="tab-name">{{ tab.name }}</span>
        <span v-if="tab.isModified" class="tab-modified-indicator">●</span>
        <button
          class="tab-close"
          @click.stop="closeTab(tab.id)"
          title="关闭标签页"
        >
          ×
        </button>
      </div>
    </div>
    <div class="tab-actions">
      <button class="tab-action-btn" @click="$emit('close-all')" title="关闭所有">
        ✕
      </button>
    </div>
    <!-- 右键菜单 -->
    <div
      v-if="contextMenu.show"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
    >
      <div class="context-menu-item" @click="closeContextMenuTab">关闭</div>
      <div class="context-menu-item" @click="closeOtherTabs">关闭其他</div>
      <div class="context-menu-item" @click="closeTabsToRight">关闭右侧标签</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';

export interface Tab {
  id: string;
  name: string;
  path: string;
  projectPath: string;
  fileType: string;
  isModified: boolean;
  leftLines: any[];
  rightLines: any[];
  isBinary: boolean;
  diffStats: any;
  scrollTop: number;
}

const props = defineProps<{
  tabs: Tab[];
  activeTabId: string;
}>();

const emit = defineEmits<{
  activate: [tabId: string];
  close: [tabId: string];
  'close-all': [];
  'close-others': [tabId: string];
  'close-to-right': [tabId: string];
}>();

const contextMenu = reactive({
  show: false,
  x: 0,
  y: 0,
  tabId: '',
});

const getFileIcon = (fileType: string) => {
  const icons: Record<string, string> = {
    'vue': '💚',
    'ts': '💙',
    'js': '💛',
    'json': '📋',
    'css': '🎨',
    'scss': '🎨',
    'html': '🌐',
    'md': '📝',
    'rs': '🦀',
    'py': '🐍',
    'java': '☕',
    'go': '🔵',
  };
  return icons[fileType] || '📄';
};

const activateTab = (tabId: string) => {
  emit('activate', tabId);
};

const closeTab = (tabId: string) => {
  emit('close', tabId);
};

const showContextMenu = (e: MouseEvent, tabId: string) => {
  contextMenu.show = true;
  contextMenu.x = e.clientX;
  contextMenu.y = e.clientY;
  contextMenu.tabId = tabId;
};

const hideContextMenu = () => {
  contextMenu.show = false;
};

const closeContextMenuTab = () => {
  emit('close', contextMenu.tabId);
  hideContextMenu();
};

const closeOtherTabs = () => {
  emit('close-others', contextMenu.tabId);
  hideContextMenu();
};

const closeTabsToRight = () => {
  emit('close-to-right', contextMenu.tabId);
  hideContextMenu();
};

// 点击其他地方关闭右键菜单
window.addEventListener('click', hideContextMenu);
</script>

<style scoped>
.tab-bar {
  display: flex;
  align-items: center;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  height: 36px;
  overflow: hidden;
}

.tab-list {
  display: flex;
  flex: 1;
  overflow-x: auto;
  scrollbar-width: none;
  -ms-overflow-style: none;
}

.tab-list::-webkit-scrollbar {
  display: none;
}

.tab-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 0 12px;
  height: 36px;
  min-width: 120px;
  max-width: 200px;
  background-color: var(--bg-tertiary);
  border-right: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.2s;
  user-select: none;
  position: relative;
}

.tab-item:hover {
  background-color: var(--bg-hover);
}

.tab-item.active {
  background-color: var(--bg-primary);
  border-bottom: 2px solid var(--accent-color);
}

.tab-item.modified .tab-name {
  font-style: italic;
}

.tab-icon {
  font-size: 12px;
  flex-shrink: 0;
}

.tab-name {
  flex: 1;
  font-size: 12px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tab-modified-indicator {
  color: var(--accent-color);
  font-size: 8px;
  flex-shrink: 0;
}

.tab-close {
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  border-radius: 3px;
  flex-shrink: 0;
  opacity: 0;
  transition: opacity 0.2s;
}

.tab-item:hover .tab-close,
.tab-item.active .tab-close {
  opacity: 1;
}

.tab-close:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.tab-actions {
  display: flex;
  align-items: center;
  padding: 0 8px;
  border-left: 1px solid var(--border-color);
}

.tab-action-btn {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 12px;
  cursor: pointer;
  border-radius: 3px;
}

.tab-action-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

/* 右键菜单 */
.context-menu {
  position: fixed;
  background-color: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 1000;
  min-width: 140px;
}

.context-menu-item {
  padding: 8px 12px;
  font-size: 12px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background-color 0.2s;
}

.context-menu-item:hover {
  background-color: var(--bg-hover);
}

.context-menu-item:first-child {
  border-radius: 4px 4px 0 0;
}

.context-menu-item:last-child {
  border-radius: 0 0 4px 4px;
}
</style>
