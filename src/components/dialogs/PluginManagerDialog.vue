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
        <div v-if="loading" class="loading-state">
          加载中...
        </div>
        <div v-else-if="plugins.length === 0" class="empty-state">
          暂无安装的插件
        </div>
        <div v-else class="plugin-items">
          <div v-for="plugin in plugins" :key="plugin.id" class="plugin-item">
            <div class="plugin-info">
              <div class="plugin-header">
                <span class="plugin-name">{{ plugin.name }}</span>
                <span class="plugin-version">v{{ plugin.version }}</span>
              </div>
              <div class="plugin-description">{{ plugin.description }}</div>
              <div class="plugin-grammars">
                <span v-for="grammar in plugin.grammars" :key="grammar.language" class="grammar-tag">
                  {{ grammar.language }}
                </span>
              </div>
            </div>
            <button class="btn btn-danger" @click="removePlugin(plugin.id)">
              删除
            </button>
          </div>
        </div>
      </div>

      <!-- 导入插件 -->
      <div class="import-section">
        <h4>导入新插件</h4>
        <div class="import-form">
          <button class="btn btn-primary" @click="selectPluginFolder" :disabled="importing">
            {{ importing ? '导入中...' : '选择插件文件夹' }}
          </button>
          <span class="import-hint">选择 VSCode 语法高亮插件文件夹</span>
        </div>
        <div v-if="importError" class="import-error">
          {{ importError }}
        </div>
      </div>
    </div>

    <template #actions>
      <button class="btn btn-secondary" @click="$emit('close')">关闭</button>
    </template>
  </DialogBase>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import DialogBase from './DialogBase.vue';

interface GrammarInfo {
  language: string;
  scope_name: string;
  path: string;
}

interface PluginInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  grammars: GrammarInfo[];
}

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
  'plugins-changed': [];
}>();

const plugins = ref<PluginInfo[]>([]);
const loading = ref(false);
const importing = ref(false);
const importError = ref('');

// 加载已安装的插件
const loadPlugins = async () => {
  loading.value = true;
  try {
    const result = await invoke<PluginInfo[]>('get_installed_plugins');
    plugins.value = result;
  } catch (e) {
    console.error('Failed to load plugins:', e);
  } finally {
    loading.value = false;
  }
};

// 选择插件文件夹
const selectPluginFolder = async () => {
  importError.value = '';
  
  try {
    const selected = await openDialog({
      directory: true,
      multiple: false,
      title: '选择 VSCode 语法高亮插件文件夹'
    });

    if (selected && typeof selected === 'string') {
      importing.value = true;
      
      try {
        await invoke<PluginInfo>('import_vscode_plugin', {
          pluginPath: selected
        });
        
        // 重新加载插件列表
        await loadPlugins();
        emit('plugins-changed');
      } catch (e) {
        console.error('Failed to import plugin:', e);
        importError.value = `导入失败: ${e}`;
      } finally {
        importing.value = false;
      }
    }
  } catch (e) {
    console.error('Failed to select folder:', e);
    importError.value = `选择文件夹失败: ${e}`;
  }
};

// 删除插件
const removePlugin = async (pluginId: string) => {
  if (!confirm('确定要删除这个插件吗？')) {
    return;
  }
  
  try {
    await invoke('remove_plugin', { pluginId });
    await loadPlugins();
    emit('plugins-changed');
  } catch (e) {
    console.error('Failed to remove plugin:', e);
    alert(`删除失败: ${e}`);
  }
};

// 组件挂载时加载插件
onMounted(() => {
  loadPlugins();
});
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

.loading-state,
.empty-state {
  padding: 24px;
  text-align: center;
  color: var(--text-secondary);
  font-size: 13px;
  background-color: var(--bg-secondary);
  border-radius: 6px;
}

.plugin-items {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.plugin-item {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  padding: 16px;
  background-color: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
  gap: 16px;
}

.plugin-info {
  flex: 1;
  min-width: 0;
}

.plugin-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.plugin-name {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-primary);
}

.plugin-version {
  font-size: 12px;
  color: var(--text-secondary);
  background-color: var(--bg-hover);
  padding: 2px 6px;
  border-radius: 4px;
}

.plugin-description {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 8px;
  line-height: 1.4;
}

.plugin-grammars {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.grammar-tag {
  font-size: 11px;
  color: var(--text-secondary);
  background-color: var(--bg-hover);
  padding: 2px 8px;
  border-radius: 12px;
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

.import-error {
  margin-top: 12px;
  padding: 12px;
  background-color: rgba(244, 67, 54, 0.1);
  border: 1px solid rgba(244, 67, 54, 0.3);
  border-radius: 6px;
  color: #f44336;
  font-size: 13px;
}

.btn-primary {
  padding: 8px 16px;
  background-color: var(--accent-color);
  color: white;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: opacity 0.2s;
}

.btn-primary:hover:not(:disabled) {
  opacity: 0.9;
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  padding: 8px 16px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.btn-secondary:hover {
  background-color: var(--bg-hover);
}

.btn-danger {
  padding: 6px 12px;
  background-color: rgba(244, 67, 54, 0.1);
  color: #f44336;
  border: 1px solid rgba(244, 67, 54, 0.3);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}

.btn-danger:hover {
  background-color: rgba(244, 67, 54, 0.2);
}
</style>
