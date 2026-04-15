import * as monaco from 'monaco-editor';
import { Registry, INITIAL, IRawTheme, IRawGrammar } from 'monaco-textmate';
import { wireTmGrammars } from 'monaco-editor-textmate';
import { pluginManager } from './pluginManager';

// VSCode 默认主题
const VSCodeLightTheme: IRawTheme = {
  name: 'vs',
  settings: [
    { settings: { foreground: '#000000', background: '#ffffff' } },
    { scope: 'emphasis', settings: { fontStyle: 'italic' } },
    { scope: 'strong', settings: { fontStyle: 'bold' } },
    { scope: 'header', settings: { foreground: '#0000ff' } },
    { scope: 'comment', settings: { foreground: '#008000' } },
    { scope: 'constant.language', settings: { foreground: '#0000ff' } },
    { scope: 'constant.numeric', settings: { foreground: '#098658' } },
    { scope: 'constant.regexp', settings: { foreground: '#811f3f' } },
    { scope: 'entity.name.tag', settings: { foreground: '#800000' } },
    { scope: 'entity.name.tag.css', settings: { foreground: '#800000' } },
    { scope: 'entity.name.tag.namespace', settings: { foreground: '#e50000' } },
    { scope: 'entity.other.attribute-name', settings: { foreground: '#e50000' } },
    { scope: 'entity.other.attribute-name.css', settings: { foreground: '#ff0000' } },
    { scope: 'invalid', settings: { foreground: '#cd3131' } },
    { scope: 'markup.underline', settings: { fontStyle: 'underline' } },
    { scope: 'markup.bold', settings: { fontStyle: 'bold' } },
    { scope: 'markup.heading', settings: { fontStyle: 'bold', foreground: '#800000' } },
    { scope: 'markup.italic', settings: { fontStyle: 'italic' } },
    { scope: 'markup.inserted', settings: { foreground: '#008000' } },
    { scope: 'markup.deleted', settings: { foreground: '#ff0000' } },
    { scope: 'markup.changed', settings: { foreground: '#0451a5' } },
    { scope: 'punctuation.definition.quote.begin.markdown', settings: { foreground: '#0451a5' } },
    { scope: 'punctuation.definition.list.begin.markdown', settings: { foreground: '#0451a5' } },
    { scope: 'markup.inline.raw', settings: { foreground: '#800000' } },
    { scope: 'punctuation.definition.tag', settings: { foreground: '#800000' } },
    { scope: 'meta.preprocessor', settings: { foreground: '#0000ff' } },
    { scope: 'meta.preprocessor.string', settings: { foreground: '#a31515' } },
    { scope: 'meta.preprocessor.numeric', settings: { foreground: '#098658' } },
    { scope: 'meta.structure.dictionary.key.python', settings: { foreground: '#0451a5' } },
    { scope: 'storage', settings: { foreground: '#0000ff' } },
    { scope: 'storage.type', settings: { foreground: '#0000ff' } },
    { scope: 'storage.modifier', settings: { foreground: '#0000ff' } },
    { scope: 'string.tag', settings: { foreground: '#800000' } },
    { scope: 'string.value', settings: { foreground: '#0451a5' } },
    { scope: 'string.regexp', settings: { foreground: '#811f3f' } },
    { scope: 'support.type.property-name', settings: { foreground: '#0451a5' } },
    { scope: 'keyword', settings: { foreground: '#0000ff' } },
    { scope: 'keyword.control', settings: { foreground: '#0000ff' } },
    { scope: 'keyword.operator', settings: { foreground: '#000000' } },
    { scope: 'keyword.operator.new', settings: { foreground: '#0000ff' } },
    { scope: 'keyword.operator.expression', settings: { foreground: '#0000ff' } },
    { scope: 'keyword.other.unit', settings: { foreground: '#098658' } },
    { scope: 'punctuation.section.embedded.begin.php', settings: { foreground: '#800000' } },
    { scope: 'punctuation.section.embedded.end.php', settings: { foreground: '#800000' } },
    { scope: 'support.function.git-rebase', settings: { foreground: '#0451a5' } },
    { scope: 'constant.sha.git-rebase', settings: { foreground: '#098658' } },
    { scope: 'storage.modifier.import.java', settings: { foreground: '#000000' } },
    { scope: 'storage.modifier.package.java', settings: { foreground: '#000000' } },
    { scope: 'variable.language', settings: { foreground: '#0000ff' } },
    { scope: 'entity.name.function', settings: { foreground: '#795e26' } },
    { scope: 'support.function', settings: { foreground: '#795e26' } },
    { scope: 'support.constant.handlebars', settings: { foreground: '#795e26' } },
    { scope: 'source.css.support.property-name', settings: { foreground: '#ff0000' } },
    { scope: 'source.css.support.property-value', settings: { foreground: '#0451a5' } },
    { scope: 'support.type', settings: { foreground: '#267f99' } },
    { scope: 'support.class', settings: { foreground: '#267f99' } },
    { scope: 'support.variable', settings: { foreground: '#001080' } },
    { scope: 'invalid.illegal.unrecognized-tag.html', settings: { fontStyle: 'normal', foreground: '#cd3131' } },
    { scope: 'entity.name.function.preprocessor', settings: { foreground: '#0000ff' } },
    { scope: 'meta.diff.header', settings: { foreground: '#0000ff' } },
    { scope: 'markup.deleted.diff', settings: { foreground: '#ff0000' } },
    { scope: 'markup.inserted.diff', settings: { foreground: '#008000' } },
    { scope: 'markup.changed.diff', settings: { foreground: '#0451a5' } },
  ]
};

// VSCode 暗色主题
const VSCodeDarkTheme: IRawTheme = {
  name: 'vs-dark',
  settings: [
    { settings: { foreground: '#d4d4d4', background: '#1e1e1e' } },
    { scope: 'emphasis', settings: { fontStyle: 'italic' } },
    { scope: 'strong', settings: { fontStyle: 'bold' } },
    { scope: 'header', settings: { foreground: '#569cd6' } },
    { scope: 'comment', settings: { foreground: '#6a9955' } },
    { scope: 'constant.language', settings: { foreground: '#569cd6' } },
    { scope: 'constant.numeric', settings: { foreground: '#b5cea8' } },
    { scope: 'constant.regexp', settings: { foreground: '#646695' } },
    { scope: 'entity.name.tag', settings: { foreground: '#569cd6' } },
    { scope: 'entity.name.tag.css', settings: { foreground: '#d7ba7d' } },
    { scope: 'entity.name.tag.namespace', settings: { foreground: '#4ec9b0' } },
    { scope: 'entity.other.attribute-name', settings: { foreground: '#9cdcfe' } },
    { scope: 'entity.other.attribute-name.css', settings: { foreground: '#d7ba7d' } },
    { scope: 'invalid', settings: { foreground: '#f44747' } },
    { scope: 'markup.underline', settings: { fontStyle: 'underline' } },
    { scope: 'markup.bold', settings: { fontStyle: 'bold' } },
    { scope: 'markup.heading', settings: { fontStyle: 'bold', foreground: '#569cd6' } },
    { scope: 'markup.italic', settings: { fontStyle: 'italic' } },
    { scope: 'markup.inserted', settings: { foreground: '#b5cea8' } },
    { scope: 'markup.deleted', settings: { foreground: '#ce9178' } },
    { scope: 'markup.changed', settings: { foreground: '#569cd6' } },
    { scope: 'punctuation.definition.quote.begin.markdown', settings: { foreground: '#6a9955' } },
    { scope: 'punctuation.definition.list.begin.markdown', settings: { foreground: '#6796e6' } },
    { scope: 'markup.inline.raw', settings: { foreground: '#ce9178' } },
    { scope: 'punctuation.definition.tag', settings: { foreground: '#808080' } },
    { scope: 'meta.preprocessor', settings: { foreground: '#569cd6' } },
    { scope: 'meta.preprocessor.string', settings: { foreground: '#ce9178' } },
    { scope: 'meta.preprocessor.numeric', settings: { foreground: '#b5cea8' } },
    { scope: 'meta.structure.dictionary.key.python', settings: { foreground: '#9cdcfe' } },
    { scope: 'storage', settings: { foreground: '#569cd6' } },
    { scope: 'storage.type', settings: { foreground: '#569cd6' } },
    { scope: 'storage.modifier', settings: { foreground: '#569cd6' } },
    { scope: 'string.tag', settings: { foreground: '#ce9178' } },
    { scope: 'string.value', settings: { foreground: '#9cdcfe' } },
    { scope: 'string.regexp', settings: { foreground: '#d16969' } },
    { scope: 'support.type.property-name', settings: { foreground: '#9cdcfe' } },
    { scope: 'keyword', settings: { foreground: '#569cd6' } },
    { scope: 'keyword.control', settings: { foreground: '#c586c0' } },
    { scope: 'keyword.operator', settings: { foreground: '#d4d4d4' } },
    { scope: 'keyword.operator.new', settings: { foreground: '#569cd6' } },
    { scope: 'keyword.operator.expression', settings: { foreground: '#569cd6' } },
    { scope: 'keyword.other.unit', settings: { foreground: '#b5cea8' } },
    { scope: 'punctuation.section.embedded.begin.php', settings: { foreground: '#569cd6' } },
    { scope: 'punctuation.section.embedded.end.php', settings: { foreground: '#569cd6' } },
    { scope: 'support.function.git-rebase', settings: { foreground: '#9cdcfe' } },
    { scope: 'constant.sha.git-rebase', settings: { foreground: '#b5cea8' } },
    { scope: 'storage.modifier.import.java', settings: { foreground: '#d4d4d4' } },
    { scope: 'storage.modifier.package.java', settings: { foreground: '#d4d4d4' } },
    { scope: 'variable.language', settings: { foreground: '#569cd6' } },
    { scope: 'entity.name.function', settings: { foreground: '#dcdcaa' } },
    { scope: 'support.function', settings: { foreground: '#dcdcaa' } },
    { scope: 'support.constant.handlebars', settings: { foreground: '#dcdcaa' } },
    { scope: 'source.css.support.property-name', settings: { foreground: '#9cdcfe' } },
    { scope: 'source.css.support.property-value', settings: { foreground: '#ce9178' } },
    { scope: 'support.type', settings: { foreground: '#4ec9b0' } },
    { scope: 'support.class', settings: { foreground: '#4ec9b0' } },
    { scope: 'support.variable', settings: { foreground: '#9cdcfe' } },
    { scope: 'invalid.illegal.unrecognized-tag.html', settings: { fontStyle: 'normal', foreground: '#f44747' } },
    { scope: 'entity.name.function.preprocessor', settings: { foreground: '#569cd6' } },
    { scope: 'meta.diff.header', settings: { foreground: '#569cd6' } },
    { scope: 'markup.deleted.diff', settings: { foreground: '#ff0000' } },
    { scope: 'markup.inserted.diff', settings: { foreground: '#00ff00' } },
    { scope: 'markup.changed.diff', settings: { foreground: '#569cd6' } },
  ]
};

let registry: Registry | null = null;
let grammarsLoaded = false;

// 内置 Vue 语法定义
const vueGrammar: IRawGrammar = {
  scopeName: 'source.vue',
  fileTypes: ['vue'],
  injectionSelector: 'L:source.js -comment -(string -meta.embedded)',
  patterns: [
    { include: '#template' },
    { include: '#script' },
    { include: '#style' },
  ],
  repository: {
    template: {
      begin: '(<)(template)',
      beginCaptures: {
        '1': { name: 'punctuation.definition.tag.begin.html' },
        '2': { name: 'entity.name.tag.template.html' }
      },
      end: '(</)(template)(>)',
      endCaptures: {
        '1': { name: 'punctuation.definition.tag.begin.html' },
        '2': { name: 'entity.name.tag.template.html' },
        '3': { name: 'punctuation.definition.tag.end.html' }
      },
      patterns: [
        { include: 'text.html.basic' }
      ]
    },
    script: {
      begin: '(<)(script)(?:(\\s+lang="ts")|(\\s+lang="typescript"))?',
      beginCaptures: {
        '1': { name: 'punctuation.definition.tag.begin.html' },
        '2': { name: 'entity.name.tag.script.html' },
        '3': { name: 'entity.other.attribute-name.html' },
        '4': { name: 'entity.other.attribute-name.html' }
      },
      end: '(</)(script)(>)',
      endCaptures: {
        '1': { name: 'punctuation.definition.tag.begin.html' },
        '2': { name: 'entity.name.tag.script.html' },
        '3': { name: 'punctuation.definition.tag.end.html' }
      },
      patterns: [
        { include: 'source.js' },
        { include: 'source.ts' }
      ]
    },
    style: {
      begin: '(<)(style)',
      beginCaptures: {
        '1': { name: 'punctuation.definition.tag.begin.html' },
        '2': { name: 'entity.name.tag.style.html' }
      },
      end: '(</)(style)(>)',
      endCaptures: {
        '1': { name: 'punctuation.definition.tag.begin.html' },
        '2': { name: 'entity.name.tag.style.html' },
        '3': { name: 'punctuation.definition.tag.end.html' }
      },
      patterns: [
        { include: 'source.css' }
      ]
    }
  }
};

export async function initMonacoGrammars(theme: 'light' | 'dark' = 'light'): Promise<Registry> {
  if (registry && grammarsLoaded) {
    return registry;
  }

  // 创建 TextMate Registry
  registry = new Registry({
    getOniguruma: async () => {
      const onig = await import('vscode-oniguruma');
      return onig;
    },
    loadGrammar: async (scopeName: string) => {
      // 加载内置 Vue 语法
      if (scopeName === 'source.vue') {
        return vueGrammar;
      }

      // 从插件加载语法
      const plugins = pluginManager.getPlugins();
      for (const plugin of plugins) {
        if (!plugin.enabled) continue;
        
        const grammar = pluginManager.getGrammarByScopeName(scopeName);
        if (grammar) {
          return grammar as IRawGrammar;
        }
      }

      return null;
    }
  });

  // 注册内置语言
  monaco.languages.register({ id: 'vue', extensions: ['.vue'] });

  // 定义语言配置
  monaco.languages.setLanguageConfiguration('vue', {
    wordPattern: /(-?\d*\.\d\w*)|([^\`\~\!\@\$\^\&\*\(\)\=\+\[\{\]\}\\\|\;\:\'\"\,\.\<\>\/\?\s]+)/g,
    brackets: [
      ['<!--', '-->'],
      ['<', '>'],
      ['{{', '}}'],
      ['{', '}'],
      ['[', ']'],
      ['(', ')']
    ],
    autoClosingPairs: [
      { open: '{', close: '}' },
      { open: '[', close: ']' },
      { open: '(', close: ')' },
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: '<', close: '>' }
    ],
    surroundingPairs: [
      { open: '"', close: '"' },
      { open: "'", close: "'" },
      { open: '<', close: '>' }
    ]
  });

  // 设置主题
  const selectedTheme = theme === 'dark' ? VSCodeDarkTheme : VSCodeLightTheme;
  monaco.editor.defineTheme('giter-compare', {
    base: theme === 'dark' ? 'vs-dark' : 'vs',
    inherit: true,
    rules: [],
    colors: {
      'editor.background': theme === 'dark' ? '#1e1e1e' : '#ffffff',
      'editor.foreground': theme === 'dark' ? '#d4d4d4' : '#000000',
    }
  });

  // 连接 TextMate grammars
  const grammars = new Map<string, string>();
  grammars.set('vue', 'source.vue');

  // 添加插件语言
  const plugins = pluginManager.getPlugins();
  for (const plugin of plugins) {
    if (!plugin.enabled) continue;
    for (const lang of plugin.languages) {
      if (!grammars.has(lang)) {
        grammars.set(lang, `source.${lang}`);
      }
    }
  }

  await wireTmGrammars(monaco, registry, grammars);

  grammarsLoaded = true;
  return registry;
}

export function getLanguageFromFilename(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase();
  
  const languageMap: Record<string, string> = {
    'vue': 'vue',
    'js': 'javascript',
    'ts': 'typescript',
    'jsx': 'javascript',
    'tsx': 'typescript',
    'json': 'json',
    'html': 'html',
    'htm': 'html',
    'css': 'css',
    'scss': 'scss',
    'sass': 'sass',
    'less': 'less',
    'py': 'python',
    'java': 'java',
    'cpp': 'cpp',
    'cc': 'cpp',
    'c': 'c',
    'h': 'cpp',
    'hpp': 'cpp',
    'go': 'go',
    'rs': 'rust',
    'php': 'php',
    'rb': 'ruby',
    'swift': 'swift',
    'kt': 'kotlin',
    'kts': 'kotlin',
    'md': 'markdown',
    'yml': 'yaml',
    'yaml': 'yaml',
    'xml': 'xml',
    'sh': 'shell',
    'bash': 'shell',
    'zsh': 'shell',
    'sql': 'sql',
    'dockerfile': 'dockerfile',
    'vue': 'vue',
  };
  
  return languageMap[ext || ''] || 'plaintext';
}

export function setMonacoTheme(theme: 'light' | 'dark') {
  const selectedTheme = theme === 'dark' ? VSCodeDarkTheme : VSCodeLightTheme;
  monaco.editor.defineTheme('giter-compare', {
    base: theme === 'dark' ? 'vs-dark' : 'vs',
    inherit: true,
    rules: [],
    colors: {
      'editor.background': theme === 'dark' ? '#1e1e1e' : '#ffffff',
      'editor.foreground': theme === 'dark' ? '#d4d4d4' : '#000000',
    }
  });
  monaco.editor.setTheme('giter-compare');
}
