import type { UserId } from './User'

export type TeamId = string

export interface Team {
  id: TeamId
  name: string
  displayName: string
  description?: string
  type?: 'open' | 'invite'
  allowOpenInvite?: boolean
  companyName?: string
  createdAt: Date
  updatedAt: Date
  isArchived?: boolean
}

export interface TeamMember {
  teamId: TeamId
  userId: UserId
  roles: string[]
  joinedAt: Date
}
