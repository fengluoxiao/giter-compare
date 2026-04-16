<template>
  <div v-if="open" class="dialog-overlay" @click.self="handleCancel">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ title }}</h3>
        <button class="close-btn" @click="handleCancel">×</button>
      </div>
      <div class="dialog-content">
        <p v-if="message" class="message">{{ message }}</p>
        <input
          ref="inputRef"
          v-model="inputValue"
          type="text"
          :placeholder="placeholder"
          class="dialog-input"
          @keyup.enter="handleConfirm"
          @keyup.escape="handleCancel"
        />
      </div>
      <div class="dialog-actions">
        <button class="btn btn-secondary" @click="handleCancel">取消</button>
        <button class="btn btn-primary" @click="handleConfirm">确定</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
  open: boolean;
  title: string;
  message?: string;
  placeholder?: string;
  defaultValue?: string;
}>();

const emit = defineEmits<{
  confirm: [value: string];
  cancel: [];
}>();

const inputValue = ref('');
const inputRef = ref<HTMLInputElement | null>(null);

watch(() => props.open, (newVal) => {
  if (newVal) {
    inputValue.value = props.defaultValue || '';
    nextTick(() => {
      inputRef.value?.focus();
    });
  }
});

const handleConfirm = () => {
  if (inputValue.value.trim()) {
    emit('confirm', inputValue.value.trim());
  }
};

const handleCancel = () => {
  emit('cancel');
};
</script>

<style scoped>
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  backdrop-filter: blur(4px);
}

.dialog {
  background-color: var(--bg-primary);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  width: 90%;
  max-width: 400px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.dialog-content {
  padding: 20px;
  background-color: var(--bg-secondary);
}

.message {
  margin: 0 0 12px 0;
  color: var(--text-primary);
  font-size: 14px;
  font-weight: 500;
}

.dialog-input {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  box-sizing: border-box;
  transition: all 0.2s;
}

.dialog-input:focus {
  outline: none;
  border-color: var(--accent-color);
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.dialog-actions {
  display: flex;
  gap: 12px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  justify-content: flex-end;
  background-color: var(--bg-primary);
}

.dialog-actions .btn {
  padding: 8px 16px;
  border-radius: 6px;
  border: 1px solid transparent;
  cursor: pointer;
  font-size: 14px;
  transition: all 0.2s;
}

.dialog-actions .btn-primary {
  background-color: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.dialog-actions .btn-primary:hover {
  opacity: 0.9;
}

.dialog-actions .btn-secondary {
  background-color: transparent;
  color: var(--text-secondary);
  border-color: var(--border-color);
}

.dialog-actions .btn-secondary:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}
</style>
