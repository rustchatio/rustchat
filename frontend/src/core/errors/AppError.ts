// Base error class for application

export type ErrorCode = 
  | 'NETWORK_ERROR'
  | 'AUTH_ERROR'
  | 'NOT_FOUND'
  | 'VALIDATION_ERROR'
  | 'RATE_LIMITED'
  | 'SERVER_ERROR'
  | 'OFFLINE'
  | 'UNKNOWN'

export class AppError extends Error {
  readonly code: ErrorCode
  readonly statusCode?: number
  readonly isRetryable: boolean
  readonly context?: Record<string, any>

  constructor(
    message: string,
    code: ErrorCode = 'UNKNOWN',
    options?: {
      statusCode?: number
      isRetryable?: boolean
      cause?: Error
      context?: Record<string, any>
    }
  ) {
    super(message, { cause: options?.cause })
    this.name = 'AppError'
    this.code = code
    this.statusCode = options?.statusCode
    this.isRetryable = options?.isRetryable ?? this.defaultRetryable(code)
    this.context = options?.context
  }

  private defaultRetryable(code: ErrorCode): boolean {
    switch (code) {
      case 'NETWORK_ERROR':
      case 'RATE_LIMITED':
      case 'SERVER_ERROR':
      case 'OFFLINE':
        return true
      default:
        return false
    }
  }

  static network(cause?: Error): AppError {
    return new AppError('Network connection failed', 'NETWORK_ERROR', {
      isRetryable: true,
      cause
    })
  }

  static notFound(resource: string): AppError {
    return new AppError(`${resource} not found`, 'NOT_FOUND', {
      isRetryable: false,
      context: { resource }
    })
  }

  static auth(message = 'Authentication required'): AppError {
    return new AppError(message, 'AUTH_ERROR', { isRetryable: false })
  }

  static offline(): AppError {
    return new AppError('You are offline', 'OFFLINE', { isRetryable: true })
  }
}

// Helper to wrap unknown errors
export function toAppError(error: unknown): AppError {
  if (error instanceof AppError) {
    return error
  }
  
  if (error instanceof Error) {
    return new AppError(error.message, 'UNKNOWN', { cause: error })
  }
  
  return new AppError(String(error), 'UNKNOWN')
}
