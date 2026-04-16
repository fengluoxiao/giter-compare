<template>
  <Teleport to="body">
    <Transition name="fade">
      <div v-if="open" class="modal-overlay" @click.self="handleCancel">
        <div class="modal-container">
          <div class="modal-header">
            <h3 class="modal-title">{{ title }}</h3>
          </div>
          <div class="modal-body">
            <input
              ref="inputRef"
              v-model="inputValue"
              type="text"
              class="modal-input"
              :placeholder="placeholder"
              @keyup.enter="handleConfirm"
              @keyup.esc="handleCancel"
            />
          </div>
          <div class="modal-footer">
            <button class="modal-btn modal-btn--cancel" @click="handleCancel">取消</button>
            <button class="modal-btn modal-btn--confirm" @click="handleConfirm">确定</button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, nextTick } from 'vue'

interface Props {
  open: boolean
  title: string
  placeholder?: string
  defaultValue?: string
}

const props = defineProps<Props>()

const emit = defineEmits<{
  confirm: [value: string]
  cancel: []
}>()

const inputValue = ref('')
const inputRef = ref<HTMLInputElement | null>(null)

watch(
  () => props.open,
  (isOpen) => {
    if (isOpen) {
      inputValue.value = props.defaultValue || ''
      nextTick(() => {
        inputRef.value?.focus()
        inputRef.value?.select()
      })
    }
  }
)

const handleConfirm = () => {
  const value = inputValue.value.trim()
  if (value) {
    emit('confirm', value)
  }
}

const handleCancel = () => {
  emit('cancel')
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  padding: 20px;
}

.modal-container {
  background: #ffffff;
  border-radius: 8px;
  width: 100%;
  max-width: 400px;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.25);
  overflow: hidden;
}

.modal-header {
  padding: 20px 24px 0;
}

.modal-title {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: #1f2937;
  line-height: 1.4;
}

.modal-body {
  padding: 16px 24px;
}

.modal-input {
  width: 100%;
  height: 40px;
  padding: 0 12px;
  border: 1px solid #d1d5db;
  border-radius: 6px;
  font-size: 14px;
  color: #1f2937;
  background: #ffffff;
  transition: all 0.2s;
  outline: none;
  box-sizing: border-box;
}

.modal-input:focus {
  border-color: #3b82f6;
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
}

.modal-input::placeholder {
  color: #9ca3af;
}

.modal-footer {
  padding: 0 24px 20px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.modal-btn {
  padding: 8px 16px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  border: none;
}

.modal-btn--cancel {
  background: #f3f4f6;
  color: #374151;
}

.modal-btn--cancel:hover {
  background: #e5e7eb;
}

.modal-btn--confirm {
  background: #3b82f6;
  color: #ffffff;
}

.modal-btn--confirm:hover {
  background: #2563eb;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
