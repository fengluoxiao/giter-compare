<template>
  <div class="diff-viewer">
    <div v-if="currentFile" class="file-info-bar">
      <div class="file-info">
        <span class="file-label">旧版本</span>
        <span class="file-path">{{ currentFile.path }}</span>
      </div>
      <div class="file-info">
        <span class="file-label">新版本</span>
        <span class="file-path">{{ currentFile.path }}</span>
      </div>
    </div>

    <div v-if="currentFile" class="diff-content">
      <div v-if="isBinary" class="binary-placeholder">
        [二进制文件]
      </div>
      <template v-else>
        <div class="diff-pane">
          <div class="pane-header">
            <span class="pane-title">HEAD</span>
          </div>
          <div class="code-content">
            <CodeViewer 
              :content="oldContent" 
              :filename="currentFile.path"
              :theme="theme"
              read-only
            />
          </div>
        </div>
        <div class="diff-divider"></div>
        <div class="diff-pane">
          <div class="pane-header">
            <span class="pane-title">{{ viewMode === 'working' ? '工作区' : '暂存区' }}</span>
          </div>
          <div class="code-content">
            <CodeViewer 
              :content="newContent" 
              :filename="currentFile.path"
              :theme="theme"
              read-only
            />
          </div>
        </div>
      </template>
    </div>

    <div v-else class="empty-state">
      选择左侧文件查看差异
    </div>

    <div v-if="currentFile && diffStats" class="diff-stats">
      <span class="stat added">+{{ diffStats.added }}</span>
      <span class="stat removed">-{{ diffStats.removed }}</span>
      <span class="stat changed">~{{ diffStats.changed }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import CodeViewer from './CodeViewer.vue';

interface FileNode {
  name: string;
  path: string;
  type: 'file' | 'directory';
  status?: string;
  children: FileNode[];
  expanded?: boolean;
}

interface DiffLine {
  lineNum: number;
  content: string;
  changeType: string;
  isDiff: boolean;
}

interface DiffStats {
  added: number;
  removed: number;
  changed: number;
}

const props = defineProps<{
  currentFile: FileNode | null;
  leftLines: DiffLine[];
  rightLines: DiffLine[];
  isBinary: boolean;
  diffStats: DiffStats | null;
  viewMode: 'working' | 'staged';
  theme?: 'light' | 'dark';
}>();

// 将 DiffLine 数组转换为文本内容
const oldContent = computed(() => {
  return props.rightLines.map(line => line.content).join('\n');
});

const newContent = computed(() => {
  return props.leftLines.map(line => line.content).join('\n');
});
</script>

<style scoped>
.diff-viewer {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background-color: var(--bg-primary);
}

.file-info-bar {
  display: flex;
  padding: 8px 16px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  gap: 20px;
}

.file-info {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-label {
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  font-weight: 600;
}

.file-path {
  font-size: 13px;
  color: var(--text-primary);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.diff-content {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.binary-placeholder {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.diff-pane {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.pane-header {
  padding: 6px 12px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  font-size: 11px;
  color: var(--text-secondary);
  text-transform: uppercase;
  font-weight: 600;
}

.code-content {
  flex: 1;
  overflow: hidden;
}

.diff-divider {
  width: 1px;
  background-color: var(--border-color);
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-secondary);
  font-size: 14px;
}

.diff-stats {
  display: flex;
  gap: 16px;
  padding: 8px 16px;
  background-color: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
}

.stat {
  font-weight: 600;
}

.stat.added {
  color: #4caf50;
}

.stat.removed {
  color: #f44336;
}

.stat.changed {
  color: #2196f3;
}
</style>
