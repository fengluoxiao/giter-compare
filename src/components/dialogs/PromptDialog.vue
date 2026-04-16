<template>
  <div v-if="open" class="dialog-overlay" @click.self="handleCancel">
    <div class="dialog">
      <div class="dialog-header">
        <h3>{{ title }}</h3>
      </div>
      <div class="dialog-content">
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
        <button class="btn btn-primary" @click="handleConfirm">确定</button>
        <button class="btn btn-secondary" @click="handleCancel">取消</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue';

const props = defineProps<{
  open: boolean;
  title: string;
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
  background-color: rgba(0, 0, 0, 0.4);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
}

.dialog {
  background-color: #f3f3f3;
  border-radius: 6px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  width: 90%;
  max-width: 360px;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  border: 1px solid #e0e0e0;
}

.dialog-header {
  padding: 16px 20px 8px;
}

.dialog-header h3 {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: #333;
}

.dialog-content {
  padding: 8px 20px 16px;
}

.dialog-input {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid #ccc;
  border-radius: 3px;
  background-color: #fff;
  color: #333;
  font-size: 13px;
  box-sizing: border-box;
  transition: border-color 0.2s;
  outline: none;
}

.dialog-input:focus {
  border-color: #007acc;
}

.dialog-input::placeholder {
  color: #999;
}

.dialog-actions {
  display: flex;
  gap: 8px;
  padding: 12px 20px 16px;
  justify-content: flex-end;
}

.dialog-actions .btn {
  padding: 6px 16px;
  border-radius: 3px;
  border: 1px solid transparent;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
  transition: all 0.15s;
}

.dialog-actions .btn-primary {
  background-color: #0e639c;
  color: white;
  border-color: #0e639c;
}

.dialog-actions .btn-primary:hover {
  background-color: #1177bb;
  border-color: #1177bb;
}

.dialog-actions .btn-secondary {
  background-color: #fff;
  color: #333;
  border-color: #ccc;
}

.dialog-actions .btn-secondary:hover {
  background-color: #e6e6e6;
  border-color: #b3b3b3;
}
</style>
