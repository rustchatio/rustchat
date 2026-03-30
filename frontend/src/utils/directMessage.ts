export function getDirectMessageCounterpartyId(channelName?: string | null, currentUserId?: string | null): string | null {
  if (!channelName || !currentUserId) {
    return null
  }

  // Support both canonical "<id1>__<id2>" and legacy/v1 "dm_<id1>_<id2>" formats
  if (channelName.includes('__')) {
    const parts = channelName.split('__')
    if (parts.length !== 2) return null
    return parts[0] === currentUserId ? parts[1] : parts[1] === currentUserId ? parts[0] : null
  }

  const parts = channelName.split('_')
  if (parts.length === 3 && parts[0] === 'dm') {
    return parts[1] === currentUserId ? parts[2] : parts[2] === currentUserId ? parts[1] : null
  }

  return null
}
