// ============================================
// Authentication Module Exports
// ============================================

// OIDC exports
export {
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

  // Types
  type OIDCConfig,
  type OIDCAuthParams,
  type TokenResponse,
  type OIDCUserInfo,
  type AuthorizationUrlOptions,
  type TokenExchangeParams,
  type InitiateOIDCLoginOptions,
  type HandleOIDCCallbackResult,
} from './oidc';
export { default as oidc } from './oidc';

// SAML exports
export {
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

  // Types
  type SAMLConfig,
  type SAMLAuthRequest,
  type SAMLResponse,
  type SAMLLoginResponse,
  type SAMLError,
  type InitiateSAMLLoginOptions,
  type SAMLRequestResponse,
  type HandleSAMLCallbackResult,
  type LogoutRequest,
} from './saml';
export { default as saml } from './saml';
