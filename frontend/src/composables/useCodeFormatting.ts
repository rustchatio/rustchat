import type { Ref } from 'vue'
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
