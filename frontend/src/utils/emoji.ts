/**
 * Emoji utility to map common names to Unicode characters
 */

const emojiMap: Record<string, string> = {
    'heart': '❤️',
    'heavy_heart_exclamation': '❣',
    'broken_heart': '💔',
    'two_hearts': '💕',
    'sparkling_heart': '💖',
    'heartpulse': '💗',
    'cupid': '💘',
    'blue_heart': '💙',
    'green_heart': '💚',
    'yellow_heart': '💛',
    'purple_heart': '💜',
    'black_heart': '🖤',
    '+1': '👍',
    'thumbsup': '👍',
    '-1': '👎',
    'thumbsdown': '👎',
    'smile': '😄',
    'smiley': '😃',
    'grinning': '😀',
    'blush': '😊',
    'wink': '😉',
    'heart_eyes': '😍',
    'kissing_heart': '😘',
    'laughing': '😆',
    'joy': '😂',
    'sweat_smile': '😅',
    'yum': '😋',
    'sunglasses': '😎',
    'ok_hand': '👌',
    'rocket': '🚀',
    'fire': '🔥',
    'tada': '🎉',
    'check': '✅',
    'cross': '❌',
    'warning': '⚠️',
    'eyes': '👀',
    'thinking_face': '🤔',
    'thinking': '🤔',
    'party_popper': '🎉',
    'clap': '👏',
    'pray': '🙏',
    'raised_hands': '🙌',
};

/**
 * Gets the Unicode character for a given emoji name.
 * Returns the original name if no match is found.
 */
export function getEmojiChar(name: string): string {
    // Strip colons if present: ":heart:" -> "heart"
    const cleanName = name.replace(/^:|:$/g, '').toLowerCase();
    return emojiMap[cleanName] || name;
}

/**
 * Replaces all :emoji_name: patterns in a string with Unicode characters.
 */
export function replaceEmojiNames(text: string): string {
    return text.replace(/:([a-z0-9_+-]+):/g, (match, name) => {
        const char = emojiMap[name.toLowerCase()];
        return char || match;
    });
}
