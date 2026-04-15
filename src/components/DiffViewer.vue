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
          <div class="code-content" ref="leftCodeContent" @scroll="syncScroll('left')">
            <DiffLines :lines="rightLines" />
          </div>
        </div>
        <div class="diff-divider"></div>
        <div class="diff-pane">
          <div class="pane-header">
            <span class="pane-title">{{ viewMode === 'working' ? '工作区' : '暂存区' }}</span>
          </div>
          <div class="code-content" ref="rightCodeContent" @scroll="syncScroll('right')">
            <DiffLines :lines="leftLines" />
          </div>
        </div>
        <!-- Minimap -->
        <Minimap
          :lines="leftLines"
          :scroll-top="leftScrollTop"
          :container-height="codeContainerHeight"
          :content-height="codeContentHeight"
          @jump="handleMinimapJump"
        />
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
import { ref, watch, nextTick } from 'vue';
import DiffLines from './DiffLines.vue';
import Minimap from './Minimap.vue';

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
}>();

const emit = defineEmits<{
  'scroll': [scrollTop: number];
}>();

// 代码内容区域 refs，用于同步滚动
const leftCodeContent = ref<HTMLElement | null>(null);
const rightCodeContent = ref<HTMLElement | null>(null);
let isSyncing = false;

// Minimap 相关
const leftScrollTop = ref(0);
const codeContainerHeight = ref(0);
const codeContentHeight = ref(0);

// 同步滚动函数
const syncScroll = (source: 'left' | 'right') => {
  if (isSyncing) return;
  isSyncing = true;

  const sourceEl = source === 'left' ? leftCodeContent.value : rightCodeContent.value;
  const targetEl = source === 'left' ? rightCodeContent.value : leftCodeContent.value;

  if (sourceEl && targetEl) {
    // 同步纵向滚动
    targetEl.scrollTop = sourceEl.scrollTop;
    // 同步横向滚动
    targetEl.scrollLeft = sourceEl.scrollLeft;
  }

  // 更新 minimap 的 scrollTop
  if (sourceEl) {
    leftScrollTop.value = sourceEl.scrollTop;
    codeContainerHeight.value = sourceEl.clientHeight;
    codeContentHeight.value = sourceEl.scrollHeight;
    emit('scroll', sourceEl.scrollTop);
  }

  // 使用 requestAnimationFrame 确保同步
  requestAnimationFrame(() => {
    isSyncing = false;
  });
};

// Minimap 跳转处理
const handleMinimapJump = (lineIndex: number) => {
  if (!leftCodeContent.value) return;

  // 计算目标滚动位置
  const lineHeight = 24; // 每行高度
  const targetScrollTop = lineIndex * lineHeight;

  leftCodeContent.value.scrollTop = targetScrollTop;
  if (rightCodeContent.value) {
    rightCodeContent.value.scrollTop = targetScrollTop;
  }

  // 更新 minimap 状态
  leftScrollTop.value = targetScrollTop;
};

// 监听线条变化，更新 minimap 尺寸
watch(() => props.leftLines, () => {
  nextTick(() => {
    if (leftCodeContent.value) {
      codeContainerHeight.value = leftCodeContent.value.clientHeight;
      codeContentHeight.value = leftCodeContent.value.scrollHeight;
    }
  });
}, { deep: true });

// 暴露方法给父组件
defineExpose({
  leftCodeContent,
  rightCodeContent,
  syncScroll
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
  overflow: auto;
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  line-height: 24px;
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
