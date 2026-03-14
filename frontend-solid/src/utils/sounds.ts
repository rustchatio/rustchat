// ============================================
// Notification Sounds
// ============================================

// Simple beep sound using Web Audio API
let audioContext: AudioContext | null = null;

// ============================================
// Audio Context Management
// ============================================

function getAudioContext(): AudioContext | null {
  if (typeof window === 'undefined') return null;
  
  if (!audioContext) {
    try {
      audioContext = new (window.AudioContext || (window as unknown as { webkitAudioContext: typeof AudioContext }).webkitAudioContext)();
    } catch {
      console.warn('[Sounds] Web Audio API not supported');
      return null;
    }
  }
  
  return audioContext;
}

// ============================================
// Sound Synthesis
// ============================================

function playTone(frequency: number, duration: number, type: OscillatorType = 'sine'): void {
  const ctx = getAudioContext();
  if (!ctx) return;

  // Resume context if suspended (browser policy)
  if (ctx.state === 'suspended') {
    ctx.resume().catch(() => {});
  }

  const oscillator = ctx.createOscillator();
  const gainNode = ctx.createGain();

  oscillator.connect(gainNode);
  gainNode.connect(ctx.destination);

  oscillator.frequency.value = frequency;
  oscillator.type = type;

  // Envelope
  gainNode.gain.setValueAtTime(0, ctx.currentTime);
  gainNode.gain.linearRampToValueAtTime(0.3, ctx.currentTime + 0.01);
  gainNode.gain.exponentialRampToValueAtTime(0.01, ctx.currentTime + duration);

  oscillator.start(ctx.currentTime);
  oscillator.stop(ctx.currentTime + duration);
}

// ============================================
// Sound Effects
// ============================================

export const Sounds = {
  /**
   * New message notification (gentle ping)
   */
  newMessage(): void {
    playTone(880, 0.15, 'sine'); // A5
    setTimeout(() => playTone(1100, 0.1, 'sine'), 80); // C#6
  },

  /**
   * Mention notification (more noticeable)
   */
  mention(): void {
    playTone(880, 0.1, 'sine');
    setTimeout(() => playTone(1100, 0.1, 'sine'), 100);
    setTimeout(() => playTone(1320, 0.2, 'sine'), 200); // E6
  },

  /**
   * Direct message notification
   */
  directMessage(): void {
    playTone(523, 0.1, 'sine'); // C5
    setTimeout(() => playTone(659, 0.1, 'sine'), 100); // E5
    setTimeout(() => playTone(784, 0.2, 'sine'), 200); // G5
  },

  /**
   * Send message confirmation
   */
  send(): void {
    playTone(440, 0.05, 'sine'); // A4
  },

  /**
   * Error sound
   */
  error(): void {
    playTone(200, 0.3, 'sawtooth');
  },

  /**
   * Connection restored
   */
  connected(): void {
    playTone(523, 0.1, 'sine');
    setTimeout(() => playTone(784, 0.2, 'sine'), 100);
  },

  /**
   * Disconnection warning
   */
  disconnected(): void {
    playTone(300, 0.2, 'triangle');
  },
};

// ============================================
// Settings
// ============================================

const SOUNDS_ENABLED_KEY = 'sounds_enabled';

export const SoundSettings = {
  /**
   * Check if sounds are enabled
   */
  isEnabled(): boolean {
    if (typeof window === 'undefined') return false;
    return localStorage.getItem(SOUNDS_ENABLED_KEY) !== 'false';
  },

  /**
   * Enable/disable sounds
   */
  setEnabled(enabled: boolean): void {
    localStorage.setItem(SOUNDS_ENABLED_KEY, enabled ? 'true' : 'false');
  },

  /**
   * Toggle sounds
   */
  toggle(): boolean {
    const newValue = !this.isEnabled();
    this.setEnabled(newValue);
    return newValue;
  },
};

// ============================================
// Safe Play Helpers (respect settings)
// ============================================

export function playNewMessageSound(): void {
  if (SoundSettings.isEnabled()) {
    Sounds.newMessage();
  }
}

export function playMentionSound(): void {
  if (SoundSettings.isEnabled()) {
    Sounds.mention();
  }
}

export function playDirectMessageSound(): void {
  if (SoundSettings.isEnabled()) {
    Sounds.directMessage();
  }
}

export function playSendSound(): void {
  if (SoundSettings.isEnabled()) {
    Sounds.send();
  }
}

export function playErrorSound(): void {
  if (SoundSettings.isEnabled()) {
    Sounds.error();
  }
}

export function playConnectedSound(): void {
  if (SoundSettings.isEnabled()) {
    Sounds.connected();
  }
}

export function playDisconnectedSound(): void {
  if (SoundSettings.isEnabled()) {
    Sounds.disconnected();
  }
}
