// Base entity types

export type EntityId = string & { __brand: 'EntityId' }

export function createEntityId(id: string): EntityId {
  return id as EntityId
}

export interface Entity<T = string> {
  id: T
  createdAt: string
  updatedAt: string
}
