export function getDirectMessageCounterpartyId(channelName?: string | null, currentUserId?: string | null): string | null {
  if (!channelName || !currentUserId) {
    return null
  }

  const parts = channelName.split('_')
  if (parts.length !== 3 || parts[0] !== 'dm') {
    return null
  }

  if (parts[1] === currentUserId) {
    return parts[2] || null
  }

  if (parts[2] === currentUserId) {
    return parts[1] || null
  }

  return null
}
