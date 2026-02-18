import type { UserId } from './User'

export type TeamId = string

export interface Team {
  id: TeamId
  name: string
  displayName: string
  description?: string
  type: 'open' | 'invite'
  allowOpenInvite: boolean
  companyName?: string
  createdAt: Date
  updatedAt: Date
}

export interface TeamMember {
  teamId: TeamId
  userId: UserId
  username: string
  displayName?: string
  avatarUrl?: string
  role: 'team_admin' | 'member' | 'guest'
  joinedAt: Date
  // Denormalized for display
  presence?: 'online' | 'away' | 'dnd' | 'offline'
}
