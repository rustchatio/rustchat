import { marked, Renderer } from 'marked'
import hljs from 'highlight.js'
import 'highlight.js/styles/github-dark.css'

// Create a custom renderer with syntax highlighting
const renderer = new Renderer()

// Override the code renderer for marked v12 API
// Note: marked v12 uses positional params: code, infostring, escaped
renderer.code = (code: string, infostring: string | undefined, _escaped: boolean) => {
  const lang = infostring || ''
  if (lang && hljs.getLanguage(lang)) {
    try {
      const highlighted = hljs.highlight(code, { language: lang }).value
      return `<div class="code-block-wrapper"><pre><code class="hljs language-${lang}">${highlighted}</code><button class="copy-button">Copy</button></pre></div>`
    } catch {
      // Fall through to auto-highlight
    }
  }
  const highlighted = hljs.highlightAuto(code).value
  return `<div class="code-block-wrapper"><pre><code class="hljs">${highlighted}</code><button class="copy-button">Copy</button></pre></div>`
}

// Configure marked with the custom renderer
marked.use({
  renderer,
  breaks: true,
  gfm: true
})

export function renderMarkdown(content: string): string {
  if (!content || content.trim().length === 0) {
    return '<p class="text-gray-400 text-xs">Nothing to preview</p>'
  }
  
  return marked.parse(content) as string
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
