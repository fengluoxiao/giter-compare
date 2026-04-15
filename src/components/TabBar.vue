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
        <span class="tab-icon" v-html="getFileIcon(tab.fileType)"></span>
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
    // Vue
    'vue': '<svg viewBox="0 0 32 32" width="14" height="14"><path fill="#41B883" d="M2 4h4l8 14 8-14h4L16 24z"/><path fill="#34495E" d="M6 4h4l6 10 6-10h4l-10 17z"/></svg>',
    // TypeScript
    'ts': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#3178C6" width="32" height="32" rx="4"/><path fill="#fff" d="M18 16v-2h8v2h-3v10h-2V16h-3zM13 26c-1.5 0-2.8-.3-3.8-.9l.7-1.6c.9.5 2 .8 3.1.8 1.2 0 1.8-.4 1.8-1.1 0-.7-.5-1-2-1.5-2-.6-3.3-1.3-3.3-3.2 0-1.9 1.5-3 3.8-3 1.3 0 2.4.3 3.2.7l-.6 1.6c-.8-.4-1.7-.6-2.6-.6-1.1 0-1.6.5-1.6 1 0 .7.5 1 2 1.5 2.1.7 3.3 1.5 3.3 3.2 0 2-1.5 3.2-4 3.2z"/></svg>',
    // JavaScript
    'js': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#F7DF1E" width="32" height="32" rx="4"/><path fill="#000" d="M22.5 24.5c-1.3 0-2.1-.6-2.5-1.3l-2.2 1.3c.8 1.5 2.5 2.7 4.8 2.7 2.9 0 4.8-1.4 4.8-4.1 0-2.8-2-3.6-4.3-4.6-1.4-.6-1.9-.9-1.9-1.6 0-.6.5-1.1 1.4-1.1 1 0 1.6.4 2.1 1.3l2-1.3c-.7-1.5-2-2.3-4.1-2.3-2.6 0-4.3 1.5-4.3 3.9 0 2.5 1.6 3.4 4 4.4 1.3.5 2.2.8 2.2 1.7 0 .7-.6 1.2-1.8 1.2zM12.3 24.5c-1.6 0-2.6-.8-3.1-1.7l-2.1 1.3c.9 1.8 2.8 3 5.3 3 3.2 0 5.3-1.6 5.3-4.8V10.5h-2.6v11.8c0 1.6-.8 2.2-1.8 2.2z"/></svg>',
    // JSON
    'json': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#4A4A4A" width="32" height="32" rx="4"/><text x="4" y="22" fill="#fff" font-size="12" font-weight="bold">{ }</text></svg>',
    // CSS
    'css': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#264DE4" width="32" height="32" rx="4"/><path fill="#fff" d="M6 8h20l-1.5 17-8.5 3-8.5-3L6 8zm4.2 4l.4 4h8.8l-.3 3-3.1 1-3.1-1-.2-2h-3l.4 4.5 6 2 6-2 .8-9H10.2z"/></svg>',
    // SCSS/Sass
    'scss': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#CC6699" width="32" height="32" rx="4"/><path fill="#fff" d="M6 8h20l-1.5 17-8.5 3-8.5-3L6 8zm4.2 4l.4 4h8.8l-.3 3-3.1 1-3.1-1-.2-2h-3l.4 4.5 6 2 6-2 .8-9H10.2z"/></svg>',
    // HTML
    'html': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#E34F26" width="32" height="32" rx="4"/><path fill="#fff" d="M6 8l2 18 8 3 8-3 2-18H6zm12.5 15l-5.5 1.5-4-1.5-.5-6h3l.2 3 3 .8 3-.8.3-3h-7l-.5-6h10l.2-3h-13.5l.5 6h7l-.2 3h-7l-.5 6h10.5l.3-3z"/></svg>',
    // Markdown
    'md': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#083FA1" width="32" height="32" rx="4"/><path fill="#fff" d="M6 8h20v16H6V8zm2 2v12h16V10H8zm2 2h3v4l2-2 2 2v-4h3v8h-3v-2l-2 2-2-2v2h-3V12z"/></svg>',
    // Rust
    'rs': '<svg viewBox="0 0 32 32" width="14" height="14"><circle fill="#000" cx="16" cy="16" r="16"/><path fill="#fff" d="M16 6c-5.5 0-10 4.5-10 10s4.5 10 10 10 10-4.5 10-10S21.5 6 16 6zm0 18c-4.4 0-8-3.6-8-8s3.6-8 8-8 8 3.6 8 8-3.6 8-8 8z"/><path fill="#fff" d="M16 10c-3.3 0-6 2.7-6 6s2.7 6 6 6 6-2.7 6-6-2.7-6-6-6zm0 10c-2.2 0-4-1.8-4-4s1.8-4 4-4 4 1.8 4 4-1.8 4-4 4z"/></svg>',
    // Python
    'py': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#3776AB" width="32" height="32" rx="4"/><path fill="#FFD43B" d="M15.5 6c-2.5 0-4.5.5-4.5 2v2.5h5v1h-7c-1.5 0-2.5 1-2.5 3.5s1 3.5 2.5 3.5h2v-2c0-2 1.5-3.5 3.5-3.5h5c1.5 0 2.5-1 2.5-2.5V8c0-1.5-2-2-4.5-2zM12.5 8c.8 0 1.5.7 1.5 1.5S13.3 11 12.5 11 11 10.3 11 9.5s.7-1.5 1.5-1.5z"/><path fill="#fff" d="M16.5 26c2.5 0 4.5-.5 4.5-2v-2.5h-5v-1h7c1.5 0 2.5-1 2.5-3.5s-1-3.5-2.5-3.5h-2v2c0 2-1.5 3.5-3.5 3.5h-5c-1.5 0-2.5 1-2.5 2.5V24c0 1.5 2 2 4.5 2zm3-1c-.8 0-1.5-.7-1.5-1.5s.7-1.5 1.5-1.5 1.5.7 1.5 1.5-.7 1.5-1.5 1.5z"/></svg>',
    // Java
    'java': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#007396" width="32" height="32" rx="4"/><path fill="#fff" d="M12 24c0 0-1.5 1 1 1.5 2.5.5 4 .5 6 0 2-.5 1-1.5 1-1.5s-1.5.5-4 .5-4-.5-4-.5zM11 21c0 0-1.5 1 1 1.5 2.5.5 4 .5 6 0 2-.5 1-1.5 1-1.5s-1.5.5-4 .5-4-.5-4-.5z"/><path fill="#fff" d="M16 8c-2 0-5 1-5 4 0 2 1.5 3 3 3.5 1.5.5 2.5 1 2.5 2s-1 1.5-2.5 1.5c-2 0-3.5-1-3.5-1s-1 2 1 2.5c2 .5 4 .5 5.5 0 2-.5 3-2 3-3.5 0-2-1.5-3-3-3.5-1.5-.5-2.5-1-2.5-2s1-1.5 2.5-1.5c1.5 0 3 1 3 1s.5-2-1-2.5c-1-.5-3-.5-5.5 0z"/></svg>',
    // Go
    'go': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#00ADD8" width="32" height="32" rx="4"/><path fill="#fff" d="M8 16c0-3 2-5 5-5s5 2 5 5-2 5-5 5-5-2-5-5zm12 0c0-3 2-5 5-5s5 2 5 5-2 5-5 5-5-2-5-5z"/></svg>',
    // React
    'tsx': '<svg viewBox="0 0 32 32" width="14" height="14"><circle fill="#61DAFB" cx="16" cy="16" r="16"/><circle fill="#fff" cx="16" cy="16" r="3"/><path fill="none" stroke="#fff" stroke-width="1.5" d="M16 8c4 0 7.5 3.5 7.5 8s-3.5 8-7.5 8-7.5-3.5-7.5-8 3.5-8 7.5-8z"/><path fill="none" stroke="#fff" stroke-width="1.5" d="M16 8c2 3.5 2 12.5 0 16"/><path fill="none" stroke="#fff" stroke-width="1.5" d="M16 8c-2 3.5-2 12.5 0 16"/></svg>',
    'jsx': '<svg viewBox="0 0 32 32" width="14" height="14"><circle fill="#61DAFB" cx="16" cy="16" r="16"/><circle fill="#fff" cx="16" cy="16" r="3"/><path fill="none" stroke="#fff" stroke-width="1.5" d="M16 8c4 0 7.5 3.5 7.5 8s-3.5 8-7.5 8-7.5-3.5-7.5-8 3.5-8 7.5-8z"/><path fill="none" stroke="#fff" stroke-width="1.5" d="M16 8c2 3.5 2 12.5 0 16"/><path fill="none" stroke="#fff" stroke-width="1.5" d="M16 8c-2 3.5-2 12.5 0 16"/></svg>',
    // C++
    'cpp': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#00599C" width="32" height="32" rx="4"/><text x="4" y="22" fill="#fff" font-size="14" font-weight="bold">C++</text></svg>',
    // C
    'c': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#A8B9CC" width="32" height="32" rx="4"/><text x="8" y="24" fill="#fff" font-size="18" font-weight="bold">C</text></svg>',
    // C#
    'cs': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#9B4F96" width="32" height="32" rx="4"/><text x="6" y="22" fill="#fff" font-size="14" font-weight="bold">C#</text></svg>',
    // PHP
    'php': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#777BB4" width="32" height="32" rx="4"/><text x="4" y="22" fill="#fff" font-size="12" font-weight="bold">php</text></svg>',
    // Ruby
    'rb': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#CC342D" width="32" height="32" rx="4"/><path fill="#fff" d="M16 6l-6 4-2 8 8 6 8-6-2-8-6-4z"/></svg>',
    // Shell/Bash
    'sh': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#4EAA25" width="32" height="32" rx="4"/><text x="6" y="22" fill="#fff" font-size="14" font-weight="bold">$_</text></svg>',
    'bash': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#4EAA25" width="32" height="32" rx="4"/><text x="6" y="22" fill="#fff" font-size="14" font-weight="bold">$_</text></svg>',
    // YAML
    'yml': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#CB171E" width="32" height="32" rx="4"/><text x="4" y="22" fill="#fff" font-size="12" font-weight="bold">YML</text></svg>',
    'yaml': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#CB171E" width="32" height="32" rx="4"/><text x="2" y="22" fill="#fff" font-size="11" font-weight="bold">YAML</text></svg>',
    // XML
    'xml': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#FF6600" width="32" height="32" rx="4"/><text x="6" y="22" fill="#fff" font-size="14" font-weight="bold">&lt;/&gt;</text></svg>',
    // SQL
    'sql': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#336791" width="32" height="32" rx="4"/><text x="4" y="22" fill="#fff" font-size="12" font-weight="bold">SQL</text></svg>',
    // Docker
    'dockerfile': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#2496ED" width="32" height="32" rx="4"/><path fill="#fff" d="M8 12h2v2H8v-2zm3 0h2v2h-2v-2zm3 0h2v2h-2v-2zm-6 3h2v2H8v-2zm3 0h2v2h-2v-2zm3 0h2v2h-2v-2zm3 0h2v2h-2v-2zM8 18h2v2H8v-2zm12 0h2v2h-2v-2zm-9 0h2v2h-2v-2zm3 0h2v2h-2v-2z"/></svg>',
    // Git
    'gitignore': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#F05032" width="32" height="32" rx="4"/><circle fill="#fff" cx="16" cy="16" r="4"/><path fill="none" stroke="#fff" stroke-width="2" d="M16 8v4M16 20v4M12 12l3 3 3-3"/></svg>',
    // Lock file
    'lock': '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#F9A825" width="32" height="32" rx="4"/><path fill="#fff" d="M16 6c-3 0-5 2-5 5v3H9v10h14V14h-2v-3c0-3-2-5-5-5zm0 2c1.5 0 3 1 3 3v3H13v-3c0-2 1.5-3 3-3z"/></svg>',
  };
  return icons[fileType] || '<svg viewBox="0 0 32 32" width="14" height="14"><rect fill="#6B7280" width="32" height="32" rx="4"/><text x="10" y="22" fill="#fff" font-size="14" font-weight="bold">📄</text></svg>';
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
