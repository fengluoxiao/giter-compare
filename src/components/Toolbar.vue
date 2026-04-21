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
      <!-- 中间区域留白 -->
    </div>
    <div class="toolbar-right">
      <button class="btn btn-secondary" @click="$emit('app-settings')" title="全局设置">
        ⚙️ 设置
      </button>
      <button class="btn btn-secondary" @click="$emit('navigate-prev')" title="上一个" :disabled="!hasPrev">
        ⬆️ 上一个
      </button>
      <button class="btn btn-secondary" @click="$emit('navigate-next')" title="下一个" :disabled="!hasNext">
        ⬇️ 下一个
      </button>
      <button class="btn btn-secondary" @click="$emit('refresh', $event)" title="刷新 (Shift+ 点击强制刷新)">
        🔄 刷新
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">


defineProps<{
  theme: string;
  hasPrev: boolean;
  hasNext: boolean;
}>();

const emit = defineEmits<{
  'compare-file': [];
  'compare-text': [];
  'toggle-theme': [];
  'navigate-prev': [];
  'navigate-next': [];
  'refresh': [event?: MouseEvent];
  'manage-plugins': [];
  'manage-workspace': [];
  'project-settings': [];
  'app-settings': [];
}>();

// 原生测试功能已移除
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
.toolbar-center,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-center {
  flex: 1;
  justify-content: center;
}
</style>
