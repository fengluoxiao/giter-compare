<template>
  <div v-if="open" class="dialog-overlay" @click.self="handleCancel">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ title }}</h3>
        <button class="close-btn" @click="handleCancel">
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M18 6L6 18M6 6l12 12"/>
          </svg>
        </button>
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
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  backdrop-filter: blur(8px);
  animation: fadeIn 0.2s ease-out;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.dialog {
  background-color: var(--bg-primary);
  border-radius: 12px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  width: 90%;
  max-width: 420px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

.dialog-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
}

.dialog-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: -0.02em;
}

.close-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  border-radius: 6px;
  transition: all 0.2s;
  padding: 0;
}

.close-btn:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
}

.dialog-content {
  padding: 24px;
  background: linear-gradient(to bottom, var(--bg-primary), var(--bg-secondary));
}

.message {
  margin: 0 0 16px 0;
  color: var(--text-primary);
  font-size: 14px;
  line-height: 1.6;
  font-weight: 500;
}

.dialog-input {
  width: 100%;
  padding: 12px 16px;
  border: 1.5px solid var(--border-color);
  border-radius: 8px;
  background-color: var(--bg-primary);
  color: var(--text-primary);
  font-size: 14px;
  box-sizing: border-box;
  transition: all 0.2s;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
}

.dialog-input:hover {
  border-color: var(--text-secondary);
}

.dialog-input:focus {
  outline: none;
  border-color: var(--accent-color);
  box-shadow: 0 0 0 4px rgba(59, 130, 246, 0.15), 0 2px 6px rgba(0, 0, 0, 0.08);
  transform: translateY(-1px);
}

.dialog-input::placeholder {
  color: var(--text-secondary);
  opacity: 0.7;
}

.dialog-actions {
  display: flex;
  gap: 12px;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  justify-content: flex-end;
  background-color: var(--bg-primary);
}

.dialog-actions .btn {
  min-width: 80px;
  padding: 10px 20px;
  border-radius: 8px;
  border: 1px solid transparent;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.dialog-actions .btn-primary {
  background: linear-gradient(135deg, var(--accent-color), var(--accent-color));
  color: white;
  border-color: var(--accent-color);
  box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
}

.dialog-actions .btn-primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
  filter: brightness(1.05);
}

.dialog-actions .btn-primary:active {
  transform: translateY(0);
}

.dialog-actions .btn-secondary {
  background-color: transparent;
  color: var(--text-secondary);
  border-color: var(--border-color);
}

.dialog-actions .btn-secondary:hover {
  background-color: var(--bg-hover);
  color: var(--text-primary);
  border-color: var(--text-secondary);
}
</style>
