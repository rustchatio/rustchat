import { marked } from 'marked'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'

// Configure marked with highlight.js
marked.setOptions({
  highlight: (code: string, lang: string | undefined) => {
    if (lang && hljs.getLanguage(lang)) {
      try {
        return hljs.highlight(code, { language: lang }).value
      } catch {
        return code
      }
    }
    return hljs.highlightAuto(code).value
  },
  langPrefix: 'hljs language-',
  breaks: true,
  gfm: true
})

export function renderMarkdown(content: string): string {
  if (!content || content.trim().length === 0) {
    return '<p class="text-gray-400 text-xs">Nothing to preview</p>'
  }
  
  return marked(content) as string
}

export function wrapWithInlineCode(text: string): string {
  return `\`${text}\``
}

export function wrapWithCodeBlock(text: string, language: string = ''): string {
  return `\`\`\`${language}\n${text}\n\`\`\``
}

export function detectCodeLanguage(text: string): string | null {
  // Simple language detection based on common patterns
  if (text.includes('function') || text.includes('const') || text.includes('let')) {
    return 'javascript'
  }
  if (text.includes('def ') || text.includes('import ') && text.includes(':')) {
    return 'python'
  }
  if (text.includes('{') && text.includes('}') && text.includes(';')) {
    return 'json'
  }
  return null
}
