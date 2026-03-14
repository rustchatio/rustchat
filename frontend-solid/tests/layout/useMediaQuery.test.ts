// ============================================
// useMediaQuery Hook Tests
// ============================================

import { describe, it, expect } from 'vitest';
import { breakpoints, useMediaQuery } from '../../src/hooks/useMediaQuery';

describe('useMediaQuery', () => {
  describe('breakpoints', () => {
    it('should have correct breakpoint values', () => {
      expect(breakpoints.sm).toBe(640);
      expect(breakpoints.md).toBe(768);
      expect(breakpoints.lg).toBe(1024);
      expect(breakpoints.xl).toBe(1280);
      expect(breakpoints['2xl']).toBe(1536);
    });
  });

  // Note: Testing actual media query behavior requires a browser environment
  // These tests verify the hook exports correctly
  it('should export useMediaQuery function', () => {
    expect(typeof useMediaQuery).toBe('function');
  });
});
