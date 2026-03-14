// ============================================
// Markdown Utility Tests
// ============================================

import { describe, it, expect } from 'vitest';
import {
  renderMarkdown,
  extractPlainText,
  hasMentions,
  getMentions,
  truncateText,
} from '../../src/utils/markdown';

describe('Markdown Utilities', () => {
  describe('renderMarkdown', () => {
    it('should render basic text', () => {
      const result = renderMarkdown('Hello world');
      expect(result).toContain('Hello world');
    });

    it('should return empty string for empty input', () => {
      expect(renderMarkdown('')).toBe('');
    });

    it('should sanitize malicious HTML', () => {
      const result = renderMarkdown('<script>alert("xss")</script>');
      expect(result).not.toContain('<script>');
    });
  });

  describe('extractPlainText', () => {
    it('should extract plain text from markdown', () => {
      const result = extractPlainText('**bold** and *italic*');
      expect(result).toBe('bold and italic');
    });

    it('should replace code blocks with [code]', () => {
      const result = extractPlainText('Some text\n```\ncode\n```\nMore text');
      expect(result).toContain('[code]');
    });

    it('should extract text from links', () => {
      const result = extractPlainText('[link text](https://example.com)');
      expect(result).toBe('link text');
    });

    it('should return empty string for empty input', () => {
      expect(extractPlainText('')).toBe('');
    });
  });

  describe('hasMentions', () => {
    it('should detect @mentions', () => {
      expect(hasMentions('Hello @user')).toBe(true);
      expect(hasMentions('Hello @user123')).toBe(true);
    });

    it('should not detect false positives', () => {
      expect(hasMentions('Hello user')).toBe(false);
    });
  });

  describe('getMentions', () => {
    it('should extract mentioned usernames', () => {
      expect(getMentions('Hello @user1 and @user2')).toEqual(['user1', 'user2']);
    });

    it('should return empty array when no mentions', () => {
      expect(getMentions('Hello everyone')).toEqual([]);
    });
  });

  describe('truncateText', () => {
    it('should not truncate short text', () => {
      expect(truncateText('short', 100)).toBe('short');
    });

    it('should truncate long text with ellipsis', () => {
      const text = 'a'.repeat(100);
      const result = truncateText(text, 20);
      expect(result).toHaveLength(20);
      expect(result.slice(-3)).toBe('...');
    });
  });
});
