// ============================================
// OIDC Authentication Tests
// ============================================

import { describe, it, expect, beforeEach, vi } from 'vitest';
import {
  generateCodeVerifier,
  generateCodeChallenge,
  generateState,
  generateNonce,
  buildAuthorizationUrl,
  parseIDToken,
  validateNonce,
  storePKCE,
  getPKCE,
  clearPKCE,
  storeState,
  getState,
  clearState,
  storeNonce,
  getNonce,
  clearNonce,
  storeOIDCRedirectUrl,
  getOIDCRedirectUrl,
  clearOIDCRedirectUrl,
  clearAllOIDCStorage,
} from '../../src/auth/oidc';

// Mock sessionStorage
const mockSessionStorage = new Map<string, string>();
vi.stubGlobal('sessionStorage', {
  getItem: (key: string) => mockSessionStorage.get(key) || null,
  setItem: (key: string, value: string) => mockSessionStorage.set(key, value),
  removeItem: (key: string) => mockSessionStorage.delete(key),
});

// Mock window
vi.stubGlobal('window', { location: { href: '' } });

describe('OIDC PKCE', () => {
  beforeEach(() => {
    mockSessionStorage.clear();
  });

  describe('generateCodeVerifier', () => {
    it('should generate a 43-character code verifier', () => {
      const verifier = generateCodeVerifier();
      expect(verifier).toHaveLength(43);
      expect(verifier).toMatch(/^[A-Za-z0-9_-]+$/);
    });

    it('should generate unique verifiers', () => {
      const verifier1 = generateCodeVerifier();
      const verifier2 = generateCodeVerifier();
      expect(verifier1).not.toBe(verifier2);
    });
  });

  describe('generateCodeChallenge', () => {
    it('should generate S256 code challenge', async () => {
      const verifier = 'test_verifier_1234567890123456789012345678901234567890';
      const challenge = await generateCodeChallenge(verifier);
      expect(challenge).toBeDefined();
      expect(challenge).not.toBe(verifier);
    });
  });

  describe('generateState', () => {
    it('should generate a 32-character state', () => {
      const state = generateState();
      expect(state).toHaveLength(32);
    });

    it('should generate unique states', () => {
      const state1 = generateState();
      const state2 = generateState();
      expect(state1).not.toBe(state2);
    });
  });

  describe('generateNonce', () => {
    it('should generate a 32-character nonce', () => {
      const nonce = generateNonce();
      expect(nonce).toHaveLength(32);
    });

    it('should generate unique nonces', () => {
      const nonce1 = generateNonce();
      const nonce2 = generateNonce();
      expect(nonce1).not.toBe(nonce2);
    });
  });
});

describe('OIDC Storage', () => {
  beforeEach(() => {
    mockSessionStorage.clear();
  });

  describe('PKCE storage', () => {
    it('should store and retrieve PKCE', () => {
      storePKCE('test_verifier');
      expect(getPKCE()).toBe('test_verifier');
    });

    it('should clear PKCE', () => {
      storePKCE('test_verifier');
      clearPKCE();
      expect(getPKCE()).toBeNull();
    });
  });

  describe('State storage', () => {
    it('should store and retrieve state', () => {
      storeState('test_state');
      expect(getState()).toBe('test_state');
    });

    it('should clear state', () => {
      storeState('test_state');
      clearState();
      expect(getState()).toBeNull();
    });
  });

  describe('Nonce storage', () => {
    it('should store and retrieve nonce', () => {
      storeNonce('test_nonce');
      expect(getNonce()).toBe('test_nonce');
    });

    it('should clear nonce', () => {
      storeNonce('test_nonce');
      clearNonce();
      expect(getNonce()).toBeNull();
    });
  });

  describe('Redirect URL storage', () => {
    it('should store and retrieve redirect URL', () => {
      storeOIDCRedirectUrl('/channels/general');
      expect(getOIDCRedirectUrl()).toBe('/channels/general');
    });

    it('should clear redirect URL', () => {
      storeOIDCRedirectUrl('/channels/general');
      clearOIDCRedirectUrl();
      expect(getOIDCRedirectUrl()).toBeNull();
    });
  });

  describe('clearAllOIDCStorage', () => {
    it('should clear all OIDC storage', () => {
      storePKCE('test_verifier');
      storeState('test_state');
      storeNonce('test_nonce');
      storeOIDCRedirectUrl('/test');

      clearAllOIDCStorage();

      expect(getPKCE()).toBeNull();
      expect(getState()).toBeNull();
      expect(getNonce()).toBeNull();
      expect(getOIDCRedirectUrl()).toBeNull();
    });
  });
});

describe('OIDC URL Building', () => {
  it('should build authorization URL with required params', () => {
    const url = buildAuthorizationUrl({
      authorizationEndpoint: 'https://auth.example.com/authorize',
      clientId: 'test_client',
      redirectUri: 'https://app.example.com/callback',
      codeChallenge: 'test_challenge',
      state: 'test_state',
      nonce: 'test_nonce',
    });

    expect(url).toContain('https://auth.example.com/authorize');
    expect(url).toContain('client_id=test_client');
    expect(url).toContain('response_type=code');
    expect(url).toContain('code_challenge=test_challenge');
    expect(url).toContain('code_challenge_method=S256');
    expect(url).toContain('state=test_state');
    expect(url).toContain('nonce=test_nonce');
  });

  it('should include default scope', () => {
    const url = buildAuthorizationUrl({
      authorizationEndpoint: 'https://auth.example.com/authorize',
      clientId: 'test_client',
      redirectUri: 'https://app.example.com/callback',
      codeChallenge: 'test_challenge',
      state: 'test_state',
      nonce: 'test_nonce',
    });

    expect(url).toContain('scope=openid+email+profile');
  });

  it('should include custom scope', () => {
    const url = buildAuthorizationUrl({
      authorizationEndpoint: 'https://auth.example.com/authorize',
      clientId: 'test_client',
      redirectUri: 'https://app.example.com/callback',
      codeChallenge: 'test_challenge',
      state: 'test_state',
      nonce: 'test_nonce',
      scope: 'openid email profile custom',
    });

    expect(url).toContain('scope=openid+email+profile+custom');
  });

  it('should include extra params', () => {
    const url = buildAuthorizationUrl({
      authorizationEndpoint: 'https://auth.example.com/authorize',
      clientId: 'test_client',
      redirectUri: 'https://app.example.com/callback',
      codeChallenge: 'test_challenge',
      state: 'test_state',
      nonce: 'test_nonce',
      extraParams: { prompt: 'consent', access_type: 'offline' },
    });

    expect(url).toContain('prompt=consent');
    expect(url).toContain('access_type=offline');
  });
});

describe('ID Token Parsing', () => {
  it('should parse valid ID token', () => {
    // Create a mock JWT payload
    const payload = { sub: 'user123', email: 'test@example.com', nonce: 'test_nonce' };
    const base64Payload = btoa(JSON.stringify(payload));
    const idToken = `header.${base64Payload}.signature`;

    const parsed = parseIDToken(idToken);
    expect(parsed).toEqual(payload);
  });

  it('should return null for invalid token', () => {
    const parsed = parseIDToken('invalid');
    expect(parsed).toBeNull();
  });

  it('should return null for empty token', () => {
    const parsed = parseIDToken('');
    expect(parsed).toBeNull();
  });
});

describe('Nonce Validation', () => {
  it('should validate matching nonce', () => {
    const payload = { nonce: 'test_nonce' };
    const base64Payload = btoa(JSON.stringify(payload));
    const idToken = `header.${base64Payload}.signature`;

    expect(validateNonce(idToken, 'test_nonce')).toBe(true);
  });

  it('should reject non-matching nonce', () => {
    const payload = { nonce: 'test_nonce' };
    const base64Payload = btoa(JSON.stringify(payload));
    const idToken = `header.${base64Payload}.signature`;

    expect(validateNonce(idToken, 'wrong_nonce')).toBe(false);
  });

  it('should reject token without nonce', () => {
    const payload = { sub: 'user123' };
    const base64Payload = btoa(JSON.stringify(payload));
    const idToken = `header.${base64Payload}.signature`;

    expect(validateNonce(idToken, 'test_nonce')).toBe(false);
  });

  it('should reject invalid token', () => {
    expect(validateNonce('invalid', 'test_nonce')).toBe(false);
  });
});
