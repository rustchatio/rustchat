/**
 * Activity Service - Business logic for activity feed
 */

import { activityRepository } from '../repositories/activityRepository'
import { useActivityStore } from '../stores/activityStore'
import { useAuthStore } from '../../../features/auth/stores/authStore'
import type { Activity } from '../types'
import { ActivityType } from '../types'

class ActivityService {
  private get store() {
    return useActivityStore()
  }

  private get userId(): string | undefined {
    const authStore = useAuthStore()
    return authStore.user?.id
  }

  async loadActivities(refresh = false): Promise<void> {
    const userId = this.userId
    if (!userId) return

    this.store.setLoading(true)

    try {
      const cursor = refresh ? undefined : this.store.cursor ?? undefined
      const filter = this.store.filter

      const response = await activityRepository.getFeed(userId, {
        cursor,
        limit: 50,
        type: filter ?? undefined,
        unreadOnly: false
      })

      if (refresh || !cursor) {
        this.store.setActivities(
          Object.values(response.activities),
          response.order
        )
      } else {
        this.store.appendActivities(
          Object.values(response.activities),
          response.order
        )
      }

      this.store.setUnreadCount(response.unreadCount)
      this.store.setHasMore(!!response.nextCursor)
      this.store.setCursor(response.nextCursor ?? null)
    } finally {
      this.store.setLoading(false)
    }
  }

  async loadMore(): Promise<void> {
    if (!this.store.hasMore || this.store.isLoading) return
    await this.loadActivities()
  }

  async markRead(activityId: string): Promise<void> {
    const userId = this.userId
    if (!userId) return

    // Optimistic update
    this.store.markActivityRead(activityId)

    try {
      await activityRepository.markRead(userId, [activityId])
    } catch (error) {
      // Revert on error
      await this.loadActivities(true)
      throw error
    }
  }

  async markAllRead(): Promise<void> {
    const userId = this.userId
    if (!userId) return

    // Optimistic update
    this.store.markAllActivitiesRead()

    try {
      await activityRepository.markAllRead(userId)
    } catch (error) {
      // Revert on error
      await this.loadActivities(true)
      throw error
    }
  }

  setFilter(type: ActivityType | null): void {
    this.store.setFilter(type)
    this.store.clearActivities()
    this.store.setCursor(null)
    void this.loadActivities(true)
  }

  openFeed(): void {
    this.store.openFeed()
    void this.loadActivities(true)
  }

  closeFeed(): void {
    this.store.closeFeed()
  }

  handleNewActivity(activity: Activity): void {
    this.store.addActivity(activity)
  }
}

export const activityService = new ActivityService()
