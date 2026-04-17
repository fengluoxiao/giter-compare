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
      <button class="btn btn-secondary" @click="testNativePanel" title="原生面板">
        🍎 原生面板
      </button>
      <button class="btn btn-secondary" @click="testNativeAlert" title="原生警告">
        ⚠️ 原生警告
      </button>
      <button class="btn btn-secondary" @click="testNativeConfirm" title="原生确认">
        ❓ 原生确认
      </button>
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
      <button class="btn btn-secondary" @click="$emit('refresh', $event)" title="刷新 (Shift+ 点击强制刷新)">
        🔄 刷新
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';

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
  'refresh': [];
  'manage-plugins': [];
  'manage-workspace': [];
}>();

// 测试原生面板
const testNativePanel = async () => {
  try {
    await invoke('open_swiftui_panel', {
      request: {
        content: '这是从 Web 前端传递的内容！\n\n原生 macOS 面板可以显示丰富的内容。',
        title: '原生 macOS 面板测试',
        button_text: '点击回调前端'
      }
    });
  } catch (error) {
    console.error('打开原生面板失败:', error);
  }
};

// 测试原生警告
const testNativeAlert = async () => {
  try {
    await invoke('show_native_alert', {
      request: {
        title: '原生 macOS 警告',
        message: '这是一个使用 AppKit 构建的原生警告弹窗。',
        button_text: '我知道了'
      }
    });
  } catch (error) {
    console.error('显示原生警告失败:', error);
  }
};

// 测试原生确认
const testNativeConfirm = async () => {
  try {
    await invoke('show_native_alert', {
      request: {
        title: '请确认',
        message: '确定要执行此操作吗？此操作不可撤销。',
        button_text: '确认'
      }
    });
  } catch (error) {
    console.error('显示原生确认失败:', error);
  }
};
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
