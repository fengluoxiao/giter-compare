<template>
  <div class="shiki-code-viewer" v-html="highlightedHtml"></div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { highlightCode } from '../utils/shikiHighlighter';

const props = defineProps<{
  code: string;
  filename: string;
  theme?: 'light' | 'dark';
}>();

const highlightedHtml = ref<string>('');

const updateHighlight = async () => {
  try {
    highlightedHtml.value = await highlightCode(
      props.code,
      props.filename,
      props.theme || 'light'
    );
  } catch (e) {
    console.error('Failed to highlight code:', e);
    highlightedHtml.value = `<pre><code>${escapeHtml(props.code)}</code></pre>`;
  }
};

const escapeHtml = (text: string): string => {
  return text
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
};

onMounted(() => {
  updateHighlight();
});

watch(() => [props.code, props.filename, props.theme], () => {
  updateHighlight();
}, { deep: true });
</script>

<style scoped>
.shiki-code-viewer {
  width: 100%;
  height: 100%;
  overflow: auto;
}

.shiki-code-viewer :deep(pre) {
  margin: 0;
  padding: 16px;
  font-family: 'JetBrains Mono', 'Fira Code', 'Consolas', monospace;
  font-size: 13px;
  line-height: 1.5;
}

.shiki-code-viewer :deep(code) {
  font-family: inherit;
}
</style>
