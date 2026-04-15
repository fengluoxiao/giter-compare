<template>
  <DialogBase :open="open" title="文本比对" @close="$emit('close')">
    <div class="text-inputs">
      <div class="text-input-group">
        <label>文本 1</label>
        <textarea
          :value="text1"
          @input="$emit('update:text1', ($event.target as HTMLTextAreaElement).value)"
          placeholder="在此粘贴第一段文本..."
        ></textarea>
      </div>
      <div class="text-input-group">
        <label>文本 2</label>
        <textarea
          :value="text2"
          @input="$emit('update:text2', ($event.target as HTMLTextAreaElement).value)"
          placeholder="在此粘贴第二段文本..."
        ></textarea>
      </div>
    </div>
    <template #actions>
      <button class="btn btn-secondary" @click="$emit('close')">取消</button>
      <button class="btn btn-primary" @click="$emit('compare')">比对</button>
    </template>
  </DialogBase>
</template>

<script setup lang="ts">
import DialogBase from './DialogBase.vue';

defineProps<{
  open: boolean;
  text1: string;
  text2: string;
}>();

defineEmits<{
  'close': [];
  'update:text1': [value: string];
  'update:text2': [value: string];
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

.text-input-group textarea {
  width: 100%;
  height: 150px;
  padding: 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-color: var(--bg-secondary);
  color: var(--text-primary);
  font-family: 'JetBrains Mono', 'Fira Code', monospace;
  font-size: 13px;
  resize: vertical;
  line-height: 1.5;
}

.text-input-group textarea:focus {
  outline: none;
  border-color: var(--accent-color);
}

.text-input-group textarea::placeholder {
  color: var(--text-secondary);
  opacity: 0.6;
}
</style>
