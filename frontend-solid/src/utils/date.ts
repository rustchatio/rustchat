// ============================================
// Date Formatting Utilities
// ============================================

import { format, formatDistanceToNow, isToday, isYesterday, differenceInDays } from 'date-fns';

/**
 * Format a message timestamp for display
 * Shows time for today, "Yesterday" for yesterday, or date for older
 */
export function formatMessageTime(timestamp: string | number | Date): string {
  const date = new Date(timestamp);
  
  if (isToday(date)) {
    return format(date, 'h:mm a');
  }
  
  if (isYesterday(date)) {
    return 'Yesterday';
  }
  
  const daysDiff = differenceInDays(new Date(), date);
  if (daysDiff < 7) {
    return format(date, 'EEEE'); // Day name
  }
  
  return format(date, 'MMM d, yyyy');
}

/**
 * Format a full date/time for tooltips
 */
export function formatFullDateTime(timestamp: string | number | Date): string {
  const date = new Date(timestamp);
  return format(date, 'MMM d, yyyy h:mm a');
}

/**
 * Format relative time (e.g., "2 hours ago")
 */
export function formatRelativeTime(timestamp: string | number | Date): string {
  const date = new Date(timestamp);
  return formatDistanceToNow(date, { addSuffix: true });
}

/**
 * Format time only (e.g., "2:30 PM")
 */
export function formatTimeOnly(timestamp: string | number | Date): string {
  const date = new Date(timestamp);
  return format(date, 'h:mm a');
}

/**
 * Format date separator label
 */
export function formatDateSeparator(timestamp: string | number | Date): string {
  const date = new Date(timestamp);
  
  if (isToday(date)) {
    return 'Today';
  }
  
  if (isYesterday(date)) {
    return 'Yesterday';
  }
  
  return format(date, 'EEEE, MMMM d, yyyy');
}

/**
 * Check if two timestamps are on the same day
 */
export function isSameDay(a: string | number | Date, b: string | number | Date): boolean {
  const dateA = new Date(a);
  const dateB = new Date(b);
  return (
    dateA.getFullYear() === dateB.getFullYear() &&
    dateA.getMonth() === dateB.getMonth() &&
    dateA.getDate() === dateB.getDate()
  );
}

/**
 * Get initials from username
 */
export function getInitials(username: string, count = 2): string {
  if (!username) return '?';
  
  const parts = username.trim().split(/\s+/);
  if (parts.length === 1) {
    return username.slice(0, count).toUpperCase();
  }
  
  return parts
    .slice(0, count)
    .map(p => p[0])
    .join('')
    .toUpperCase();
}

