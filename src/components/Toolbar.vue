<template>
  <div class="toolbar-container">
    <div class="swiftui-test-buttons">
      <button class="test-btn" @click="testSwiftUIPanel" title="测试 SwiftUI 面板">
        <span class="icon">🍎</span>
        <span class="label">SwiftUI</span>
      </button>
      <button class="test-btn" @click="testNativeAlert" title="测试原生警告">
        <span class="icon">⚠️</span>
        <span class="label">警告</span>
      </button>
      <button class="test-btn" @click="testConfirm" title="测试确认对话框">
        <span class="icon">❓</span>
        <span class="label">确认</span>
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { openSwiftUIPanel, showNativeAlert, showConfirm, showInfo } from '../utils/swiftuiNative';

const props = defineProps<{
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

// 测试 SwiftUI 面板
const testSwiftUIPanel = async () => {
  try {
    const result = await openSwiftUIPanel({
      content: '这是从 Web 前端传递的内容！\n\nSwiftUI 原生面板可以显示丰富的内容。',
      title: 'SwiftUI 原生面板测试',
      buttonText: '点击回调前端'
    });
    console.log('面板返回结果:', result);
    await showInfo(`用户操作: ${result.status}`, '回调结果');
  } catch (error) {
    console.error('测试失败:', error);
  }
};

// 测试原生警告
const testNativeAlert = async () => {
  try {
    const result = await showNativeAlert({
      title: '原生 SwiftUI 警告',
      message: '这是一个使用 SwiftUI 构建的原生警告弹窗，支持 macOS 和 iOS 平台。',
      buttonText: '我知道了'
    });
    console.log('警告返回结果:', result);
  } catch (error) {
    console.error('测试失败:', error);
  }
};

// 测试确认对话框
const testConfirm = async () => {
  try {
    const confirmed = await showConfirm(
      '确定要执行此操作吗？此操作不可撤销。',
      '请确认'
    );
    if (confirmed) {
      await showInfo('用户点击了确认', '结果');
    } else {
      await showInfo('用户取消了操作', '结果');
    }
  } catch (error) {
    console.error('测试失败:', error);
  }
};
</script>

<style scoped>
.toolbar-container {
  height: auto;
  padding: 8px 16px;
  background: var(--bg-secondary, #f5f5f5);
  border-bottom: 1px solid var(--border-color, #e0e0e0);
}

.swiftui-test-buttons {
  display: flex;
  gap: 8px;
  align-items: center;
}

.test-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid var(--border-color, #d0d0d0);
  border-radius: 6px;
  background: var(--bg-primary, #ffffff);
  color: var(--text-primary, #333333);
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.test-btn:hover {
  background: var(--bg-hover, #e8e8e8);
  border-color: var(--border-hover, #b0b0b0);
}

.test-btn:active {
  transform: scale(0.98);
}

.test-btn .icon {
  font-size: 14px;
}

.test-btn .label {
  font-weight: 500;
}

/* 暗色主题适配 */
@media (prefers-color-scheme: dark) {
  .toolbar-container {
    background: var(--bg-secondary, #2d2d2d);
    border-bottom-color: var(--border-color, #3d3d3d);
  }

  .test-btn {
    background: var(--bg-primary, #3d3d3d);
    border-color: var(--border-color, #4d4d4d);
    color: var(--text-primary, #e0e0e0);
  }

  .test-btn:hover {
    background: var(--bg-hover, #4d4d4d);
    border-color: var(--border-hover, #5d5d5d);
  }
}
</style>
