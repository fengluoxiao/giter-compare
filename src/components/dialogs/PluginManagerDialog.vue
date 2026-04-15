<template>
  <DialogBase :open="open" title="语法高亮插件管理" large @close="$emit('close')">
    <div class="plugin-manager">
      <!-- 默认内置插件提示 -->
      <div class="builtin-notice">
        <span class="notice-icon">ℹ️</span>
        <span class="notice-text">已内置 Vue 语法高亮支持</span>
      </div>

      <!-- 插件列表 -->
      <div class="plugin-list">
        <h4>已安装插件</h4>
        <div class="empty-state">
          暂无安装的插件
        </div>
      </div>

      <!-- 导入插件 -->
      <div class="import-section">
        <h4>导入新插件</h4>
        <div class="import-form">
          <button class="btn btn-primary" @click="selectPluginFolder">
            选择插件文件夹
          </button>
          <span class="import-hint">选择 VSCode 语法高亮插件文件夹</span>
        </div>
      </div>
    </div>

    <template #actions>
      <button class="btn btn-secondary" @click="$emit('close')">关闭</button>
    </template>
  </DialogBase>
</template>

<script setup lang="ts">
import DialogBase from './DialogBase.vue';

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
  'plugins-changed': [];
}>();

const selectPluginFolder = async () => {
  console.log('选择插件文件夹');
};
</script>

<style scoped>
.plugin-manager {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.builtin-notice {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background-color: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.notice-icon {
  font-size: 16px;
}

.notice-text {
  font-size: 13px;
  color: var(--text-primary);
}

.plugin-list h4,
.import-section h4 {
  font-size: 14px;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.empty-state {
  padding: 24px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
  background-color: var(--bg-secondary);
  border-radius: 6px;
}

.import-section {
  border-top: 1px solid var(--border-color);
  padding-top: 20px;
}

.import-form {
  display: flex;
  align-items: center;
  gap: 12px;
}

.import-hint {
  font-size: 12px;
  color: var(--text-secondary);
}

.btn-primary {
  padding: 8px 16px;
  background-color: var(--accent-color);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.btn-primary:hover {
  opacity: 0.9;
}

.btn-secondary {
  padding: 8px 16px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.btn-secondary:hover {
  background-color: var(--bg-hover);
}
</style>
