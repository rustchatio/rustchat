import { createSignal, For, Show, onMount } from 'solid-js';

interface EmojiPickerProps {
  onSelect: (emoji: string) => void;
  onClose: () => void;
}

const EMOJI_CATEGORIES = {
  recent: 'Recently Used',
  smileys: 'Smileys',
  people: 'People',
  nature: 'Nature',
  food: 'Food',
};

const COMMON_EMOJIS: Record<string, string[]> = {
  recent: [],
  smileys: ['😀', '😃', '😄', '😁', '😆', '😅', '😂', '🤣', '😊', '😇', '🙂', '🙃', '😉', '😌', '😍', '🥰', '😘', '😗', '😙', '😚', '😋', '😛', '😝', '😜', '🤪', '🤨', '🧐', '🤓', '😎', '🤩', '🥳', '😏', '😒', '😞', '😔', '😟', '😕', '🙁', '☹️', '😣', '😖', '😫', '😩', '🥺', '😢', '😭', '😤', '😠', '😡', '🤬', '🤯', '😳', '🥵', '🥶', '😱', '😨', '😰', '😥', '😓', '🤗', '🤔', '🤭', '🤫', '🤥', '😶', '😐', '😑', '😬', '🙄', '😯', '😦', '😧', '😮', '😲', '🥱', '😴', '🤤', '😪', '😵', '🤐', '🥴', '🤢', '🤮', '🤧', '😷', '🤒', '🤕', '🤑', '🤠', '😈', '👿', '👹', '👺', '🤡', '💩', '👻', '💀', '☠️', '👽', '👾', '🤖', '🎃', '😺', '😸', '😹', '😻', '😼', '😽', '🙀', '😿', '😾'],
  people: ['👋', '🤚', '🖐️', '✋', '🖖', '👌', '🤌', '🤏', '✌️', '🤞', '🤟', '🤘', '🤙', '👈', '👉', '👆', '🖕', '👇', '☝️', '👍', '👎', '✊', '👊', '🤛', '🤜', '👏', '🙌', '👐', '🤲', '🤝', '🙏', '✍️', '💅', '🤳', '💪', '🦾', '🦿', '🦵', '🦶', '👂', '🦻', '👃', '🧠', '🫀', '🫁', '🦷', '🦴', '👀', '👁️', '👅', '👄', '👶', '🧒', '👦', '👧', '🧑', '👱', '👨', '🧔', '👨‍🦰', '👨‍🦱', '👨‍🦳', '👨‍🦲', '👩', '👩‍🦰', '👩‍🦱', '👩‍🦳', '👩‍🦲', '👱‍♀️', '👱‍♂️', '🧓', '👴', '👵'],
  nature: ['🐶', '🐱', '🐭', '🐹', '🐰', '🦊', '🐻', '🐼', '🐨', '🐯', '🦁', '🐮', '🐷', '🐽', '🐸', '🐵', '🙈', '🙉', '🙊', '🐒', '🐔', '🐧', '🐦', '🐤', '🐣', '🐥', '🦆', '🦅', '🦉', '🦇', '🐺', '🐗', '🐴', '🦄', '🐝', '🐛', '🦋', '🐌', '🐞', '🐜', '🦟', '🦗', '🕷️', '🕸️', '🦂', '🐢', '🐍', '🦎', '🦖', '🦕', '🐙', '🦑', '🦐', '🦞', '🦀', '🐡', '🐠', '🐟', '🐬', '🐳', '🐋', '🦈', '🐊', '🐅', '🐆', '🦓', '🦍', '🦧', '🐘', '🦛', '🦏', '🐪', '🐫', '🦒', '🦘', '🐃', '🐂', '🐄', '🐎', '🐖', '🐏', '🐑', '🦙', '🐐', '🦌', '🐕', '🐩', '🦮', '🐕‍🦺', '🐈', '🐈‍⬛', '🐓', '🦃', '🦚', '🦜', '🦢', '🦩', '🕊️', '🐇', '🦝', '🦨', '🦡', '🦦', '🦥', '🐁', '🐀', '🐿️', '🦔'],
  food: ['🍏', '🍎', '🍐', '🍊', '🍋', '🍌', '🍉', '🍇', '🍓', '🫐', '🍈', '🍒', '🍑', '🍍', '🥝', '🍅', '🍆', '🥑', '🥦', '🥬', '🥒', '🌶️', '🫑', '🌽', '🥕', '🫒', '🧄', '🧅', '🥔', '🍠', '🥐', '🥯', '🍞', '🥖', '🥨', '🧀', '🥚', '🍳', '🧈', '🥞', '🧇', '🥓', '🥩', '🍗', '🍖', '🦴', '🌭', '🍔', '🍟', '🍕', '🫓', '🥪', '🥙', '🧆', '🌮', '🌯', '🫔', '🥗', '🥘', '🫕', '🥫', '🍝', '🍜', '🍲', '🍛', '🍣', '🍱', '🥟', '🦪', '🍤', '🍙', '🍚', '🍘', '🍥', '🥠', '🥮', '🍢', '🍡', '🍧', '🍨', '🍦', '🥧', '🧁', '🍰', '🎂', '🍮', '🍭', '🍬', '🍫', '🍿', '🍩', '🍪', '🌰', '🥜', '🍯', '🥛', '🍼', '🫖', '☕', '🍵', '🧃', '🥤', '🧋', '🍶', '🍺', '🍻', '🥂', '🍷', '🥃', '🍸', '🍹', '🧉', '🍾'],
};

export function EmojiPicker(props: EmojiPickerProps) {
  const [activeCategory, setActiveCategory] = createSignal('smileys');
  const [, setRecentEmojis] = createSignal<string[]>([]);
  const [searchQuery, setSearchQuery] = createSignal('');

  onMount(() => {
    // Load recent emojis from localStorage
    const stored = localStorage.getItem('recent_emojis');
    if (stored) {
      setRecentEmojis(JSON.parse(stored));
    }
  });

  const handleEmojiSelect = (emoji: string) => {
    // Add to recent
    setRecentEmojis(prev => {
      const updated = [emoji, ...prev.filter(e => e !== emoji)].slice(0, 20);
      localStorage.setItem('recent_emojis', JSON.stringify(updated));
      return updated;
    });
    props.onSelect(emoji);
  };

  const filteredEmojis = () => {
    const query = searchQuery().toLowerCase();
    if (!query) return COMMON_EMOJIS[activeCategory()] || [];
    
    // Simple search - in real app would search by name
    return Object.values(COMMON_EMOJIS).flat().slice(0, 50);
  };

  return (
    <div class="absolute bottom-full left-3 right-3 z-40 mb-2 max-h-[360px] rounded-xl border border-border-1 bg-bg-surface-1 shadow-xl sm:left-0 sm:right-auto sm:w-96">
      <div class="flex items-center gap-2 border-b border-border-1 p-2">
        <input
          type="text"
          placeholder="Search emoji..."
          value={searchQuery()}
          onInput={(e) => setSearchQuery(e.currentTarget.value)}
          class="h-9 w-full rounded-md border border-border-1 bg-bg-app px-3 text-sm text-text-1 placeholder:text-text-3 focus:outline-none focus:ring-2 focus:ring-brand/40"
        />
        <button
          class="inline-flex h-9 w-9 items-center justify-center rounded-md border border-border-1 text-text-2 transition-colors hover:bg-bg-surface-2 hover:text-text-1"
          onClick={props.onClose}
          type="button"
          aria-label="Close emoji picker"
        >
          ×
        </button>
      </div>
      
      <Show when={!searchQuery()}>
        <div class="flex items-center gap-1 overflow-x-auto border-b border-border-1 p-2">
          {Object.entries(EMOJI_CATEGORIES).map(([key, label]) => (
            <button
              class={`inline-flex h-8 min-w-8 items-center justify-center rounded-md border px-2 text-sm transition-colors ${
                activeCategory() === key
                  ? 'border-brand/50 bg-brand/15 text-text-1'
                  : 'border-border-1 text-text-2 hover:bg-bg-surface-2 hover:text-text-1'
              }`}
              onClick={() => setActiveCategory(key)}
              title={label}
              type="button"
            >
              {COMMON_EMOJIS[key]?.[0] || '🔤'}
            </button>
          ))}
        </div>
      </Show>

      <div class="grid max-h-64 grid-cols-8 gap-1 overflow-y-auto p-2">
        <For each={filteredEmojis()}>
          {(emoji) => (
            <button
              class="inline-flex h-8 w-8 items-center justify-center rounded-md text-lg transition-colors hover:bg-bg-surface-2"
              onClick={() => handleEmojiSelect(emoji)}
              title={emoji}
              type="button"
            >
              {emoji}
            </button>
          )}
        </For>
      </div>
    </div>
  );
}
