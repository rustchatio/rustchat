// ============================================
// OIDC Authentication Module
// ============================================

import { client } from '../api/client';

// ============================================
// Types
// ============================================

export interface OIDCConfig {
  authorization_endpoint: string;
  token_endpoint: string;
  userinfo_endpoint?: string;
  end_session_endpoint?: string;
  jwks_uri?: string;
  issuer?: string;
  scopes_supported?: string[];
}

export interface OIDCAuthParams {
  client_id: string;
  redirect_uri: string;
  scope: string;
  response_type: string;
  code_challenge: string;
  code_challenge_method: string;
  state: string;
  nonce: string;
}

export interface TokenResponse {
  access_token: string;
  id_token?: string;
  refresh_token?: string;
  token_type: string;
  expires_in: number;
  scope?: string;
}

export interface OIDCUserInfo {
  sub: string;
  name?: string;
  given_name?: string;
  family_name?: string;
  middle_name?: string;
  nickname?: string;
  preferred_username?: string;
  profile?: string;
  picture?: string;
  website?: string;
  email?: string;
  email_verified?: boolean;
  gender?: string;
  birthdate?: string;
  zoneinfo?: string;
  locale?: string;
  phone_number?: string;
  phone_number_verified?: boolean;
  updated_at?: number;
}

// ============================================
// PKCE Code Verifier/Challenge Generation
// ============================================

const PKCE_KEY = 'rustchat_oidc_pkce';
const STATE_KEY = 'rustchat_oidc_state';
const NONCE_KEY = 'rustchat_oidc_nonce';
const OIDC_REDIRECT_KEY = 'rustchat_oidc_redirect';

/**
 * Generate a cryptographically secure random string
 */
function generateRandomString(length: number): string {
  const array = new Uint8Array(length);
  crypto.getRandomValues(array);
  return Array.from(array, (byte) => byte.toString(36).padStart(2, '0')).join('').slice(0, length);
}

/**
 * Generate PKCE code verifier
 * Returns a base64url-encoded random string
 */
export function generateCodeVerifier(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return base64URLEncode(array);
}

/**
 * Generate PKCE code challenge from verifier
 * Uses SHA-256 hash
 */
export async function generateCodeChallenge(verifier: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(verifier);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  return base64URLEncode(new Uint8Array(hashBuffer));
}

/**
 * Base64URL encode a Uint8Array
 */
function base64URLEncode(buffer: Uint8Array): string {
  const base64 = btoa(String.fromCharCode(...buffer));
  return base64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

/**
 * Generate state parameter for CSRF protection
 */
export function generateState(): string {
  return generateRandomString(32);
}

/**
 * Generate nonce for replay attack protection
 */
export function generateNonce(): string {
  return generateRandomString(32);
}

// ============================================
// Storage Helpers
// ============================================

export function storePKCE(verifier: string): void {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem(PKCE_KEY, verifier);
}

export function getPKCE(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(PKCE_KEY);
}

export function clearPKCE(): void {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem(PKCE_KEY);
}

export function storeState(state: string): void {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem(STATE_KEY, state);
}

export function getState(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(STATE_KEY);
}

export function clearState(): void {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem(STATE_KEY);
}

export function storeNonce(nonce: string): void {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem(NONCE_KEY, nonce);
}

export function getNonce(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(NONCE_KEY);
}

export function clearNonce(): void {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem(NONCE_KEY);
}

export function storeOIDCRedirectUrl(url: string): void {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem(OIDC_REDIRECT_KEY, url);
}

export function getOIDCRedirectUrl(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(OIDC_REDIRECT_KEY);
}

export function clearOIDCRedirectUrl(): void {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem(OIDC_REDIRECT_KEY);
}

export function clearAllOIDCStorage(): void {
  clearPKCE();
  clearState();
  clearNonce();
  clearOIDCRedirectUrl();
}

// ============================================
// OIDC Authorization URL Construction
// ============================================

export interface AuthorizationUrlOptions {
  authorizationEndpoint: string;
  clientId: string;
  redirectUri: string;
  scope?: string;
  codeChallenge: string;
  state: string;
  nonce: string;
  extraParams?: Record<string, string>;
}

/**
 * Build OIDC authorization URL with PKCE
 */
export function buildAuthorizationUrl(options: AuthorizationUrlOptions): string {
  const params = new URLSearchParams({
    client_id: options.clientId,
    redirect_uri: options.redirectUri,
    response_type: 'code',
    scope: options.scope || 'openid email profile',
    code_challenge: options.codeChallenge,
    code_challenge_method: 'S256',
    state: options.state,
    nonce: options.nonce,
  });

  // Add any extra parameters
  if (options.extraParams) {
    Object.entries(options.extraParams).forEach(([key, value]) => {
      params.append(key, value);
    });
  }

  return `${options.authorizationEndpoint}?${params.toString()}`;
}

// ============================================
// OIDC Discovery
// ============================================

/**
 * Fetch OIDC configuration from discovery endpoint
 */
export async function discoverOIDCConfig(issuerUrl: string): Promise<OIDCConfig> {
  const discoveryUrl = issuerUrl.endsWith('/')
    ? `${issuerUrl}.well-known/openid-configuration`
    : `${issuerUrl}/.well-known/openid-configuration`;

  const response = await fetch(discoveryUrl);
  if (!response.ok) {
    throw new Error(`Failed to fetch OIDC configuration: ${response.statusText}`);
  }

  return response.json();
}

// ============================================
// Token Exchange
// ============================================

export interface TokenExchangeParams {
  tokenEndpoint: string;
  clientId: string;
  clientSecret?: string;
  code: string;
  redirectUri: string;
  codeVerifier: string;
}

/**
 * Exchange authorization code for tokens
 */
export async function exchangeCodeForTokens(params: TokenExchangeParams): Promise<TokenResponse> {
  const body = new URLSearchParams({
    grant_type: 'authorization_code',
    client_id: params.clientId,
    code: params.code,
    redirect_uri: params.redirectUri,
    code_verifier: params.codeVerifier,
  });

  if (params.clientSecret) {
    body.append('client_secret', params.clientSecret);
  }

  const response = await fetch(params.tokenEndpoint, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/x-www-form-urlencoded',
      Accept: 'application/json',
    },
    body: body.toString(),
  });

  if (!response.ok) {
    const error = await response.text();
    throw new Error(`Token exchange failed: ${error}`);
  }

  return response.json();
}

/**
 * Exchange code using backend API (preferred method)
 */
export async function exchangeCodeWithBackend(code: string, state: string): Promise<TokenResponse> {
  const response = await client.post<TokenResponse>('/auth/oidc/callback', {
    code,
    state,
  });
  return response.data;
}

// ============================================
// UserInfo
// ============================================

/**
 * Fetch user info from OIDC provider
 */
export async function fetchUserInfo(userinfoEndpoint: string, accessToken: string): Promise<OIDCUserInfo> {
  const response = await fetch(userinfoEndpoint, {
    headers: {
      Authorization: `Bearer ${accessToken}`,
      Accept: 'application/json',
    },
  });

  if (!response.ok) {
    throw new Error(`Failed to fetch user info: ${response.statusText}`);
  }

  return response.json();
}

// ============================================
// ID Token Validation
// ============================================

/**
 * Parse ID token (JWT) without verification
 * Note: Full verification should be done server-side
 */
export function parseIDToken(idToken: string): Record<string, unknown> | null {
  try {
    const parts = idToken.split('.');
    if (parts.length !== 3) return null;

    const payload = parts[1];
    const normalized = payload.replace(/-/g, '+').replace(/_/g, '/');
    const paddingLength = (4 - (normalized.length % 4)) % 4;
    const padded = normalized + '='.repeat(paddingLength);

    return JSON.parse(atob(padded));
  } catch {
    return null;
  }
}

/**
 * Validate nonce from ID token
 */
export function validateNonce(idToken: string, expectedNonce: string): boolean {
  const payload = parseIDToken(idToken);
  if (!payload || !payload.nonce) return false;
  return payload.nonce === expectedNonce;
}

// ============================================
// Complete OIDC Flow
// ============================================

export interface InitiateOIDCLoginOptions {
  authorizationEndpoint: string;
  clientId: string;
  redirectUri: string;
  scope?: string;
  redirectUrl?: string;
  extraParams?: Record<string, string>;
}

/**
 * Initiate OIDC login flow
 * Generates PKCE, state, nonce and redirects to authorization endpoint
 */
export async function initiateOIDCLogin(options: InitiateOIDCLoginOptions): Promise<void> {
  // Generate PKCE
  const codeVerifier = generateCodeVerifier();
  const codeChallenge = await generateCodeChallenge(codeVerifier);

  // Generate state and nonce
  const state = generateState();
  const nonce = generateNonce();

  // Store for later verification
  storePKCE(codeVerifier);
  storeState(state);
  storeNonce(nonce);

  // Store redirect URL if provided
  if (options.redirectUrl) {
    storeOIDCRedirectUrl(options.redirectUrl);
  }

  // Build and navigate to authorization URL
  const authUrl = buildAuthorizationUrl({
    authorizationEndpoint: options.authorizationEndpoint,
    clientId: options.clientId,
    redirectUri: options.redirectUri,
    scope: options.scope,
    codeChallenge,
    state,
    nonce,
    extraParams: options.extraParams,
  });

  window.location.href = authUrl;
}

/**
 * Handle OIDC callback
 * Validates state, exchanges code for tokens
 */
export interface HandleOIDCCallbackResult {
  success: boolean;
  tokens?: TokenResponse;
  error?: string;
}

export async function handleOIDCCallback(url: string = window.location.href): Promise<HandleOIDCCallbackResult> {
  const urlObj = new URL(url);
  const code = urlObj.searchParams.get('code');
  const state = urlObj.searchParams.get('state');
  const error = urlObj.searchParams.get('error');
  const errorDescription = urlObj.searchParams.get('error_description');

  // Check for OAuth error
  if (error) {
    return {
      success: false,
      error: errorDescription || error,
    };
  }

  // Validate code and state exist
  if (!code || !state) {
    return {
      success: false,
      error: 'Missing authorization code or state',
    };
  }

  // Validate state matches
  const storedState = getState();
  if (!storedState || storedState !== state) {
    return {
      success: false,
      error: 'Invalid state parameter',
    };
  }

  // Get code verifier
  const codeVerifier = getPKCE();
  if (!codeVerifier) {
    return {
      success: false,
      error: 'Missing PKCE verifier',
    };
  }

  try {
    // Exchange code for tokens via backend
    const tokens = await exchangeCodeWithBackend(code, state);

    // Validate nonce if ID token present
    if (tokens.id_token) {
      const nonce = getNonce();
      if (nonce && !validateNonce(tokens.id_token, nonce)) {
        return {
          success: false,
          error: 'Invalid nonce',
        };
      }
    }

    // Clear storage
    clearAllOIDCStorage();

    return {
      success: true,
      tokens,
    };
  } catch (err) {
    return {
      success: false,
      error: err instanceof Error ? err.message : 'Token exchange failed',
    };
  }
}

// ============================================
// Exports
// ============================================

export const oidc = {
  // Generation
  generateCodeVerifier,
  generateCodeChallenge,
  generateState,
  generateNonce,

  // Storage
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

  // URL & Flow
  buildAuthorizationUrl,
  initiateOIDCLogin,
  handleOIDCCallback,

  // Exchange & Validation
  exchangeCodeForTokens,
  exchangeCodeWithBackend,
  discoverOIDCConfig,
  fetchUserInfo,
  parseIDToken,
  validateNonce,
};

export default oidc;
