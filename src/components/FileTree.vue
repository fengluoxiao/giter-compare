<template>
  <div class="file-tree">
    <template v-for="node in nodes" :key="node.path">
      <div
        v-if="node.type === 'directory'"
        class="tree-item directory"
        :class="{ expanded: node.expanded, 'has-changes': node.status }"
        :style="{ paddingLeft: `${(level || 0) * 20}px` }"
        @click="$emit('toggle', node)"
      >
        <span class="tree-arrow">{{ node.expanded ? '▼' : '▶' }}</span>
        <span class="tree-icon">{{ node.expanded ? '📂' : '📁' }}</span>
        <span class="tree-name">{{ node.name }}</span>
        <span v-if="node.status" class="tree-status-right" :class="node.status.toLowerCase()">
          {{ getStatusIcon(node.status) }}
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
        <span class="tree-line" v-if="level > 0">└─</span>
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
interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  status?: string;
  children: FileNode[];
  expanded?: boolean;
}

defineProps<{
  nodes: FileNode[];
  level?: number;
}>();

defineEmits<{
  select: [path: string];
  toggle: [node: FileNode];
}>();

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
