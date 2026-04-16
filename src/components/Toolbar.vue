<template>
  <div class="toolbar">
    <div class="toolbar-left">
      <button class="btn btn-secondary" @click="$emit('compare-file')">
        <span class="btn-icon">📄</span>
        比对文件
      </button>
      <button class="btn btn-secondary" @click="$emit('compare-text')">
        <span class="btn-icon">📝</span>
        比对文本
      </button>
    </div>
    <div class="toolbar-center">
      <span class="path-display">{{ currentPath || '未选择文件夹' }}</span>
    </div>
    <div class="toolbar-right">
      <button class="btn btn-secondary" @click="$emit('manage-workspace')" title="工作区管理">
        💼 工作区
      </button>
      <button class="btn btn-secondary" @click="$emit('manage-plugins')" title="语法高亮插件">
        🔌 插件
      </button>
      <button class="btn btn-secondary" @click="$emit('toggle-theme')" title="切换主题">
        {{ theme === 'dark' ? '☀️ 浅色' : '🌙 深色' }}
      </button>
      <button class="btn btn-secondary" @click="$emit('navigate-prev')" title="上一个" :disabled="!hasPrev">
        ⬆️ 上一个
      </button>
      <button class="btn btn-secondary" @click="$emit('navigate-next')" title="下一个" :disabled="!hasNext">
        ⬇️ 下一个
      </button>
      <button class="btn btn-secondary" @click="$emit('refresh')" title="刷新">
        🔄 刷新
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  theme: string;
  currentPath: string;
  hasPrev: boolean;
  hasNext: boolean;
}>();

defineEmits<{
  'compare-file': [];
  'compare-text': [];
  'toggle-theme': [];
  'navigate-prev': [];
  'navigate-next': [];
  'refresh': [];
  'manage-plugins': [];
  'manage-workspace': [];
}>();
</script>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background-color: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  gap: 12px;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-center {
  flex: 1;
  text-align: center;
  overflow: hidden;
}

.path-display {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
</style>
