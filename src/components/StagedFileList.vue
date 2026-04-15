<template>
  <div class="staged-file-list">
    <div
      v-for="file in stagedFiles"
      :key="file.path"
      class="staged-file-item"
      :class="[file.status?.toLowerCase(), { active: selectedPath === file.path }]"
      @click="$emit('select', file.path)"
    >
      <span class="file-icon">📄</span>
      <span class="file-name" :title="file.path">{{ file.name }}</span>
      <span class="file-status" :class="file.status?.toLowerCase()">
        {{ getStatusIcon(file.status) }}
      </span>
    </div>
    <div v-if="stagedFiles.length === 0" class="empty-state">
      没有更改的文件
    </div>
  </div>
</template>

<script setup lang="ts">
interface StagedFile {
  name: string;
  path: string;
  status?: string;
}

defineProps<{
  stagedFiles: StagedFile[];
  selectedPath?: string;
}>();

defineEmits<{
  select: [path: string];
}>();

const getStatusIcon = (status?: string): string => {
  switch (status) {
    case 'Modified': return 'M';
    case 'Added': return 'A';
    case 'Deleted': return 'D';
    case 'Renamed': return 'R';
    case 'Untracked': return '?';
    default: return '';
  }
};
</script>

<style scoped>
.staged-file-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.staged-file-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.staged-file-item:hover {
  background-color: var(--bg-hover);
}

.staged-file-item.active {
  background-color: var(--accent-color);
  color: white;
}

.staged-file-item.active .file-name {
  color: white;
}

.staged-file-item.active .file-status {
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
}

.file-icon {
  font-size: 14px;
  flex-shrink: 0;
}

.file-name {
  flex: 1;
  font-size: 13px;
  color: var(--text-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.file-status {
  width: 18px;
  height: 18px;
  border-radius: 3px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  font-weight: bold;
  flex-shrink: 0;
}

.file-status.modified {
  background-color: var(--changed-bg);
  color: var(--changed-text);
}

.file-status.added {
  background-color: var(--added-bg);
  color: var(--added-text);
}

.file-status.deleted {
  background-color: var(--removed-bg);
  color: var(--removed-text);
}

.file-status.untracked {
  background-color: var(--bg-tertiary);
  color: var(--text-secondary);
}

.empty-state {
  padding: 24px 16px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
}
</style>
