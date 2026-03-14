// ============================================
// Date Utility Tests
// ============================================

import { describe, it, expect } from 'vitest';
import {
  formatMessageTime,
  formatFullDateTime,
  formatDateSeparator,
  isSameDay,
  getInitials,
} from '../../src/utils/date';
import { formatFileSize } from '../../src/utils/file';

describe('Date Utilities', () => {
  describe('formatMessageTime', () => {
    it('should format today as time only', () => {
      const now = new Date();
      const result = formatMessageTime(now.toISOString());
      // Should be time format like "2:30 PM"
      expect(result).toMatch(/\d{1,2}:\d{2} (AM|PM)/);
    });

    it('should format yesterday as "Yesterday"', () => {
      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);
      const result = formatMessageTime(yesterday.toISOString());
      expect(result).toBe('Yesterday');
    });
  });

  describe('formatFullDateTime', () => {
    it('should format full date and time', () => {
      const date = new Date('2024-01-15T14:30:00');
      const result = formatFullDateTime(date.toISOString());
      // Should contain date and time
      expect(result).toContain('2024');
      expect(result).toContain('Jan');
      expect(result).toContain('15');
    });
  });

  describe('formatDateSeparator', () => {
    it('should format today as "Today"', () => {
      const now = new Date();
      const result = formatDateSeparator(now.toISOString());
      expect(result).toBe('Today');
    });

    it('should format yesterday as "Yesterday"', () => {
      const yesterday = new Date();
      yesterday.setDate(yesterday.getDate() - 1);
      const result = formatDateSeparator(yesterday.toISOString());
      expect(result).toBe('Yesterday');
    });

    it('should format older dates with full date', () => {
      const oldDate = new Date('2024-01-15T14:30:00');
      const result = formatDateSeparator(oldDate.toISOString());
      expect(result).toContain('Monday');
      expect(result).toContain('January');
      expect(result).toContain('15');
      expect(result).toContain('2024');
    });
  });

  describe('isSameDay', () => {
    it('should return true for same day', () => {
      const date1 = new Date('2024-01-15T10:00:00');
      const date2 = new Date('2024-01-15T14:00:00');
      expect(isSameDay(date1, date2)).toBe(true);
    });

    it('should return false for different days', () => {
      const date1 = new Date('2024-01-15T10:00:00');
      const date2 = new Date('2024-01-16T10:00:00');
      expect(isSameDay(date1, date2)).toBe(false);
    });
  });

  describe('getInitials', () => {
    it('should get initials from single word', () => {
      expect(getInitials('john', 2)).toBe('JO');
      expect(getInitials('john', 1)).toBe('J');
    });

    it('should get initials from multiple words', () => {
      expect(getInitials('john doe')).toBe('JD');
      expect(getInitials('john doe smith')).toBe('JD');
    });

    it('should handle empty string', () => {
      expect(getInitials('')).toBe('?');
    });
  });

  describe('formatFileSize', () => {
    it('should format bytes', () => {
      expect(formatFileSize(0)).toBe('0 B');
      expect(formatFileSize(100)).toBe('100 B');
    });

    it('should format KB', () => {
      expect(formatFileSize(1024)).toBe('1 KB');
      expect(formatFileSize(2048)).toBe('2 KB');
    });

    it('should format MB', () => {
      expect(formatFileSize(1024 * 1024)).toBe('1 MB');
      expect(formatFileSize(5 * 1024 * 1024)).toBe('5 MB');
    });

    it('should format GB', () => {
      expect(formatFileSize(1024 * 1024 * 1024)).toBe('1 GB');
    });
  });
});
