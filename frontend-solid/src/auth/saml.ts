// ============================================
// SAML Authentication Module
// ============================================

import { client } from '../api/client';

// ============================================
// Types
// ============================================

export interface SAMLConfig {
  enabled: boolean;
  idp_url: string;
  idp_entity_id: string;
  sp_entity_id: string;
  acs_url: string;
  slo_url?: string;
  name_id_format?: string;
  sign_requests: boolean;
  want_assertions_signed: boolean;
  certificate?: string;
  metadata_url?: string;
}

export interface SAMLAuthRequest {
  saml_request: string;
  relay_state?: string;
  sig_alg?: string;
  signature?: string;
}

export interface SAMLResponse {
  saml_response: string;
  relay_state?: string;
}

export interface SAMLLoginResponse {
  token: string;
  user: {
    id: string;
    username: string;
    email: string;
    display_name?: string;
    first_name?: string;
    last_name?: string;
    role: string;
  };
}

export interface SAMLError {
  error: string;
  error_description?: string;
}

// ============================================
// Storage Helpers
// ============================================

const RELAY_STATE_KEY = 'rustchat_saml_relay_state';
const REQUEST_ID_KEY = 'rustchat_saml_request_id';
const SAML_REDIRECT_KEY = 'rustchat_saml_redirect';

export function storeRelayState(relayState: string): void {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem(RELAY_STATE_KEY, relayState);
}

export function getRelayState(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(RELAY_STATE_KEY);
}

export function clearRelayState(): void {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem(RELAY_STATE_KEY);
}

export function storeRequestId(requestId: string): void {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem(REQUEST_ID_KEY, requestId);
}

export function getRequestId(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(REQUEST_ID_KEY);
}

export function clearRequestId(): void {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem(REQUEST_ID_KEY);
}

export function storeSAMLRedirectUrl(url: string): void {
  if (typeof window === 'undefined') return;
  sessionStorage.setItem(SAML_REDIRECT_KEY, url);
}

export function getSAMLRedirectUrl(): string | null {
  if (typeof window === 'undefined') return null;
  return sessionStorage.getItem(SAML_REDIRECT_KEY);
}

export function clearSAMLRedirectUrl(): void {
  if (typeof window === 'undefined') return;
  sessionStorage.removeItem(SAML_REDIRECT_KEY);
}

export function clearAllSAMLStorage(): void {
  clearRelayState();
  clearRequestId();
  clearSAMLRedirectUrl();
}

// ============================================
// Relay State Generation
// ============================================

/**
 * Generate a secure random relay state
 */
export function generateRelayState(): string {
  const array = new Uint8Array(16);
  crypto.getRandomValues(array);
  return Array.from(array, (byte) => byte.toString(16).padStart(2, '0')).join('');
}

// ============================================
// SAML Request Generation
// ============================================

export interface InitiateSAMLLoginOptions {
  idpUrl: string;
  redirectUrl?: string;
}

export interface SAMLRequestResponse {
  saml_request: string;
  relay_state: string;
  request_id: string;
  idp_url: string;
}

/**
 * Initiate SAML login by fetching SAML request from backend
 */
export async function initiateSAMLLogin(options: InitiateSAMLLoginOptions): Promise<void> {
  try {
    // Fetch SAML request from backend
    const response = await client.get<SAMLRequestResponse>('/auth/saml/request');
    const { saml_request, relay_state, request_id, idp_url } = response.data;

    // Store relay state and request ID for validation
    storeRelayState(relay_state);
    storeRequestId(request_id);

    // Store redirect URL if provided
    if (options.redirectUrl) {
      storeSAMLRedirectUrl(options.redirectUrl);
    }

    // Build form data for POST binding
    const form = document.createElement('form');
    form.method = 'POST';
    form.action = idp_url || options.idpUrl;
    form.style.display = 'none';

    // Add SAMLRequest field
    const samlInput = document.createElement('input');
    samlInput.type = 'hidden';
    samlInput.name = 'SAMLRequest';
    samlInput.value = saml_request;
    form.appendChild(samlInput);

    // Add RelayState field
    const relayInput = document.createElement('input');
    relayInput.type = 'hidden';
    relayInput.name = 'RelayState';
    relayInput.value = relay_state;
    form.appendChild(relayInput);

    // Submit form
    document.body.appendChild(form);
    form.submit();
    document.body.removeChild(form);
  } catch (error) {
    throw new Error('Failed to initiate SAML login');
  }
}

/**
 * Alternative: Redirect binding (GET)
 * Builds redirect URL with SAML request
 */
export async function buildSAMLRedirectUrl(idpUrl: string): Promise<string> {
  const response = await client.get<SAMLRequestResponse>('/auth/saml/request');
  const { saml_request, relay_state } = response.data;

  // Store relay state
  storeRelayState(relay_state);

  const params = new URLSearchParams({
    SAMLRequest: saml_request,
    RelayState: relay_state,
  });

  return `${idpUrl}?${params.toString()}`;
}

// ============================================
// SAML Response Handling
// ============================================

export interface HandleSAMLCallbackResult {
  success: boolean;
  data?: SAMLLoginResponse;
  error?: string;
}

/**
 * Parse SAML response from URL or form POST
 */
export function parseSAMLResponse(url: string = window.location.href): SAMLResponse | null {
  // Check for POST data in document
  if (typeof document !== 'undefined') {
    const samlInput = document.querySelector('input[name="SAMLResponse"]') as HTMLInputElement;
    const relayInput = document.querySelector('input[name="RelayState"]') as HTMLInputElement;

    if (samlInput?.value) {
      return {
        saml_response: samlInput.value,
        relay_state: relayInput?.value,
      };
    }
  }

  // Check URL parameters (for error cases)
  const urlObj = new URL(url);
  const error = urlObj.searchParams.get('error');
  const errorDescription = urlObj.searchParams.get('error_description');

  if (error) {
    throw new Error(errorDescription || error);
  }

  return null;
}

/**
 * Handle SAML callback
 * Validates relay state and exchanges SAML response for tokens
 */
export async function handleSAMLCallback(
  samlResponse?: string,
  relayState?: string
): Promise<HandleSAMLCallbackResult> {
  // Get SAML response from parameter or parse from URL
  let response = samlResponse;
  let state = relayState;

  if (!response) {
    const parsed = parseSAMLResponse();
    if (parsed) {
      response = parsed.saml_response;
      state = parsed.relay_state;
    }
  }

  if (!response) {
    return {
      success: false,
      error: 'Missing SAML response',
    };
  }

  // Validate relay state
  const storedState = getRelayState();
  if (!storedState || storedState !== state) {
    return {
      success: false,
      error: 'Invalid relay state',
    };
  }

  try {
    // Exchange SAML response for tokens via backend
    const result = await client.post<SAMLLoginResponse>('/auth/saml/callback', {
      saml_response: response,
      relay_state: state,
    });

    // Clear storage
    clearAllSAMLStorage();

    return {
      success: true,
      data: result.data,
    };
  } catch (error) {
    return {
      success: false,
      error: error instanceof Error ? error.message : 'SAML authentication failed',
    };
  }
}

// ============================================
// SAML Metadata
// ============================================

/**
 * Fetch SAML configuration from backend
 */
export async function getSAMLConfig(): Promise<SAMLConfig | null> {
  try {
    const response = await client.get<SAMLConfig>('/auth/saml/config');
    return response.data;
  } catch (error) {
    return null;
  }
}

/**
 * Check if SAML is enabled
 */
export async function isSAMLEnabled(): Promise<boolean> {
  const config = await getSAMLConfig();
  return config?.enabled ?? false;
}

// ============================================
// SAML Logout
// ============================================

export interface LogoutRequest {
  saml_logout_request: string;
  relay_state?: string;
}

/**
 * Initiate SAML Single Logout (SLO)
 */
export async function initiateSAMLLogout(): Promise<string | null> {
  try {
    const response = await client.post<LogoutRequest>('/auth/saml/logout');
    const { saml_logout_request, relay_state } = response.data;

    // Build logout URL
    const config = await getSAMLConfig();
    if (!config?.slo_url) return null;

    const params = new URLSearchParams({
      SAMLRequest: saml_logout_request,
    });

    if (relay_state) {
      params.append('RelayState', relay_state);
    }

    return `${config.slo_url}?${params.toString()}`;
  } catch (error) {
    return null;
  }
}

// ============================================
// SAML Assertion Parser (Basic)
// ============================================

/**
 * Decode base64 SAML assertion for debugging
 * Note: Actual parsing should be done server-side
 */
export function decodeSAMLResponse(encodedResponse: string): string {
  try {
    // SAML uses base64 encoding
    const decoded = atob(encodedResponse);
    return decoded;
  } catch {
    return '';
  }
}

/**
 * Extract basic info from SAML response (client-side only, for debugging)
 */
export function extractSAMLInfo(decodedResponse: string): {
  issuer?: string;
  subject?: string;
  issueInstant?: string;
} {
  const parser = new DOMParser();
  const doc = parser.parseFromString(decodedResponse, 'text/xml');

  return {
    issuer: doc.querySelector('Issuer')?.textContent || undefined,
    subject: doc.querySelector('Subject NameID')?.textContent || undefined,
    issueInstant: doc.documentElement.getAttribute('IssueInstant') || undefined,
  };
}

// ============================================
// Exports
// ============================================

export const saml = {
  // Storage
  storeRelayState,
  getRelayState,
  clearRelayState,
  storeRequestId,
  getRequestId,
  clearRequestId,
  storeSAMLRedirectUrl,
  getSAMLRedirectUrl,
  clearSAMLRedirectUrl,
  clearAllSAMLStorage,

  // Generation
  generateRelayState,

  // Flow
  initiateSAMLLogin,
  buildSAMLRedirectUrl,
  parseSAMLResponse,
  handleSAMLCallback,

  // Config
  getSAMLConfig,
  isSAMLEnabled,

  // Logout
  initiateSAMLLogout,

  // Utils
  decodeSAMLResponse,
  extractSAMLInfo,
};

export default saml;
