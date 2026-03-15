export type AvatarSize = 'sm' | 'md' | 'lg' | 'xl' | number;

function normalizeAvatarSize(size: AvatarSize): string {
  if (typeof size === 'number') {
    return String(Math.max(16, Math.min(512, Math.floor(size))));
  }

  switch (size) {
    case 'sm':
      return '32';
    case 'md':
      return '64';
    case 'lg':
      return '128';
    case 'xl':
      return '256';
    default:
      return '64';
  }
}

export function avatarSizedUrl(url: string | null | undefined, size: AvatarSize = 'md'): string | undefined {
  if (!url) return undefined;

  const normalized = normalizeAvatarSize(size);
  const separator = url.includes('?') ? '&' : '?';
  return `${url}${separator}size=${normalized}`;
}

