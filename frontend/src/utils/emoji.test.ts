import { describe, expect, it } from 'vitest'
import { getPreferredEmojiName, getReactionEmojiKey } from './emoji'

describe('emoji reaction canonicalization', () => {
  it('collapses common thumbs-up aliases to one reaction key', () => {
    expect(getReactionEmojiKey(':+1:')).toBe('👍')
    expect(getReactionEmojiKey('thumbsup')).toBe('👍')
    expect(getReactionEmojiKey('👍')).toBe('👍')
  })

  it('chooses a stable API name for common glyph aliases', () => {
    expect(getPreferredEmojiName('👍')).toBe('+1')
    expect(getPreferredEmojiName('thumbsup')).toBe('+1')
    expect(getPreferredEmojiName(':+1:')).toBe('+1')
  })
})
