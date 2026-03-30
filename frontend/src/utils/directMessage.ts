export function getDirectMessageCounterpartyId(channelName?: string | null, currentUserId?: string | null): string | null {
  if (!channelName || !currentUserId) {
    return null
  }

  // Support canonical format: <id1>__<id2>
  if (channelName.includes('__')) {
    const parts = channelName.split('__')
    if (parts.length !== 2) return null
    if (parts[0] === currentUserId) return parts[1] || null
    if (parts[1] === currentUserId) return parts[0] || null
    return null
  }

  // Support legacy format: dm_<id1>_<id2>
  const parts = channelName.split('_')
  if (parts.length === 3 && parts[0] === 'dm') {
    if (parts[1] === currentUserId) return parts[2] || null
    if (parts[2] === currentUserId) return parts[1] || null
  }

  return null
}
