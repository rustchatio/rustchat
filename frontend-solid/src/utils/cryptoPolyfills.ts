function formatUuidV4(bytes: Uint8Array): string {
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

function fallbackRandomUUID(): `${string}-${string}-${string}-${string}-${string}` {
  const cryptoObj = typeof globalThis !== 'undefined' ? globalThis.crypto : undefined;

  if (cryptoObj?.getRandomValues) {
    return formatUuidV4(
      cryptoObj.getRandomValues(new Uint8Array(16))
    ) as `${string}-${string}-${string}-${string}-${string}`;
  }

  const bytes = new Uint8Array(16);
  for (let index = 0; index < bytes.length; index += 1) {
    bytes[index] = Math.floor(Math.random() * 256);
  }

  return formatUuidV4(bytes) as `${string}-${string}-${string}-${string}-${string}`;
}

export function installCryptoPolyfills() {
  const cryptoObj = typeof globalThis !== 'undefined' ? globalThis.crypto : undefined;
  if (!cryptoObj) return;

  const maybeCrypto = cryptoObj as Crypto & { randomUUID?: () => string };
  if (typeof maybeCrypto.randomUUID === 'function') return;

  maybeCrypto.randomUUID = fallbackRandomUUID;
}
