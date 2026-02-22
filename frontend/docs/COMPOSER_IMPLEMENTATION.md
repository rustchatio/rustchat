# Mattermost-like Composer Implementation

## Overview
This document describes the implementation of a Mattermost-compatible message composer for Rustchat Web UI.

## Features Implemented

### 1. Markdown Transforms (`lib/markdownTransforms.ts`)
Pure functions for text transformations:
- **Bold**: `**text**` (Ctrl+B)
- **Italic**: `*text*` (Ctrl+I)
- **Strikethrough**: `~~text~~` (Ctrl+Shift+X)
- **Heading**: `#### text`
- **Link**: `[text](url)` (Ctrl+K)
- **Inline Code**: `` `text` ``
- **Code Block**: ```language\ncode\n```
- **Quote**: `> text` (Alt+Q)
- **Bulleted List**: `- item` (Ctrl+Shift+8)
- **Numbered List**: `1. item` (Ctrl+Shift+7)

All transforms preserve cursor position and handle selection correctly.

### 2. Draft Persistence (`hooks/useDraft.ts`)
- Per-channel draft storage in localStorage
- Automatic cleanup of drafts older than 7 days
- Restores draft when revisiting channel
- Clears draft on successful send

### 3. Keyboard Shortcuts (`hooks/useKeybindings.ts`)
Mattermost-compatible shortcuts:
- Send: Enter (Shift+Enter for newline)
- Bold: Ctrl+B
- Italic: Ctrl+I
- Strikethrough: Ctrl+Shift+X
- Link: Ctrl+K
- Bulleted List: Ctrl+Shift+8
- Numbered List: Ctrl+Shift+7

### 4. Autocomplete
Three autocomplete providers:
- **@mentions**: User mentions from team members
- **:emoji:**: Emoji names with search (200+ emojis)
- **~channel**: Channel references

Keyboard navigation:
- ↑/↓: Navigate suggestions
- Enter/Tab: Select
- Escape: Close

### 5. File Attachments
- Drag & drop support
- File picker (multiple files)
- Paste images from clipboard
- Upload progress indicator
- Remove attachments before sending

### 6. UI Features
- Auto-resizing textarea (max 400px)
- Toggle formatting toolbar
- Font size toggle (small/normal/large)
- Emoji picker
- Send button with dropdown options
- Keyboard hints footer

## File Structure
```
frontend/src/components/composer/
├── MattermostComposer.vue       # Main composer component
├── lib/
│   ├── markdownTransforms.ts    # Pure markdown functions
│   └── __tests__/
│       └── markdownTransforms.test.ts
├── hooks/
│   ├── useDraft.ts              # Draft persistence
│   ├── useKeybindings.ts        # Keyboard shortcuts
│   └── __tests__/
│       └── useDraft.test.ts
├── autocomplete/
│   ├── EmojiAutocomplete.vue    # :emoji: autocomplete
│   └── ChannelAutocomplete.vue  # ~channel autocomplete
└── __tests__/
    └── MattermostComposer.integration.test.ts
```

## Mattermost Mobile Compatibility

### API Compatibility
- Uses existing `filesApi.upload()` for attachments
- Uses existing WebSocket `sendMessage()` for posts
- Payload format: `{ channel_id, message, file_ids, root_id }`

### WebSocket Events
- Typing indicators via `sendTyping()`
- Message delivery via existing WebSocket handlers
- Compatible with Mattermost Mobile expectations

### Markdown Compatibility
All markdown syntax is Mattermost-compatible:
- Bold/Italic/Strikethrough
- Code blocks with language
- Links
- Lists (bulleted and numbered)
- Quotes

## Testing

### Unit Tests
```bash
npm test -- src/components/composer/lib/__tests__/markdownTransforms.test.ts
```

### Integration Tests
```bash
npm test -- src/components/composer/__tests__/MattermostComposer.integration.test.ts
```

### E2E Tests
```bash
npx playwright test e2e/composer.spec.ts
```

## Usage

### Basic Usage
```vue
<template>
  <MattermostComposer
    :channel-id="currentChannel.id"
    :channel-name="currentChannel.name"
    @send="handleSend"
    @typing="handleTyping"
  />
</template>

<script setup>
import MattermostComposer from './components/composer/MattermostComposer.vue'

function handleSend({ content, fileIds }) {
  // Send via WebSocket or API
}

function handleTyping() {
  // Send typing indicator
}
</script>
```

### With Custom Placeholder
```vue
<MattermostComposer
  :channel-id="channelId"
  :channel-name="channelName"
  placeholder="Write to @username"
/>
```

### Disabled State
```vue
<MattermostComposer
  :channel-id="channelId"
  :channel-name="channelName"
  :disabled="isReadOnly"
/>
```

## Accessibility
- ARIA labels on all buttons
- Keyboard-only navigation
- Screen reader friendly
- High contrast focus states

## Performance Considerations
- Auto-resize uses requestAnimationFrame
- Autocomplete debounced
- Draft saves throttled
- Virtual scrolling for large emoji list

## Future Enhancements
- [ ] Slash command autocomplete
- [ ] Giphy integration
- [ ] Custom emoji support
- [ ] Message scheduling
- [ ] Voice messages
- [ ] Drawing/whiteboard
