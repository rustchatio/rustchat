# Composer Enhancement Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Transform the message composer with side-by-side preview, improved code formatting, and 15% smaller fonts.

**Architecture:** Split-pane composer with live markdown preview using highlight.js for syntax highlighting. Enhanced toolbar with text-wrapping shortcuts. 15% font reduction applied consistently across all composer elements.

**Tech Stack:** Vue 3, highlight.js, marked, Tailwind CSS

**Design Document:** `docs/plans/2026-04-06-composer-enhancement-design.md`

---

## Prerequisites

Ensure these packages are available (install if missing):
- highlight.js ^11.9.0
- marked ^12.0.0

---

## Task 1: Install Dependencies

**Files:**
- Modify: `frontend/package.json`

**Step 1: Add dependencies**

Add to `frontend/package.json` dependencies section:

```json
{
  "dependencies": {
    "highlight.js": "^11.9.0",
    "marked": "^12.0.0"
  }
}
```

**Step 2: Install packages**

Run:
```bash
cd frontend && npm install
```

Expected: Packages installed successfully

**Step 3: Commit**

```bash
git add frontend/package.json frontend/package-lock.json
git commit -m "deps: add highlight.js and marked for composer enhancement"
```

---

## Task 2: Create Markdown Preview Service

**Files:**
- Create: `frontend/src/services/markdownService.ts`

**Step 1: Create markdown service with syntax highlighting**

```typescript
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
```

**Step 2: Verify service compiles**

Run:
```bash
cd frontend && npm run build 2>&1 | grep -i "error" | head -10
```

Expected: No errors related to markdownService.ts

**Step 3: Commit**

```bash
git add frontend/src/services/markdownService.ts
git commit -m "feat: add markdown service with syntax highlighting"
```

---

## Task 3: Update MarkdownPreview Component

**Files:**
- Modify: `frontend/src/components/composer/MarkdownPreview.vue`

**Step 1: Replace entire component**

```vue
<script setup lang="ts">
import { computed } from 'vue'
import { renderMarkdown } from '../../services/markdownService'

const props = defineProps<{
  content: string
}>()

const renderedHtml = computed(() => {
  return renderMarkdown(props.content)
})

function copyCode(event: MouseEvent) {
  const target = event.target as HTMLElement
  const codeBlock = target.closest('.code-block-wrapper')?.querySelector('code')
  if (codeBlock) {
    navigator.clipboard.writeText(codeBlock.textContent || '')
    target.textContent = 'Copied!'
    setTimeout(() => {
      target.textContent = 'Copy'
    }, 2000)
  }
}
</script>

<template>
  <div 
    class="markdown-preview prose prose-xs max-w-none p-3 bg-bg-surface-2/50 rounded-lg border border-border-1 overflow-auto max-h-80"
    v-html="renderedHtml"
    @click="copyCode"
  ></div>
</template>

<style scoped>
.markdown-preview :deep(pre) {
  position: relative;
  background: #1e1e1e;
  border-radius: 0.375rem;
  padding: 0.75rem;
  overflow-x: auto;
  margin: 0.5rem 0;
}

.markdown-preview :deep(pre code) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 0.75rem;
  line-height: 1.5;
}

.markdown-preview :deep(code:not(pre code)) {
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  background: rgba(0, 0, 0, 0.1);
  padding: 0.125rem 0.375rem;
  border-radius: 0.25rem;
  font-size: 0.75rem;
}

.markdown-preview :deep(.code-block-wrapper) {
  position: relative;
}

.markdown-preview :deep(.copy-button) {
  position: absolute;
  top: 0.5rem;
  right: 0.5rem;
  padding: 0.25rem 0.5rem;
  background: rgba(255, 255, 255, 0.1);
  border: none;
  border-radius: 0.25rem;
  color: #fff;
  font-size: 0.625rem;
  cursor: pointer;
  opacity: 0;
  transition: opacity 0.2s;
}

.markdown-preview :deep(.code-block-wrapper:hover .copy-button) {
  opacity: 1;
}

.markdown-preview :deep(p) {
  margin: 0.5rem 0;
  font-size: 0.75rem;
  line-height: 1.5;
}

.markdown-preview :deep(strong) {
  font-weight: 700;
}

.markdown-preview :deep(em) {
  font-style: italic;
}

.markdown-preview :deep(blockquote) {
  border-left: 3px solid var(--border-color, #ccc);
  padding-left: 0.75rem;
  margin: 0.5rem 0;
  color: var(--text-muted, #666);
  font-size: 0.75rem;
}

.markdown-preview :deep(ul), .markdown-preview :deep(ol) {
  margin: 0.5rem 0;
  padding-left: 1.25rem;
  font-size: 0.75rem;
}

.markdown-preview :deep(a) {
  color: var(--brand-color, #0066cc);
  text-decoration: underline;
}

.markdown-preview :deep(hr) {
  margin: 1rem 0;
  border: none;
  border-top: 1px solid var(--border-color, #ccc);
}
</style>
```

**Step 2: Test component builds**

Run:
```bash
cd frontend && npm run build 2>&1 | tail -20
```

Expected: Build succeeds with no errors

**Step 3: Commit**

```bash
git add frontend/src/components/composer/MarkdownPreview.vue
git commit -m "feat: enhance markdown preview with syntax highlighting and smaller fonts"
```

---

## Task 4: Create useCodeFormatting Composable

**Files:**
- Create: `frontend/src/composables/useCodeFormatting.ts`

**Step 1: Create composable**

```typescript
import { ref } from 'vue'
import { wrapWithInlineCode, wrapWithCodeBlock } from '../services/markdownService'

export function useCodeFormatting(textareaRef: Ref<HTMLTextAreaElement | null>) {
  const lastSelection = ref<{ start: number; end: number; text: string } | null>(null)

  function saveSelection() {
    const textarea = textareaRef.value
    if (!textarea) return
    
    lastSelection.value = {
      start: textarea.selectionStart,
      end: textarea.selectionEnd,
      text: textarea.value.substring(textarea.selectionStart, textarea.selectionEnd)
    }
  }

  function wrapSelection(
    content: Ref<string>,
    wrapper: (text: string) => string
  ): boolean {
    const textarea = textareaRef.value
    if (!textarea) return false

    const start = textarea.selectionStart
    const end = textarea.selectionEnd
    const selectedText = content.value.substring(start, end)

    if (selectedText) {
      const wrapped = wrapper(selectedText)
      content.value = content.value.substring(0, start) + wrapped + content.value.substring(end)
      
      // Restore cursor position after the wrapped text
      setTimeout(() => {
        textarea.focus()
        const newPosition = start + wrapped.length
        textarea.setSelectionRange(newPosition, newPosition)
      }, 0)
      
      return true
    }
    
    return false
  }

  function formatInlineCode(content: Ref<string>): boolean {
    return wrapSelection(content, wrapWithInlineCode)
  }

  function formatCodeBlock(content: Ref<string>): boolean {
    return wrapSelection(content, (text) => wrapWithCodeBlock(text, ''))
  }

  function handleKeydown(event: KeyboardEvent, content: Ref<string>): boolean {
    // Handle ` key for inline code
    if (event.key === '`' && !event.ctrlKey && !event.metaKey && !event.altKey) {
      const textarea = textareaRef.value
      if (!textarea) return false

      const start = textarea.selectionStart
      const end = textarea.selectionEnd
      
      if (start !== end) {
        event.preventDefault()
        // Check if triple backtick for code block
        const isTriple = event.repeat || false
        if (isTriple) {
          return formatCodeBlock(content)
        }
        return formatInlineCode(content)
      }
    }
    
    return false
  }

  return {
    saveSelection,
    formatInlineCode,
    formatCodeBlock,
    handleKeydown,
    lastSelection
  }
}
```

**Step 2: Verify composable compiles**

Run:
```bash
cd frontend && npm run build 2>&1 | grep -i "error" | head -10
```

Expected: No errors

**Step 3: Commit**

```bash
git add frontend/src/composables/useCodeFormatting.ts
git commit -m "feat: add useCodeFormatting composable for text wrapping"
```

---

## Task 5: Update FormattingToolbar Component

**Files:**
- Modify: `frontend/src/components/composer/FormattingToolbar.vue`

**Step 1: Add emit for format actions**

Modify the script setup section to add format action handling:

```typescript
const emit = defineEmits<{
  (e: 'format', type: string): void
  (e: 'togglePreview'): void
}>()
```

Replace the existing formatActions array buttons with smaller versions:

Update the button class from:
```
class="flex h-11 w-11 shrink-0 items-center justify-center rounded-r-1 text-text-3 transition-standard hover:bg-bg-surface-1 hover:text-text-1 focus-ring"
```

To:
```
class="flex h-8 w-8 shrink-0 items-center justify-center rounded-r-1 text-text-3 transition-standard hover:bg-bg-surface-1 hover:text-text-1 focus-ring"
```

Update icon size from `class="w-4 h-4"` to `class="w-3.5 h-3.5"`

Update the divider from `class="mx-1 h-6 w-px"` to `class="mx-1 h-5 w-px"`

Update help text font sizes from `text-xs` to `text-[11px]`

**Step 2: Full updated component**

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { Bold, Italic, Strikethrough, Heading, Code, Link2, List, ListOrdered, Quote, Eye, EyeOff, HelpCircle } from 'lucide-vue-next'

const emit = defineEmits<{
  (e: 'format', type: string): void
  (e: 'togglePreview'): void
}>()

defineProps<{
  showPreview: boolean
}>()

const formatActions = [
  { icon: Bold, type: 'bold', title: 'Bold (Ctrl+B)', label: 'Bold' },
  { icon: Italic, type: 'italic', title: 'Italic (Ctrl+I)', label: 'Italic' },
  { icon: Strikethrough, type: 'strike', title: 'Strikethrough (Ctrl+Shift+X)', label: 'Strikethrough' },
  { icon: Heading, type: 'heading', title: 'Heading', label: 'Heading' },
  { icon: Code, type: 'code', title: 'Inline code', label: 'Inline code' },
  { icon: Code, type: 'codeblock', title: 'Code block', label: 'Code block' },
  { icon: Link2, type: 'link', title: 'Link', label: 'Link' },
  { icon: Quote, type: 'quote', title: 'Quote', label: 'Quote' },
  { icon: List, type: 'bullet', title: 'Bullet list', label: 'Bullet list' },
  { icon: ListOrdered, type: 'numbered', title: 'Numbered list', label: 'Numbered list' },
]

const showHelp = ref(false)
const rootRef = ref<HTMLElement | null>(null)

function handleDocumentClick(event: MouseEvent) {
  const target = event.target as Node | null
  if (!target) return
  if (rootRef.value?.contains(target)) return
  showHelp.value = false
}

onMounted(() => {
  document.addEventListener('mousedown', handleDocumentClick)
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleDocumentClick)
})
</script>

<template>
  <div ref="rootRef" class="relative flex items-center gap-1 overflow-x-auto border-b border-border-1 bg-bg-surface-2/50 px-2 py-1 whitespace-nowrap">
    <!-- Formatting buttons -->
    <button
      v-for="action in formatActions"
      :key="action.type"
      @click="$emit('format', action.type)"
      :title="action.title"
      :aria-label="action.label"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-r-1 text-text-3 transition-standard hover:bg-bg-surface-1 hover:text-text-1 focus-ring"
    >
      <component :is="action.icon" class="w-3.5 h-3.5" />
    </button>
    
    <!-- Divider -->
    <div class="mx-1 h-5 w-px shrink-0 bg-border-1"></div>
    
    <!-- Preview toggle -->
    <button
      @click="$emit('togglePreview')"
      :title="showPreview ? 'Hide preview' : 'Show preview'"
      aria-label="Toggle markdown preview"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-r-1 transition-standard focus-ring"
      :class="showPreview 
        ? 'bg-brand/10 text-brand'
        : 'text-text-3 hover:bg-bg-surface-1 hover:text-text-1'"
    >
      <component :is="showPreview ? EyeOff : Eye" class="w-3.5 h-3.5" />
    </button>

    <button
      @click="showHelp = !showHelp"
      title="Formatting help"
      aria-label="Formatting help"
      class="flex h-8 w-8 shrink-0 items-center justify-center rounded-r-1 text-text-3 transition-standard hover:bg-bg-surface-1 hover:text-text-1 focus-ring"
      :class="showHelp ? 'bg-bg-surface-1 text-text-1' : ''"
    >
      <HelpCircle class="w-3.5 h-3.5" />
    </button>

    <div
      v-if="showHelp"
      class="absolute right-1 top-full mt-2 z-[130] w-[22rem] rounded-r-2 border border-border-1 bg-bg-surface-1 p-3 shadow-2xl"
    >
      <p class="text-[11px] font-semibold text-text-1">Formatting shortcuts</p>
      <div class="mt-2 space-y-1 text-[11px] text-text-2">
        <p><kbd class="rounded bg-bg-surface-2 px-1">Ctrl/Cmd+B</kbd> Bold</p>
        <p><kbd class="rounded bg-bg-surface-2 px-1">Ctrl/Cmd+I</kbd> Italic</p>
        <p><kbd class="rounded bg-bg-surface-2 px-1">Toolbar</kbd> Insert link</p>
        <p><kbd class="rounded bg-bg-surface-2 px-1">Ctrl/Cmd+Shift+X</kbd> Strikethrough</p>
        <p><kbd class="rounded bg-bg-surface-2 px-1">Ctrl/Cmd+Shift+7</kbd> Numbered list</p>
        <p><kbd class="rounded bg-bg-surface-2 px-1">Ctrl/Cmd+Shift+8</kbd> Bulleted list</p>
      </div>
      <p class="mt-2 text-[10px] text-text-3">Use <code>:emoji:</code>, <code>@mention</code>, <code>~channel</code>, and <code>^k</code> (or Ctrl/Cmd+K) for command menu.</p>
    </div>
  </div>
</template>
```

**Step 2: Verify build**

Run:
```bash
cd frontend && npm run build 2>&1 | tail -10
```

**Step 3: Commit**

```bash
git add frontend/src/components/composer/FormattingToolbar.vue
git commit -m "feat: reduce formatting toolbar size by ~15%"
```

---

## Task 6: Update MessageComposer Component

**Files:**
- Modify: `frontend/src/components/composer/MessageComposer.vue`

**Step 1: Add imports and composable**

Add to imports section:
```typescript
import { useCodeFormatting } from '../../composables/useCodeFormatting'
```

**Step 2: Initialize composable**

In setup, add:
```typescript
const { formatInlineCode, formatCodeBlock } = useCodeFormatting(textareaRef)
```

**Step 3: Add format handler**

Add function:
```typescript
function handleFormat(type: string) {
  switch (type) {
    case 'code':
      formatInlineCode(content)
      break
    case 'codeblock':
      formatCodeBlock(content)
      break
    // Other format cases can be added later
  }
}
```

**Step 4: Update textarea styling**

Change line ~873 from:
```
class="max-h-80 min-h-[44px] w-full resize-none border-0 bg-transparent px-3 py-2.5 text-sm leading-relaxed text-text-1 placeholder:text-text-3 focus:ring-0"
```

To:
```
class="max-h-80 min-h-[40px] w-full resize-none border-0 bg-transparent px-3 py-2 text-xs leading-relaxed text-text-1 placeholder:text-text-3 focus:ring-0"
```

**Step 5: Update bottom toolbar button sizes**

Find the Paperclip and Smile buttons (lines ~886-904) and change:
- `class="flex h-11 w-11...` → `class="flex h-9 w-9...`
- `class="w-4 h-4"` → `class="w-3.5 h-3.5"`

**Step 6: Update Send button**

Find the Send button (around line ~975) and change:
- `class="... h-11 ..."` → `class="... h-9 ..."`
- `class="... text-sm ..."` → `class="... text-xs ..."`

**Step 7: Update FormattingToolbar binding**

Update the FormattingToolbar usage to bind format event:
```vue
<FormattingToolbar 
  :showPreview="showPreview"
  @togglePreview="showPreview = !showPreview"
  @format="handleFormat"
/>
```

**Step 8: Update preview section styling**

Find the preview section (lines ~806-809) and ensure it has proper layout classes.

**Step 9: Verify build**

Run:
```bash
cd frontend && npm run build 2>&1 | tail -10
```

**Step 10: Commit**

```bash
git add frontend/src/components/composer/MessageComposer.vue
git commit -m "feat: integrate code formatting and reduce composer font sizes"
```

---

## Task 7: Implement Side-by-Side Layout

**Files:**
- Modify: `frontend/src/components/composer/MessageComposer.vue`

**Step 1: Update template structure for side-by-side layout**

Wrap textarea and preview in a flex container:

```vue
<!-- Editor Container -->
<div class="flex gap-3">
  <!-- Textarea -->
  <div class="flex-1">
    <textarea
      ref="textareaRef"
      v-model="content"
      rows="1"
      class="max-h-80 min-h-[40px] w-full resize-none border-0 bg-transparent px-3 py-2 text-xs leading-relaxed text-text-1 placeholder:text-text-3 focus:ring-0"
      :placeholder="placeholderText"
      aria-label="Message composer"
      @keydown="handleKeydown"
      @input="handleInput"
      @blur="handleTextareaBlur"
    ></textarea>
  </div>
  
  <!-- Preview Panel -->
  <div v-if="showPreview" class="flex-1 border-l border-border-1">
    <MarkdownPreview :content="content" />
  </div>
</div>
```

**Step 2: Add localStorage persistence for preview toggle**

Add to setup:
```typescript
import { useStorage } from '@vueuse/core'

const showPreview = useStorage('composer-show-preview', false)
```

Remove: `const showPreview = ref(false)`

**Step 3: Verify build**

Run:
```bash
cd frontend && npm run build 2>&1 | tail -10
```

**Step 4: Commit**

```bash
git add frontend/src/components/composer/MessageComposer.vue
git commit -m "feat: implement side-by-side preview layout with persistence"
```

---

## Task 8: Update ChannelView Composer Usage

**Files:**
- Verify: `frontend/src/views/main/ChannelView.vue`

**Step 1: Ensure ChannelView passes proper props**

The ChannelView already binds correctly:
```vue
<MessageComposer 
  @send="onSendMessage" 
  @typing="onTyping" 
  @stopTyping="onStopTyping"
  @startAudioCall="onStartAudioCall"
/>
```

No changes needed to ChannelView.vue for this feature.

---

## Task 9: Add Highlight.js Styles

**Files:**
- Modify: `frontend/src/main.ts` or `frontend/src/style.css`

**Step 1: Import highlight.js theme**

Add to `frontend/src/main.ts`:
```typescript
import 'highlight.js/styles/github-dark.css'
```

**Step 2: Commit**

```bash
git add frontend/src/main.ts
git commit -m "feat: import highlight.js dark theme"
```

---

## Task 10: Final Verification

**Files:**
- All modified files

**Step 1: Full build test**

Run:
```bash
cd frontend && npm run build 2>&1 | tail -20
```

Expected: Clean build with no errors

**Step 2: Visual verification checklist**

Manually verify in browser:
- [ ] Composer fonts are ~15% smaller
- [ ] Toolbar buttons are smaller
- [ ] Preview toggle works
- [ ] Preview persists after refresh
- [ ] Side-by-side layout displays correctly
- [ ] Code formatting toolbar buttons work
- [ ] Syntax highlighting appears in preview
- [ ] Copy button appears on code blocks in preview
- [ ] Inline code styling looks good in preview

**Step 3: Commit any final fixes**

```bash
git add .
git commit -m "fix: final composer enhancement adjustments"
```

---

## Summary

This implementation plan adds:
1. ✅ highlight.js and marked for syntax highlighting and markdown parsing
2. ✅ Side-by-side preview layout with localStorage persistence
3. ✅ Code formatting toolbar buttons (inline code, code block)
4. ✅ 15% font size reduction across all composer elements
5. ✅ Enhanced preview with copy-to-clipboard on code blocks
6. ✅ Improved markdown rendering with proper styling

**Estimated time:** 2-3 hours  
**Risk level:** Medium (touches core composer component)  
**Testing:** Manual visual verification required
