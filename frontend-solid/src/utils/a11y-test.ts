// ============================================
// Accessibility Testing Utilities
// Uses axe-core for automated accessibility testing
// ============================================

import type { Result, RunOptions } from 'axe-core';

// ============================================
// axe-core Integration
// ============================================

let axeCore: typeof import('axe-core') | null = null;

/**
 * Dynamically import axe-core (for browser environments)
 */
async function getAxe(): Promise<typeof import('axe-core')> {
  if (!axeCore) {
    axeCore = await import('axe-core');
  }
  return axeCore;
}

// ============================================
// WCAG 2.1 AA Configuration
// ============================================

const WCAG21_AA_CONFIG: RunOptions = {
  runOnly: {
    type: 'tag',
    values: ['wcag2a', 'wcag2aa', 'wcag21aa', 'best-practice'],
  },
  rules: {
    // Enable experimental rules
    'experimental': { enabled: true },
    // Disable rules that require manual review
    'color-contrast': { enabled: true },
    'bypass': { enabled: false }, // Skip link handled separately
  },
};

// ============================================
// Test Runner
// ============================================

export interface A11yTestResult {
  passes: number;
  violations: number;
  incomplete: number;
  violationsList: Result[];
}

/**
 * Run accessibility audit on an element
 */
export async function runA11yAudit(
  element: HTMLElement = document.body,
  options: RunOptions = WCAG21_AA_CONFIG
): Promise<A11yTestResult> {
  const axe = await getAxe();

  try {
    const results = await axe.run(element, options);

    return {
      passes: results.passes.length,
      violations: results.violations.length,
      incomplete: results.incomplete.length,
      violationsList: results.violations,
    };
  } catch (error) {
    console.error('Accessibility audit failed:', error);
    throw error;
  }
}

/**
 * Quick check for critical accessibility issues
 */
export async function quickA11yCheck(
  element: HTMLElement = document.body
): Promise<{ hasViolations: boolean; criticalIssues: string[] }> {
  const axe = await getAxe();

  const criticalConfig: RunOptions = {
    runOnly: {
      type: 'rule',
      values: [
        'aria-hidden-focus',
        'button-name',
        'color-contrast',
        'duplicate-id',
        'image-alt',
        'label',
        'link-name',
        'region',
        'tabindex',
      ],
    },
  };

  try {
    const results = await axe.run(element, criticalConfig);

    return {
      hasViolations: results.violations.length > 0,
      criticalIssues: results.violations.map((v) => v.description),
    };
  } catch (error) {
    console.error('Quick a11y check failed:', error);
    return { hasViolations: true, criticalIssues: ['Audit failed'] };
  }
}

// ============================================
// Console Reporter
// ============================================

/**
 * Report accessibility violations to console
 */
export function reportViolations(result: A11yTestResult): void {
  if (result.violations === 0) {
    console.log('✅ No accessibility violations found!');
    return;
  }

  console.group(`❌ ${result.violations} accessibility violations found:`);

  result.violationsList.forEach((violation, index) => {
    console.group(`${index + 1}. ${violation.description}`);
    console.log('Impact:', violation.impact);
    console.log('Help:', violation.help);
    console.log('WCAG:', violation.tags.filter((t) => t.startsWith('wcag')).join(', '));
    console.log('Elements:', violation.nodes.length);

    violation.nodes.slice(0, 3).forEach((node) => {
      console.log('  -', node.html.substring(0, 100));
    });

    if (violation.nodes.length > 3) {
      console.log(`  ... and ${violation.nodes.length - 3} more`);
    }

    console.groupEnd();
  });

  console.groupEnd();
}

// ============================================
// Component Testing Helpers
// ============================================

/**
 * Test a component for accessibility
 * Usage in tests: await testComponentAccessibility(() => <MyComponent />)
 */
export async function testComponentAccessibility(
  renderFn: () => HTMLElement,
  options?: RunOptions
): Promise<A11yTestResult> {
  const container = renderFn();
  document.body.appendChild(container);

  try {
    const result = await runA11yAudit(container, options);
    reportViolations(result);
    return result;
  } finally {
    document.body.removeChild(container);
  }
}

// ============================================
// Accessibility Assertions
// ============================================

/**
 * Assert no accessibility violations
 * For use in tests
 */
export function expectNoViolations(result: A11yTestResult): void {
  if (result.violations > 0) {
    const messages = result.violationsList.map(
      (v) => `${v.description} (${v.impact})`
    );
    throw new Error(
      `Expected no accessibility violations but found ${result.violations}:\n${messages.join('\n')}`
    );
  }
}

// ============================================
// Keyboard Navigation Testing
// ============================================

export interface KeyboardTestStep {
  key: string;
  shiftKey?: boolean;
  expectedFocus?: string;
  description: string;
}

/**
 * Test keyboard navigation flow
 */
export async function testKeyboardNavigation(
  steps: KeyboardTestStep[],
  container: HTMLElement
): Promise<{ passed: boolean; errors: string[] }> {
  const errors: string[] = [];

  for (const step of steps) {
    // Dispatch keyboard event
    const event = new KeyboardEvent('keydown', {
      key: step.key,
      shiftKey: step.shiftKey,
      bubbles: true,
    });
    document.activeElement?.dispatchEvent(event);

    // Check focus if expected
    if (step.expectedFocus) {
      const expected = container.querySelector(step.expectedFocus);
      if (document.activeElement !== expected) {
        errors.push(
          `Step "${step.description}": Expected focus on "${step.expectedFocus}" but found "${document.activeElement?.tagName}"`
        );
      }
    }

    // Small delay between steps
    await new Promise((resolve) => setTimeout(resolve, 50));
  }

  return { passed: errors.length === 0, errors };
}

// ============================================
// Focus Management Testing
// ============================================

/**
 * Test focus trap within a container
 */
export async function testFocusTrap(
  container: HTMLElement,
  options: { tabCount?: number } = {}
): Promise<{ isTrapped: boolean; tabOrder: string[] }> {
  const tabCount = options.tabCount ?? 10;
  const tabOrder: string[] = [];

  // Tab through elements
  for (let i = 0; i < tabCount; i++) {
    const event = new KeyboardEvent('keydown', {
      key: 'Tab',
      bubbles: true,
    });
    document.dispatchEvent(event);

    const activeEl = document.activeElement;
    if (activeEl) {
      tabOrder.push(activeEl.tagName + (activeEl.id ? `#${activeEl.id}` : ''));

      // Check if focus left the container
      if (!container.contains(activeEl)) {
        return { isTrapped: false, tabOrder };
      }
    }
  }

  return { isTrapped: true, tabOrder };
}

// ============================================
// Export
// ============================================

export default {
  runA11yAudit,
  quickA11yCheck,
  reportViolations,
  testComponentAccessibility,
  expectNoViolations,
  testKeyboardNavigation,
  testFocusTrap,
  WCAG21_AA_CONFIG,
};
