// ============================================
// Password Strength & Validation Tests
// ============================================

import { describe, it, expect } from 'vitest';

// Password strength calculator (copied from ResetPassword.tsx for testing)
function calculatePasswordStrength(password: string): { score: number; label: string } {
  if (!password) {
    return { score: 0, label: 'Enter password' };
  }

  let score = 0;
  const checks = {
    length: password.length >= 8,
    lowercase: /[a-z]/.test(password),
    uppercase: /[A-Z]/.test(password),
    number: /\d/.test(password),
    special: /[!@#$%^&*(),.?":{}|<>]/.test(password),
  };

  const passedChecks = Object.values(checks).filter(Boolean).length;

  if (password.length < 6) {
    score = 0;
  } else {
    score = Math.min(4, Math.floor((passedChecks / 5) * 4) + (password.length >= 12 ? 1 : 0));
  }

  const labels = ['Very Weak', 'Weak', 'Fair', 'Good', 'Strong'];

  return {
    score,
    label: labels[score],
  };
}

function getPasswordRequirements(password: string) {
  return {
    length: password.length >= 8,
    lowercase: /[a-z]/.test(password),
    uppercase: /[A-Z]/.test(password),
    number: /\d/.test(password),
    special: /[!@#$%^&*(),.?":{}|<>]/.test(password),
  };
}

describe('Password Strength', () => {
  it('should return score 0 for empty password', () => {
    const result = calculatePasswordStrength('');
    expect(result.score).toBe(0);
    expect(result.label).toBe('Enter password');
  });

  it('should return Weak for short password', () => {
    const result = calculatePasswordStrength('abc');
    expect(result.score).toBeLessThanOrEqual(1);
  });

  it('should return higher score for longer password', () => {
    const weak = calculatePasswordStrength('password');
    const strong = calculatePasswordStrength('password123!ABC');
    expect(strong.score).toBeGreaterThan(weak.score);
  });

  it('should return Strong for complex password', () => {
    const result = calculatePasswordStrength('MyP@ssw0rd!2024');
    expect(result.score).toBeGreaterThanOrEqual(3);
  });

  it('should require all character types for max score', () => {
    const onlyLower = calculatePasswordStrength('password');
    const allTypes = calculatePasswordStrength('Passw0rd!');
    expect(allTypes.score).toBeGreaterThan(onlyLower.score);
  });
});

describe('Password Requirements', () => {
  it('should detect length requirement', () => {
    expect(getPasswordRequirements('short').length).toBe(false);
    expect(getPasswordRequirements('longenough').length).toBe(true);
  });

  it('should detect lowercase requirement', () => {
    expect(getPasswordRequirements('PASSWORD').lowercase).toBe(false);
    expect(getPasswordRequirements('Password').lowercase).toBe(true);
  });

  it('should detect uppercase requirement', () => {
    expect(getPasswordRequirements('password').uppercase).toBe(false);
    expect(getPasswordRequirements('Password').uppercase).toBe(true);
  });

  it('should detect number requirement', () => {
    expect(getPasswordRequirements('password').number).toBe(false);
    expect(getPasswordRequirements('passw0rd').number).toBe(true);
  });

  it('should detect special character requirement', () => {
    expect(getPasswordRequirements('password').special).toBe(false);
    expect(getPasswordRequirements('pass!word').special).toBe(true);
  });

  it('should pass all requirements for valid password', () => {
    const reqs = getPasswordRequirements('MyP@ssw0rd');
    expect(reqs.length).toBe(true);
    expect(reqs.lowercase).toBe(true);
    expect(reqs.uppercase).toBe(true);
    expect(reqs.number).toBe(true);
    expect(reqs.special).toBe(true);
  });
});
