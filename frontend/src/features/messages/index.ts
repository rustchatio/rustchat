// Messages Feature - Public API
// Usage: import { useMessageStore, messageService, MessageList } from '@/features/messages'

// Stores
export { useMessageStore } from './stores/messageStore'

// Services
export { messageService } from './services/messageService'
export { threadService, type ThreadResponse, type ThreadQueryParams } from './services/threadService'

// Repositories
export { messageRepository } from './repositories/messageRepository'

// Handlers
export { handleWebSocketEvent as handleMessageWebSocketEvent } from './handlers/messageSocketHandlers'

// Composables (to be created)
// export { useMessages } from './composables/useMessages'
// export { useThread } from './composables/useThread'

// Components (to be created)
// export { default as MessageList } from './components/MessageList.vue'
// export { default as MessageInput } from './components/MessageInput.vue'
// export { default as ThreadPanel } from './components/ThreadPanel.vue'
