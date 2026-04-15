<template>
  <DialogBase :open="open" title="添加项目" large @close="$emit('close')">
    <div class="add-project-form">
      <div class="form-row">
        <div class="form-group form-group-name">
          <label>项目备注（可选）</label>
          <input
            :value="projectName"
            @input="$emit('update:projectName', ($event.target as HTMLInputElement).value)"
            placeholder="留空则使用文件夹名称"
          />
        </div>
        <button class="btn btn-primary" @click="$emit('select-path')">选择文件夹</button>
      </div>
    </div>

    <!-- 待添加项目列表 -->
    <div v-if="pendingProjects.length > 0" class="pending-projects">
      <h4>待添加项目 ({{ pendingProjects.length }})</h4>
      <div class="pending-list">
        <div v-for="(item, index) in pendingProjects" :key="index" class="pending-item">
          <div class="pending-name-wrapper">
            <input
              v-if="editingIndex === index"
              :value="editingName"
              @input="$emit('update:editingName', ($event.target as HTMLInputElement).value)"
              class="pending-name-input"
              @blur="$emit('save-name', index)"
              @keyup.enter="$emit('save-name', index)"
              @keyup.esc="$emit('cancel-edit')"
              ref="pendingNameInput"
            />
            <span
              v-else
              class="pending-name editable"
              @click="$emit('start-edit', index, item.name)"
              title="点击修改备注"
            >{{ item.name }}</span>
          </div>
          <span class="pending-path">{{ item.path }}</span>
          <button class="btn btn-icon btn-remove" @click="$emit('remove', index)" title="删除">✕</button>
        </div>
      </div>
    </div>

    <template #actions>
      <button class="btn btn-secondary" @click="$emit('close')">取消</button>
      <button
        class="btn btn-primary"
        @click="$emit('confirm')"
        :disabled="pendingProjects.length === 0"
      >
        确定添加 ({{ pendingProjects.length }})
      </button>
    </template>
  </DialogBase>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';
import DialogBase from './DialogBase.vue';

interface PendingProject {
  name: string;
  path: string;
}

const props = defineProps<{
  open: boolean;
  projectName: string;
  pendingProjects: PendingProject[];
  editingIndex: number;
  editingName: string;
}>();

const emit = defineEmits<{
  'close': [];
  'update:projectName': [value: string];
  'select-path': [];
  'remove': [index: number];
  'confirm': [];
  'start-edit': [index: number, name: string];
  'save-name': [index: number];
  'cancel-edit': [];
  'update:editingName': [value: string];
}>();

const pendingNameInput = ref<HTMLInputElement | null>(null);

// 当开始编辑时聚焦输入框
watch(() => props.editingIndex, (newIndex) => {
  if (newIndex >= 0) {
    nextTick(() => {
      pendingNameInput.value?.focus();
      pendingNameInput.value?.select();
    });
  }
});
</script>

<style scoped>
.add-project-form {
  margin-bottom: 20px;
}

.form-row {
  display: flex;
  gap: 12px;
  align-items: flex-end;
}

.form-group {
  flex: 1;
}

.form-group label {
  display: block;
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
  margin-bottom: 8px;
}

.form-group input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 14px;
}

.form-group input:focus {
  outline: none;
  border-color: var(--accent-color);
}

.pending-projects {
  border-top: 1px solid var(--border-color);
  padding-top: 20px;
}

.pending-projects h4 {
  font-size: 14px;
  color: var(--text-primary);
  margin: 0 0 12px 0;
}

.pending-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-height: 200px;
  overflow-y: auto;
}

.pending-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 12px;
  background-color: var(--bg-secondary);
  border-radius: 6px;
  border: 1px solid var(--border-color);
}

.pending-name-wrapper {
  flex: 0 0 150px;
}

.pending-name {
  font-size: 13px;
  color: var(--text-primary);
  font-weight: 500;
}

.pending-name.editable {
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background-color 0.2s;
}

.pending-name.editable:hover {
  background-color: var(--bg-hover);
}

.pending-name-input {
  width: 100%;
  padding: 4px 8px;
  border: 1px solid var(--accent-color);
  border-radius: 4px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 13px;
}

.pending-path {
  flex: 1;
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.btn-remove {
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 4px;
  font-size: 14px;
}

.btn-remove:hover {
  background-color: var(--bg-hover);
  color: #f44336;
}
</style>
