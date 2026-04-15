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
        <div v-if="plugins.length === 0" class="empty-state">
          暂无安装的插件
        </div>
        <div
          v-for="plugin in plugins"
          :key="plugin.id"
          class="plugin-item"
          :class="{ disabled: !plugin.enabled }"
        >
          <div class="plugin-info">
            <div class="plugin-header">
              <span class="plugin-name">{{ plugin.name }}</span>
              <span class="plugin-version">v{{ plugin.version }}</span>
            </div>
            <div class="plugin-description">{{ plugin.description }}</div>
            <div class="plugin-languages">
              支持语言: {{ plugin.languages.join(', ') }}
            </div>
          </div>
          <div class="plugin-actions">
            <button
              class="btn btn-icon"
              :class="plugin.enabled ? 'btn-disable' : 'btn-enable'"
              @click="togglePlugin(plugin)"
              :title="plugin.enabled ? '禁用' : '启用'"
            >
              {{ plugin.enabled ? '禁用' : '启用' }}
            </button>
            <button
              class="btn btn-icon btn-delete"
              @click="uninstallPlugin(plugin.id)"
              title="卸载"
            >
              <svg viewBox="0 0 24 24" width="14" height="14" fill="currentColor">
                <path d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"/>
              </svg>
            </button>
          </div>
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

  <!-- 冲突提示弹窗 -->
  <DialogBase
    v-if="showConflictDialog"
    :open="true"
    title="插件冲突"
    @close="showConflictDialog = false"
  >
    <div class="conflict-content">
      <p>以下插件与要安装的插件存在语言冲突：</p>
      <ul class="conflict-list">
        <li v-for="conflict in pendingConflicts" :key="conflict">{{ conflict }}</li>
      </ul>
      <p>请选择操作：</p>
    </div>
    <template #actions>
      <button class="btn btn-secondary" @click="showConflictDialog = false">取消安装</button>
      <button class="btn btn-warning" @click="disableConflictsAndInstall">禁用冲突插件并安装</button>
    </template>
  </DialogBase>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import DialogBase from './DialogBase.vue';
import { pluginManager, type GrammarPlugin } from '../../utils/pluginManager';

const props = defineProps<{
  open: boolean;
}>();

const emit = defineEmits<{
  close: [];
  'plugins-changed': [];
}>();

const plugins = ref<GrammarPlugin[]>([]);
const showConflictDialog = ref(false);
const pendingConflicts = ref<string[]>([]);
const pendingPluginPath = ref('');

onMounted(async () => {
  await pluginManager.init();
  loadPlugins();
});

const loadPlugins = () => {
  plugins.value = pluginManager.getPlugins();
};

const selectPluginFolder = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择 VSCode 语法高亮插件文件夹'
    });

    if (selected && typeof selected === 'string') {
      const result = await pluginManager.installPlugin(selected);
      
      if (result.success) {
        loadPlugins();
        emit('plugins-changed');
        alert(result.message);
      } else if (result.conflicts && result.conflicts.length > 0) {
        pendingConflicts.value = result.conflicts;
        pendingPluginPath.value = selected;
        showConflictDialog.value = true;
      } else {
        alert(result.message);
      }
    }
  } catch (e) {
    console.error('Failed to select plugin folder:', e);
    alert('选择插件文件夹失败');
  }
};

const disableConflictsAndInstall = async () => {
  // 禁用冲突的插件
  for (const plugin of plugins.value) {
    if (pendingConflicts.value.some(c => c.includes(plugin.name))) {
      await pluginManager.disablePlugin(plugin.id);
    }
  }

  // 重新安装
  const result = await pluginManager.installPlugin(pendingPluginPath.value);
  if (result.success) {
    loadPlugins();
    emit('plugins-changed');
    alert(result.message);
  } else {
    alert(result.message);
  }

  showConflictDialog.value = false;
  pendingConflicts.value = [];
  pendingPluginPath.value = '';
};

const togglePlugin = async (plugin: GrammarPlugin) => {
  if (plugin.enabled) {
    await pluginManager.disablePlugin(plugin.id);
  } else {
    await pluginManager.enablePlugin(plugin.id);
  }
  loadPlugins();
  emit('plugins-changed');
};

const uninstallPlugin = async (pluginId: string) => {
  if (confirm('确定要卸载此插件吗？')) {
    await pluginManager.uninstallPlugin(pluginId);
    loadPlugins();
    emit('plugins-changed');
  }
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

.plugin-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 12px;
  background-color: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
  margin-bottom: 8px;
}

.plugin-item.disabled {
  opacity: 0.6;
}

.plugin-info {
  flex: 1;
}

.plugin-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.plugin-name {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
}

.plugin-version {
  font-size: 11px;
  color: var(--text-secondary);
  background-color: var(--bg-primary);
  padding: 2px 6px;
  border-radius: 3px;
}

.plugin-description {
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.plugin-languages {
  font-size: 11px;
  color: var(--text-secondary);
}

.plugin-actions {
  display: flex;
  gap: 8px;
}

.btn-enable {
  background-color: var(--accent-color);
  color: white;
  border: none;
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.btn-disable {
  background-color: var(--bg-primary);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  padding: 4px 12px;
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
}

.btn-delete {
  width: 28px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
}

.btn-delete:hover {
  background-color: var(--bg-hover);
  color: #f44336;
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

.conflict-content {
  font-size: 14px;
  color: var(--text-primary);
}

.conflict-list {
  margin: 12px 0;
  padding-left: 20px;
}

.conflict-list li {
  margin-bottom: 4px;
  color: var(--text-secondary);
}

.btn-warning {
  background-color: #ff9800;
  color: white;
  border: none;
  padding: 8px 16px;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
}

.btn-warning:hover {
  background-color: #f57c00;
}
</style>
