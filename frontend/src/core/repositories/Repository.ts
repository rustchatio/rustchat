// Repository pattern - abstraction over data access
// This allows us to swap implementations (API, cache, mock) without changing business logic

import type { AppError } from '../errors/AppError'

export interface QueryOptions {
  limit?: number
  offset?: number
  cursor?: string
  sortBy?: string
  sortOrder?: 'asc' | 'desc'
}

export interface PaginatedResult<T> {
  items: T[]
  hasMore: boolean
  nextCursor?: string
  total?: number
}

export interface Repository<T, ID> {
  // Read operations
  findById(id: ID): Promise<T | null>
  findMany(options?: QueryOptions): Promise<PaginatedResult<T>>
  
  // Write operations
  create(entity: Omit<T, 'id'>): Promise<T>
  update(id: ID, changes: Partial<T>): Promise<T>
  delete(id: ID): Promise<void>
}

// For read-only repositories
export interface ReadRepository<T, ID> {
  findById(id: ID): Promise<T | null>
  findMany(options?: QueryOptions): Promise<PaginatedResult<T>>
}

// Cache decorator for repositories
export function withCache<T, ID>(
  repo: Repository<T, ID>,
  options: { ttlMs?: number; keyFn?: (id: ID) => string } = {}
): Repository<T, ID> {
  const cache = new Map<string, { data: any; expires: number }>()
  const ttlMs = options.ttlMs ?? 60000 // 1 minute default
  
  const getCacheKey = options.keyFn ?? ((id: ID) => String(id))
  
  return {
    async findById(id: ID) {
      const key = getCacheKey(id)
      const cached = cache.get(key)
      
      if (cached && cached.expires > Date.now()) {
        return cached.data as T
      }
      
      const result = await repo.findById(id)
      if (result) {
        cache.set(key, { data: result, expires: Date.now() + ttlMs })
      }
      return result
    },
    
    async findMany(options) {
      // Don't cache list queries by default
      return repo.findMany(options)
    },
    
    async create(entity) {
      const result = await repo.create(entity)
      cache.set(getCacheKey((result as any).id), { 
        data: result, 
        expires: Date.now() + ttlMs 
      })
      return result
    },
    
    async update(id, changes) {
      const result = await repo.update(id, changes)
      cache.set(getCacheKey(id), { data: result, expires: Date.now() + ttlMs })
      return result
    },
    
    async delete(id) {
      await repo.delete(id)
      cache.delete(getCacheKey(id))
    }
  }
}
