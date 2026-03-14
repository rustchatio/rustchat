// ============================================
// Markdown Rendering Utilities
// ============================================

import { marked } from 'marked';
import DOMPurify from 'dompurify';
import hljs from 'highlight.js';
import 'highlight.js/styles/github-dark.css';

// ============================================
// Helper Functions
// ============================================

function escapeHtml(text: string): string {
  const div = document.createElement('div');
  div.textContent = text;
  return div.innerHTML;
}

function highlightCode(code: string, lang?: string): string {
  if (lang && hljs.getLanguage(lang)) {
    try {
      return hljs.highlight(code, { language: lang }).value;
    } catch {
      // Fall through to auto-highlight
    }
  }
  try {
    return hljs.highlightAuto(code).value;
  } catch {
    return escapeHtml(code);
  }
}

// ============================================
// Custom Renderer
// ============================================

const renderer = new marked.Renderer();

// Custom link renderer
renderer.link = (link: { href: string; title?: string | null; text: string }) => {
  const safeHref = DOMPurify.sanitize(link.href);
  const safeTitle = link.title ? DOMPurify.sanitize(link.title) : '';
  return `<a href="${safeHref}"${safeTitle ? ` title="${safeTitle}"` : ''} target="_blank" rel="noopener noreferrer" class="text-brand hover:underline">${link.text}</a>`;
};

// Custom code block renderer - use 'text' property as per new API
renderer.code = (code: { text: string; lang?: string; escaped?: boolean }) => {
  const lang = code.lang || '';
  const rawCode = code.text;
  const highlighted = code.escaped ? rawCode : highlightCode(rawCode, lang);
  const languageClass = lang ? ` language-${lang}` : '';
  const displayLang = lang || 'text';
  
  return `<div class="code-block relative group my-2 rounded-lg overflow-hidden bg-[#0d1117]">
    <div class="flex items-center justify-between px-3 py-1.5 bg-[#161b22] border-b border-[#30363d]">
      <span class="text-xs text-text-3 font-mono">${displayLang}</span>
      <button class="copy-btn text-xs text-text-3 hover:text-text-1 opacity-0 group-hover:opacity-100 transition-opacity" data-code="${escapeHtml(rawCode)}">
        Copy
      </button>
    </div>
    <pre class="p-3 overflow-x-auto"><code class="hljs${languageClass}">${highlighted}</code></pre>
  </div>`;
};

// Custom inline code renderer
renderer.codespan = (code: { text: string }) => {
  return `<code class="px-1.5 py-0.5 bg-bg-surface-2 rounded text-sm font-mono text-text-1">${code.text}</code>`;
};

// Custom blockquote renderer
renderer.blockquote = (blockquote: { text: string }) => {
  return `<blockquote class="pl-4 border-l-4 border-brand/50 my-2 text-text-2 italic">${blockquote.text}</blockquote>`;
};

// Custom paragraph renderer
renderer.paragraph = (paragraph: { text: string }) => {
  return `<p class="mb-2 last:mb-0">${paragraph.text}</p>`;
};

// Custom list renderer
renderer.list = (list: { items: unknown[]; ordered: boolean }) => {
  const tag = list.ordered ? 'ol' : 'ul';
  const type = list.ordered ? 'list-decimal' : 'list-disc';
  // items are already rendered strings in the new API
  const items = list.items.map((item: unknown) => `<li class="ml-4">${item}</li>`).join('');
  return `<${tag} class="${type} my-2 pl-2">${items}</${tag}>`;
};

// Custom list item renderer
renderer.listitem = (item: { text: string }) => {
  return item.text;
};

// Custom heading renderer
renderer.heading = (heading: { text: string; depth: number }) => {
  const sizes: Record<number, string> = {
    1: 'text-xl font-bold mb-3',
    2: 'text-lg font-semibold mb-2',
    3: 'text-base font-semibold mb-2',
    4: 'text-sm font-semibold mb-1',
    5: 'text-xs font-semibold mb-1',
    6: 'text-xs font-medium mb-1',
  };
  return `<h${heading.depth} class="${sizes[heading.depth] || sizes[3]}">${heading.text}</h${heading.depth}>`;
};

// Custom horizontal rule renderer
renderer.hr = () => {
  return `<hr class="my-4 border-border-1" />`;
};

// Custom table renderer
renderer.table = (table: { header: unknown[]; rows: unknown[][] }) => {
  // header and rows are already rendered
  const headerStr = table.header.join('');
  const rowsStr = table.rows.map((row: unknown[]) => `<tr>${row.join('')}</tr>`).join('');
  return `<div class="overflow-x-auto my-2">
    <table class="w-full text-sm border border-border-1 rounded-lg">
      <thead><tr>${headerStr}</tr></thead>
      <tbody>${rowsStr}</tbody>
    </table>
  </div>`;
};

// Custom table row renderer
renderer.tablerow = (row: { text: string }) => {
  return `<tr class="border-b border-border-1 last:border-0 hover:bg-bg-surface-2/50">${row.text}</tr>`;
};

// Custom table cell renderer
renderer.tablecell = (cell: { text: string; header: boolean; align: 'center' | 'left' | 'right' | null }) => {
  const tag = cell.header ? 'th' : 'td';
  const align = cell.align ? ` align="${cell.align}"` : '';
  const classes = cell.header
    ? 'px-3 py-2 text-left font-semibold border-b border-border-1 bg-bg-surface-2'
    : 'px-3 py-2';
  return `<${tag} class="${classes}"${align}>${cell.text}</${tag}>`;
};

// Apply custom renderer
marked.use({ renderer });

// ============================================
// Public API
// ============================================

/**
 * Render markdown to sanitized HTML
 */
export function renderMarkdown(content: string): string {
  if (!content) return '';
  
  try {
    const rawHtml = marked.parse(content, { async: false }) as string;
    return DOMPurify.sanitize(rawHtml, {
      ALLOWED_TAGS: [
        'p', 'br', 'strong', 'b', 'em', 'i', 'u', 's', 'strike', 'del',
        'a', 'code', 'pre', 'blockquote', 'ul', 'ol', 'li',
        'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
        'hr', 'table', 'thead', 'tbody', 'tr', 'th', 'td', 'div', 'span'
      ],
      ALLOWED_ATTR: [
        'href', 'title', 'target', 'rel', 'class', 'data-code',
        'align'
      ],
    });
  } catch (error) {
    console.error('Markdown rendering error:', error);
    return escapeHtml(content);
  }
}

/**
 * Extract plain text from markdown
 */
export function extractPlainText(content: string): string {
  if (!content) return '';
  
  // Remove code blocks
  let text = content.replace(/```[\s\S]*?```/g, ' [code] ');
  
  // Remove inline code
  text = text.replace(/`([^`]+)`/g, '$1');
  
  // Remove links but keep text
  text = text.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1');
  text = text.replace(/<([^>]+)>/g, '$1');
  
  // Remove formatting
  text = text.replace(/(\*\*|__|\*|_|~~|`)/g, '');
  
  // Remove headings
  text = text.replace(/^#{1,6}\s+/gm, '');
  
  // Remove blockquotes
  text = text.replace(/^>\s*/gm, '');
  
  // Collapse whitespace
  text = text.replace(/\s+/g, ' ').trim();
  
  return text;
}

/**
 * Detect if content has mentions
 */
export function hasMentions(content: string): boolean {
  return /@\w+/.test(content);
}

/**
 * Get mentioned usernames from content
 */
export function getMentions(content: string): string[] {
  const matches = content.match(/@(\w+)/g);
  return matches ? matches.map(m => m.slice(1)) : [];
}

/**
 * Truncate text to a maximum length
 */
export function truncateText(text: string, maxLength: number): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength - 3) + '...';
}
