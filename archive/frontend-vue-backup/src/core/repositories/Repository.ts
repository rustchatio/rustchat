// Base repository interface defining CRUD operations

import type { Result } from '../types/Result'

export interface QueryOptions {
  page?: number
  perPage?: number
  sortBy?: string
  sortOrder?: 'asc' | 'desc'
}

export interface ListResult<T> {
  items: T[]
  total: number
  hasMore: boolean
}

export interface Repository<T, ID = string> {
  findById(id: ID): Promise<Result<T | null>>
  findAll(options?: QueryOptions): Promise<Result<ListResult<T>>>
  create(data: Omit<T, 'id' | 'createdAt' | 'updatedAt'>): Promise<Result<T>>
  update(id: ID, data: Partial<T>): Promise<Result<T>>
  delete(id: ID): Promise<Result<void>>
}
