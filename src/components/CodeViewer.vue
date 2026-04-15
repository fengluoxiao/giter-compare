<template>
  <div ref="editorContainer" class="code-viewer"></div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch } from 'vue';
import * as monaco from 'monaco-editor';
import { initMonacoGrammars, getLanguageFromFilename, setMonacoTheme } from '../utils/monacoConfig';

const props = defineProps<{
  content: string;
  filename: string;
  theme?: 'light' | 'dark';
  readOnly?: boolean;
}>();

const editorContainer = ref<HTMLElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;

onMounted(async () => {
  if (!editorContainer.value) return;

  // 初始化 Monaco grammars
  await initMonacoGrammars(props.theme || 'light');

  editor = monaco.editor.create(editorContainer.value, {
    value: props.content,
    language: getLanguageFromFilename(props.filename),
    theme: 'giter-compare',
    readOnly: props.readOnly ?? true,
    minimap: { enabled: false },
    scrollBeyondLastLine: false,
    automaticLayout: true,
    fontSize: 13,
    lineNumbers: 'on',
    renderWhitespace: 'selection',
    folding: true,
    lineDecorationsWidth: 50,
    renderLineHighlight: 'line',
    selectOnLineNumbers: true,
    matchBrackets: 'always',
    autoIndent: 'full',
    formatOnPaste: true,
    formatOnType: true,
    wordWrap: 'off',
  });
});

onUnmounted(() => {
  if (editor) {
    editor.dispose();
    editor = null;
  }
});

watch(() => props.content, (newContent) => {
  if (editor) {
    const currentPosition = editor.getPosition();
    editor.setValue(newContent);
    if (currentPosition) {
      editor.setPosition(currentPosition);
    }
  }
});

watch(() => props.filename, (newFilename) => {
  if (editor) {
    monaco.editor.setModelLanguage(editor.getModel()!, getLanguageFromFilename(newFilename));
  }
});

watch(() => props.theme, (newTheme) => {
  if (newTheme) {
    setMonacoTheme(newTheme);
  }
});
</script>

<style scoped>
.code-viewer {
  width: 100%;
  height: 100%;
}
</style>
