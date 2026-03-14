// Team Repository - Data access for teams

import { teamsApi } from '../../../api/teams'
import type { Team, TeamMember, TeamId } from '../../../core/entities/Team'
import type { UserId } from '../../../core/entities/User'
import { withRetry } from '../../../core/services/retry'

export interface CreateTeamRequest {
  name: string
  displayName: string
  description?: string
}

export const teamRepository = {
  // List user's teams
  async list(): Promise<Team[]> {
    return withRetry(async () => {
      const response = await teamsApi.list()
      return response.data.map(normalizeTeam)
    })
  },

  // List public teams available to join
  async listPublic(): Promise<Team[]> {
    return withRetry(async () => {
      const response = await teamsApi.listPublic()
      return response.data.map(normalizeTeam)
    })
  },

  // Get single team
  async getById(teamId: TeamId): Promise<Team | null> {
    return withRetry(async () => {
      try {
        const response = await teamsApi.get(teamId)
        return normalizeTeam(response.data)
      } catch (error: any) {
        if (error?.response?.status === 404) return null
        throw error
      }
    })
  },

  // Create new team
  async create(data: CreateTeamRequest): Promise<Team> {
    return withRetry(async () => {
      const response = await teamsApi.create({
        name: data.name,
        display_name: data.displayName,
        description: data.description
      })
      return normalizeTeam(response.data)
    })
  },

  // Update team
  async update(teamId: TeamId, data: Partial<CreateTeamRequest>): Promise<Team> {
    return withRetry(async () => {
      const response = await teamsApi.update(teamId, {
        name: data.name,
        display_name: data.displayName,
        description: data.description
      })
      return normalizeTeam(response.data)
    })
  },

  // Delete team
  async delete(teamId: TeamId): Promise<void> {
    await withRetry(() => teamsApi.delete(teamId))
  },

  // Join a public team
  async join(teamId: TeamId): Promise<void> {
    await withRetry(() => teamsApi.join(teamId))
  },

  // Leave a team
  async leave(teamId: TeamId): Promise<void> {
    await withRetry(() => teamsApi.leave(teamId))
  },

  // Get team members
  async getMembers(teamId: TeamId): Promise<TeamMember[]> {
    return withRetry(async () => {
      const response = await teamsApi.getMembers(teamId)
      return response.data.map(normalizeTeamMember)
    })
  }
}

function normalizeTeam(raw: any): Team {
  return {
    id: raw.id as TeamId,
    name: raw.name,
    displayName: raw.display_name,
    description: raw.description,
    createdAt: new Date(raw.created_at || Date.now()),
    updatedAt: new Date(raw.updated_at || raw.created_at || Date.now()),
    isArchived: raw.delete_at ? true : false
  }
}

function normalizeTeamMember(raw: any): TeamMember {
  return {
    teamId: raw.team_id as TeamId,
    userId: raw.user_id as UserId,
    roles: raw.roles || [],
    joinedAt: new Date(raw.joined_at || Date.now())
  }
}
