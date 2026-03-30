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

  it('supports canonical direct message names with double underscores', () => {
    expect(getDirectMessageCounterpartyId('me__you', 'me')).toBe('you')
    expect(getDirectMessageCounterpartyId('you__me', 'me')).toBe('you')
  })

  it('still supports legacy dm_ prefixed names', () => {
    expect(getDirectMessageCounterpartyId('dm_me_you', 'me')).toBe('you')
    expect(getDirectMessageCounterpartyId('dm_you_me', 'me')).toBe('you')
  })

  it('returns null when the user is not part of the name', () => {
    expect(getDirectMessageCounterpartyId('alice__bob', 'charlie')).toBeNull()
    expect(getDirectMessageCounterpartyId('dm_alice_bob', 'charlie')).toBeNull()
  })
})
