import { describe, expect, it } from 'vitest'
import { getDirectMessageCounterpartyId } from './directMessage'

describe('getDirectMessageCounterpartyId', () => {
  it('returns the other participant when the current user is first', () => {
    expect(getDirectMessageCounterpartyId('dm_me_you', 'me')).toBe('you')
  })

  it('returns the other participant when the current user is second', () => {
    expect(getDirectMessageCounterpartyId('dm_you_me', 'me')).toBe('you')
  })

  it('returns null for malformed channel names', () => {
    expect(getDirectMessageCounterpartyId('town-square', 'me')).toBeNull()
  })
})
