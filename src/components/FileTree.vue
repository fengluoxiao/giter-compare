<template>
  <div class="file-tree" style="min-width: fit-content;">
    <template v-for="node in nodes" :key="node.path">
      <div
        v-if="node.type === 'directory'"
        class="tree-item directory"
        :class="{ 
          expanded: node.expanded, 
          'has-changes': hasChangesInDirectory(node)
        }"
        :style="{ paddingLeft: `${(level || 0) * 20}px` }"
        @click="$emit('toggle', node)"
        @contextmenu.prevent="showContextMenu($event, node)"
      >
        <span class="tree-arrow">
          <svg v-if="node.expanded" viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
            <path d="M7 10l5 5 5-5z"/>
          </svg>
          <svg v-else viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
            <path d="M10 17l5-5-5-5v10z"/>
          </svg>
        </span>
        <span class="tree-icon">{{ node.expanded ? '📂' : '📁' }}</span>
        <span class="tree-name">{{ node.name }}</span>
        <span class="tree-status-placeholder-right"></span>
      </div>
      <div
        v-else
        class="tree-item file"
        :class="[node.status?.toLowerCase(), node.status ? 'changed' : 'unchanged']"
        :style="{ paddingLeft: `${(level || 0) * 20 + 20}px` }"
        @click="$emit('select', node.path)"
        @contextmenu.prevent="showContextMenu($event, node)"
      >
        <span class="tree-line" v-if="(level || 0) > 0">└─</span>
        <span class="tree-icon">📄</span>
        <span class="tree-name">{{ node.name }}</span>
        <span v-if="node.status" class="tree-status-right" :class="node.status.toLowerCase()">
          {{ getStatusIcon(node.status) }}
        </span>
        <span v-else class="tree-status-placeholder-right"></span>
      </div>
      <FileTree
        v-if="node.type === 'directory' && node.expanded && node.children"
        :nodes="node.children"
        :level="(level || 0) + 1"
        @select="$emit('select', $event)"
        @toggle="$emit('toggle', $event)"
        @menu-action="(action, node) => $emit('menu-action', action, node)"
      />
    </template>

    <!-- 右击菜单 -->
    <div
      v-if="contextMenuVisible"
      class="tree-context-menu"
      :style="{ left: `${contextMenuX}px`, top: `${contextMenuY}px` }"
      @click.stop
    >
      <div class="context-menu-item" @click.stop="handleMenuAction('copyPath')">
        📋 复制路径
      </div>
      <div class="context-menu-item" @click.stop="handleMenuAction('copyRelativePath')">
        📄 复制相对路径
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item" @click.stop="handleMenuAction('openInFinder')">
        📂 在 Finder 中打开
      </div>
      <div class="context-menu-item" @click.stop="handleMenuAction('openInEditor')">
        ✏️ 在编辑器中打开
      </div>
      <template v-if="contextMenuNode?.type === 'file'">
        <div class="context-menu-divider"></div>
        <div class="context-menu-item" @click.stop="handleMenuAction('viewHistory')">
          📜 查看历史变更
        </div>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue';

interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  status?: string;
  children: FileNode[];
  expanded?: boolean;
}

const props = defineProps<{
  nodes: FileNode[];
  level?: number;
}>();

const emit = defineEmits<{
  select: [path: string];
  toggle: [node: FileNode];
  'menu-action': [action: string, node: FileNode];
}>();

// 右击菜单状态
const contextMenuVisible = ref(false);
const contextMenuX = ref(0);
const contextMenuY = ref(0);
const contextMenuNode = ref<FileNode | null>(null);

// 显示右击菜单
const showContextMenu = (event: MouseEvent, node: FileNode) => {
  contextMenuNode.value = node;
  contextMenuX.value = event.clientX;
  contextMenuY.value = event.clientY;
  contextMenuVisible.value = true;
};

// 隐藏右击菜单
const hideContextMenu = () => {
  contextMenuVisible.value = false;
};

// 处理菜单项点击
const handleMenuAction = (action: string) => {
  console.log('handleMenuAction:', action, contextMenuNode.value?.path);
  if (contextMenuNode.value) {
    emit('menu-action', action, contextMenuNode.value);
  }
  hideContextMenu();
};

// 点击其他地方隐藏菜单
const handleDocumentClick = (event: MouseEvent) => {
  const target = event.target as HTMLElement;
  if (!target.closest('.tree-context-menu')) {
    hideContextMenu();
  }
};

onMounted(() => {
  document.addEventListener('click', handleDocumentClick);
});

onUnmounted(() => {
  document.removeEventListener('click', handleDocumentClick);
});

// 递归检查文件夹下是否有任何更改的文件
const hasChangesInDirectory = (node: FileNode): boolean => {
  if (node.type === 'file') {
    return !!node.status;
  }
  if (node.children && node.children.length > 0) {
    return node.children.some(child => hasChangesInDirectory(child));
  }
  return false;
};

// 获取文件夹下所有更改的状态列表
const getDirectoryChangeStatuses = (node: FileNode): string[] => {
  const statuses: string[] = [];
  
  const collectStatuses = (n: FileNode) => {
    if (n.type === 'file' && n.status) {
      statuses.push(n.status);
    }
    if (n.children) {
      n.children.forEach(child => collectStatuses(child));
    }
  };
  
  collectStatuses(node);
  return statuses;
};

// 获取文件夹的汇总状态（优先级：Modified > Added > Deleted > Renamed > Untracked）
const getDirectorySummaryStatus = (node: FileNode): string | null => {
  const statuses = getDirectoryChangeStatuses(node);
  if (statuses.length === 0) return null;
  
  const priority = ['Modified', 'Added', 'Deleted', 'Renamed', 'Untracked'];
  for (const p of priority) {
    if (statuses.includes(p)) return p;
  }
  return statuses[0];
};

const getStatusIcon = (status: string): string => {
  switch (status) {
    case 'Modified': return 'M';
    case 'Added': return 'A';
    case 'Deleted': return 'D';
    case 'Renamed': return 'R';
    case 'Untracked': return '?';
    default: return ' ';
  }
};
</script>

<style scoped>
/* 右击菜单 */
.tree-context-menu {
  position: fixed;
  z-index: 9999;
  background-color: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  padding: 4px 0;
  min-width: 180px;
  font-size: 13px;
}

.context-menu-item {
  padding: 8px 16px;
  cursor: pointer;
  color: var(--text-primary);
  transition: background-color 0.15s;
  display: flex;
  align-items: center;
  gap: 8px;
}

.context-menu-item:hover {
  background-color: var(--bg-hover);
}

.context-menu-item.danger {
  color: #e51400;
}

.context-menu-item.danger:hover {
  background-color: rgba(229, 20, 0, 0.1);
}

.context-menu-divider {
  height: 1px;
  background-color: var(--border-color);
  margin: 4px 0;
}
</style>
