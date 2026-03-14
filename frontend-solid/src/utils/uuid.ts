// ============================================
// UUID Utilities
// ============================================

function formatUuidV4(bytes: Uint8Array): string {
  // RFC 4122 version and variant bits.
  bytes[6] = (bytes[6] & 0x0f) | 0x40;
  bytes[8] = (bytes[8] & 0x3f) | 0x80;

  const hex = Array.from(bytes, (b) => b.toString(16).padStart(2, '0')).join('');
  return [
    hex.slice(0, 8),
    hex.slice(8, 12),
    hex.slice(12, 16),
    hex.slice(16, 20),
    hex.slice(20, 32),
  ].join('-');
}

export function generateUUID(): string {
  const cryptoObj = typeof globalThis !== 'undefined' ? globalThis.crypto : undefined;
  const nativeRandomUUID = (cryptoObj as (Crypto & { randomUUID?: () => string }) | undefined)
    ?.randomUUID;

  if (typeof nativeRandomUUID === 'function') {
    return nativeRandomUUID.call(cryptoObj);
  }

  if (cryptoObj?.getRandomValues) {
    return formatUuidV4(cryptoObj.getRandomValues(new Uint8Array(16)));
  }

  const bytes = new Uint8Array(16);
  for (let i = 0; i < bytes.length; i += 1) {
    bytes[i] = Math.floor(Math.random() * 256);
  }
  return formatUuidV4(bytes);
}

