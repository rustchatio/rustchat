# Composer Enhancement Design

**Date:** 2026-04-06  
**Topic:** Enhanced composer with side-by-side preview and improved code formatting

## Overview

Transform the message composer into a more usable, Slack-like experience with a focus on **seeing what you're sending**. The preview becomes a first-class feature with syntax highlighting and better code block styling.

## Problems Addressed

1. Users can't easily see how their formatted messages will look before sending
2. Code formatting is difficult to use and understand
3. Composer fonts are too large, wasting screen space
4. No visual feedback when applying markdown formatting

## Solution: Enhanced Side-by-Side Preview

### Key Features

#### 1. Side-by-Side Layout
- Composer textarea on left (60%), Preview on right (40%)
- Both panes scroll independently
- Preview updates live as you type (debounced 300ms)
- Toggle button to collapse/expand preview
- Preference persisted to localStorage

#### 2. Improved Code Handling
- **Toolbar integration:** Click "Inline Code" or "Code Block" → automatically wraps selected text
- **Keyboard shortcuts:**
  - Select text + `` ` `` → wraps as `inline code`
  - Select text + `` ``` `` → wraps as ```code block```
- **Syntax highlighting in preview:** Uses highlight.js supporting 50+ languages
- **Language selector:** Auto-detects or allows manual language selection for code blocks

#### 3. Font Size Reduction (15%)

| Element | Current | New |
|---------|---------|-----|
| Toolbar buttons | 14px | 12px |
| Textarea input | 14px | 12px |
| Placeholder text | 14px | 12px |
| Preview text | 14px | 12px |
| Send button | 14px | 12px |

#### 4. Preview Enhancements
- Syntax-highlighted code blocks with optional line numbers
- Better inline code styling (subtle background, monospace font)
- "Copy code" button on code blocks in preview
- Clear visual distinction between editable and preview content

### Visual Layout

```
┌─────────────────────────────────────────────────────────────┐
│  B I S H ` ```  [👁 Preview active]                         │  ← Toolbar (smaller fonts)
├──────────────────────────┬──────────────────────────────────┤
│  Type your message...    │  [Preview Panel]                 │
│  ```js                   │  ┌────────────────────────┐      │
│  const x = 1;            │  │ const x = 1;           │      │
│  ```                     │  │ // syntax highlighted  │      │
│                          │  └────────────────────────┘      │
│  [=======textarea======] │  [======preview pane=====]       │
├──────────────────────────┴──────────────────────────────────┤
│  [📎] [😊]                                    [Send ⏎]     │  ← Bottom bar
└─────────────────────────────────────────────────────────────┘
```

### Component Changes

#### MessageComposer.vue
- Add side-by-side layout mode
- Integrate highlight.js for preview syntax highlighting
- Add keyboard shortcut handlers for code formatting
- Update all font sizes (reduce by ~15%)

#### MarkdownPreview.vue
- Replace simple regex-based parser with proper markdown parser
- Add highlight.js integration for code blocks
- Add copy-to-clipboard button on code blocks
- Update styling to match reduced font sizes

#### FormattingToolbar.vue
- Add text wrapping functionality for code buttons
- Reduce icon and button sizes
- Maintain existing button layout

#### New: useCodeFormatting composable
- Handle text selection wrapping
- Detect code blocks and suggest languages
- Keyboard shortcut management

### Data Flow

```
User types ──debounced 300ms──► Parse markdown ──► highlight.js ──► Render preview
                                    │
                                    ▼
                              Store in local state
                                    │
                              User clicks Send
                                    │
                                    ▼
                              Collapse preview
                              Emit message event
```

### Dependencies to Add

```json
{
  "highlight.js": "^11.9.0",
  "marked": "^12.0.0"
}
```

### Error Handling

- **Preview parse errors:** Fall back to rendering as plain text
- **Large messages (>10KB):** Disable live preview to maintain performance
- **Failed syntax highlighting:** Show plain code block without highlighting
- **Browser compatibility:** Graceful degradation if highlight.js fails to load

### Accessibility

- Maintain keyboard navigation between panes (Tab/Shift+Tab)
- Screen reader announcements for preview toggle
- Sufficient color contrast for inline code backgrounds

## Success Criteria

1. Users can see formatted output in real-time as they type
2. Code formatting is accessible via toolbar and keyboard shortcuts
3. Font sizes are reduced by approximately 15% throughout composer
4. No performance degradation with large messages
5. Preview accurately reflects how message will appear when sent

## Out of Scope

- Full WYSIWYG/rich text editor replacement
- Custom themes for code highlighting
- Collaborative editing features
- Mobile-specific composer redesign
