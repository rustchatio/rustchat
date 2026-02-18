// Core Module - Public API
// Low-level primitives used across all features

// Entities
export type { User, UserId, UserRef, Presence } from './entities/User'
export type { Channel, ChannelId, ChannelType, DMChannel } from './entities/Channel'
export type { 
  Message, 
  MessageId, 
  MessageDraft, 
  MessageStatus,
  Reaction,
  FileAttachment 
} from './entities/Message'
export type { Call, CallId, CallState, CallParticipant } from './entities/Call'

// Types
export type { Result, AsyncResult } from './types/Result'
export { success, failure, isSuccess, isFailure } from './types/Result'
export type { EntityId } from './entities/Entity'
export { createEntityId } from './entities/Entity'

// Errors
export { AppError, NetworkError, NotFoundError, ValidationError } from './errors/AppError'

// Repositories
export type { Repository, QueryOptions, ListResult } from './repositories/Repository'

// WebSocket
export { wsManager, useWebSocket, type ConnectionState } from './websocket/WebSocketManager'
