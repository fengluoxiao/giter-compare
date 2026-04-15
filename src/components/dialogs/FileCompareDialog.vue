<template>
  <DialogBase :open="open" title="选择要比对的文件" @close="$emit('close')">
    <div class="text-inputs">
      <div class="text-input-group">
        <label>旧版本</label>
        <button class="btn btn-secondary" @click="$emit('select-old')">选择文件</button>
        <span v-if="oldPath" class="selected-path">{{ oldPath }}</span>
      </div>
      <div class="text-input-group">
        <label>新版本</label>
        <button class="btn btn-secondary" @click="$emit('select-new')">选择文件</button>
        <span v-if="newPath" class="selected-path">{{ newPath }}</span>
      </div>
    </div>
    <template #actions>
      <button class="btn btn-secondary" @click="$emit('close')">取消</button>
      <button class="btn btn-primary" @click="$emit('compare')" :disabled="!oldPath || !newPath">比对</button>
    </template>
  </DialogBase>
</template>

<script setup lang="ts">
import DialogBase from './DialogBase.vue';

defineProps<{
  open: boolean;
  oldPath: string;
  newPath: string;
}>();

defineEmits<{
  'close': [];
  'select-old': [];
  'select-new': [];
  'compare': [];
}>();
</script>

<style scoped>
.text-inputs {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.text-input-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.text-input-group label {
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

.selected-path {
  font-size: 12px;
  color: var(--text-secondary);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  word-break: break-all;
  padding: 8px;
  background-color: var(--bg-tertiary);
  border-radius: 4px;
}
</style>
