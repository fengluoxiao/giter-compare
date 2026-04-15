import { createHighlighterCore, type HighlighterCore } from 'shiki/core'
import { createOnigurumaEngine } from 'shiki/engine/oniguruma'

// 缓存 highlighter 实例
let highlighter: HighlighterCore | null = null
let initPromise: Promise<HighlighterCore> | null = null

// 语言映射
const langMap: Record<string, string> = {
  'vue': 'vue',
  'js': 'javascript',
  'ts': 'typescript',
  'jsx': 'jsx',
  'tsx': 'tsx',
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
  'h': 'c',
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
  'sh': 'bash',
  'bash': 'bash',
  'zsh': 'bash',
  'sql': 'sql',
  'dockerfile': 'dockerfile',
}

export function getLanguageFromFilename(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase() || ''
  return langMap[ext] || 'text'
}

export async function initHighlighter(): Promise<HighlighterCore> {
  if (highlighter) {
    return highlighter
  }

  if (initPromise) {
    return initPromise
  }

  initPromise = createHighlighterCore({
    themes: [
      import('shiki/themes/github-light.mjs'),
      import('shiki/themes/github-dark.mjs'),
    ],
    langs: [
      import('shiki/langs/vue.mjs'),
      import('shiki/langs/javascript.mjs'),
      import('shiki/langs/typescript.mjs'),
      import('shiki/langs/jsx.mjs'),
      import('shiki/langs/tsx.mjs'),
      import('shiki/langs/json.mjs'),
      import('shiki/langs/html.mjs'),
      import('shiki/langs/css.mjs'),
      import('shiki/langs/scss.mjs'),
      import('shiki/langs/python.mjs'),
      import('shiki/langs/java.mjs'),
      import('shiki/langs/cpp.mjs'),
      import('shiki/langs/c.mjs'),
      import('shiki/langs/go.mjs'),
      import('shiki/langs/rust.mjs'),
      import('shiki/langs/php.mjs'),
      import('shiki/langs/ruby.mjs'),
      import('shiki/langs/swift.mjs'),
      import('shiki/langs/kotlin.mjs'),
      import('shiki/langs/markdown.mjs'),
      import('shiki/langs/yaml.mjs'),
      import('shiki/langs/xml.mjs'),
      import('shiki/langs/bash.mjs'),
      import('shiki/langs/sql.mjs'),
      import('shiki/langs/dockerfile.mjs'),
    ],
    engine: createOnigurumaEngine(() => import('shiki/wasm')),
  })

  highlighter = await initPromise
  return highlighter
}

export async function highlightCode(
  code: string,
  filename: string,
  theme: 'light' | 'dark' = 'light'
): Promise<string> {
  const highlighter = await initHighlighter()
  const lang = getLanguageFromFilename(filename)
  const themeName = theme === 'dark' ? 'github-dark' : 'github-light'

  return highlighter.codeToHtml(code, {
    lang,
    theme: themeName,
  })
}

export async function highlightCodeToTokens(
  code: string,
  filename: string,
  theme: 'light' | 'dark' = 'light'
) {
  const highlighter = await initHighlighter()
  const lang = getLanguageFromFilename(filename)
  const themeName = theme === 'dark' ? 'github-dark' : 'github-light'

  return highlighter.codeToTokens(code, {
    lang,
    theme: themeName,
  })
}
