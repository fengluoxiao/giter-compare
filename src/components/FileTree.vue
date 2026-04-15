<template>
  <div class="file-tree" style="min-width: fit-content;">
    <template v-for="node in nodes" :key="node.path">
      <div
        v-if="node.type === 'directory'"
        class="tree-item directory"
        :class="{ 
          expanded: node.expanded, 
          'has-changes': hasChangesInDirectory(node),
          [getDirectorySummaryStatus(node)?.toLowerCase() || '']: getDirectorySummaryStatus(node)
        }"
        :style="{ paddingLeft: `${(level || 0) * 20}px` }"
        @click="$emit('toggle', node)"
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
        <span 
          v-if="hasChangesInDirectory(node)" 
          class="tree-status-right" 
          :class="getDirectorySummaryStatus(node)?.toLowerCase()"
        >
          {{ getStatusIcon(getDirectorySummaryStatus(node)!) }}
        </span>
        <span v-else class="tree-status-placeholder-right"></span>
      </div>
      <div
        v-else
        class="tree-item file"
        :class="[node.status?.toLowerCase(), node.status ? 'changed' : 'unchanged']"
        :style="{ paddingLeft: `${(level || 0) * 20 + 20}px` }"
        @click="$emit('select', node.path)"
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
      />
    </template>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

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

defineEmits<{
  select: [path: string];
  toggle: [node: FileNode];
}>();

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
