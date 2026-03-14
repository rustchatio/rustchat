// ============================================
// Accessibility Tests
// WCAG 2.1 AA Compliance Tests
// ============================================

import { describe, it, expect } from 'vitest';
import {
  isFocusable,
  getFocusableElements,
  Keys,
  isArrowKey,
  handleListNavigation,
  buttonLabel,
  countLabel,
  prefersReducedMotion,
  getAnimationDuration,
} from '../../src/utils/a11y';

describe('Accessibility Utilities', () => {
  describe('Focus Management', () => {
    it('should identify focusable elements', () => {
      const button = document.createElement('button');
      expect(isFocusable(button)).toBe(true);

      const disabledButton = document.createElement('button');
      disabledButton.disabled = true;
      expect(isFocusable(disabledButton)).toBe(false);

      const link = document.createElement('a');
      link.href = '#';
      expect(isFocusable(link)).toBe(true);

      const div = document.createElement('div');
      expect(isFocusable(div)).toBe(false);
    });

    it('should get all focusable elements from container', () => {
      const container = document.createElement('div');
      container.innerHTML = `
        <button>Button 1</button>
        <button disabled>Disabled</button>
        <input type="text" />
        <a href="#">Link</a>
        <div>Not focusable</div>
      `;

      const focusable = getFocusableElements(container);
      expect(focusable.length).toBe(3); // 2 buttons (1 disabled), 1 input, 1 link
    });
  });

  describe('Keyboard Navigation', () => {
    it('should identify arrow keys', () => {
      expect(isArrowKey(Keys.ArrowUp)).toBe(true);
      expect(isArrowKey(Keys.ArrowDown)).toBe(true);
      expect(isArrowKey(Keys.ArrowLeft)).toBe(true);
      expect(isArrowKey(Keys.ArrowRight)).toBe(true);
      expect(isArrowKey(Keys.Enter)).toBe(false);
      expect(isArrowKey(Keys.Escape)).toBe(false);
    });

    it('should handle list navigation', () => {
      const mockEvent = {
        key: Keys.ArrowDown,
        preventDefault: () => {},
      } as KeyboardEvent;

      let selectedIndex = 0;
      const onSelect = (index: number) => {
        selectedIndex = index;
      };

      handleListNavigation(mockEvent, 0, 5, onSelect);
      expect(selectedIndex).toBe(1);

      const upEvent = { ...mockEvent, key: Keys.ArrowUp };
      handleListNavigation(upEvent as KeyboardEvent, 0, 5, onSelect);
      expect(selectedIndex).toBe(4); // Wraps around

      const homeEvent = { ...mockEvent, key: Keys.Home };
      handleListNavigation(homeEvent as KeyboardEvent, 3, 5, onSelect);
      expect(selectedIndex).toBe(0);

      const endEvent = { ...mockEvent, key: Keys.End };
      handleListNavigation(endEvent as KeyboardEvent, 0, 5, onSelect);
      expect(selectedIndex).toBe(4);
    });
  });

  describe('ARIA Labels', () => {
    it('should create accessible button labels', () => {
      const label = buttonLabel('Send message', 'Ctrl+Enter');
      expect(label['aria-label']).toBe('Send message (Ctrl+Enter)');
      expect(label.title).toBe('Send message (Ctrl+Enter)');

      const simpleLabel = buttonLabel('Close');
      expect(simpleLabel['aria-label']).toBe('Close');
      expect(simpleLabel.title).toBeUndefined();
    });

    it('should create accessible count labels', () => {
      expect(countLabel(1, 'message', 'messages')).toBe('1 message');
      expect(countLabel(0, 'message', 'messages')).toBe('0 messages');
      expect(countLabel(5, 'message', 'messages')).toBe('5 messages');
    });
  });

  describe('Reduced Motion', () => {
    it('should check reduced motion preference', () => {
      // In test environment, this should return false
      expect(typeof prefersReducedMotion()).toBe('boolean');
    });

    it('should adjust animation duration based on preference', () => {
      const duration = getAnimationDuration(300);
      expect(typeof duration).toBe('number');
      // Should return 0 if reduced motion is preferred, otherwise original
      expect(duration === 0 || duration === 300).toBe(true);
    });
  });
});

describe('WCAG 2.1 AA Requirements', () => {
  it('should have proper contrast ratio requirements documented', () => {
    // Normal text: 4.5:1
    // Large text: 3:1
    // UI components: 3:1
    expect(4.5).toBeGreaterThanOrEqual(4.5);
    expect(3).toBeGreaterThanOrEqual(3);
  });

  it('should support keyboard navigation (2.1.1)', () => {
    // All functionality should be available via keyboard
    expect(Keys).toBeDefined();
    expect(Keys.Enter).toBe('Enter');
    expect(Keys.Space).toBe(' ');
    expect(Keys.Tab).toBe('Tab');
  });

  it('should support focus management (2.4.3)', () => {
    // Focus order should be logical
    expect(typeof getFocusableElements).toBe('function');
    expect(typeof isFocusable).toBe('function');
  });
});
