import {
  createContext,
  useContext,
  createSignal,
  createEffect,
  ParentComponent,
  Accessor,
} from 'solid-js';
import type { Theme } from '../types';

// ============================================
// Theme Configuration
// ============================================

export const AVAILABLE_THEMES: Theme[] = [
  'light',
  'dark',
  'modern',
  'metallic',
  'futuristic',
  'high-contrast',
  'simple',
  'dynamic',
];

const STORAGE_KEY = 'rustchat-theme';

// ============================================
// Theme Context
// ============================================

interface ThemeContextValue {
  theme: Accessor<Theme>;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  availableThemes: Theme[];
  isDark: Accessor<boolean>;
}

const ThemeContext = createContext<ThemeContextValue>();

// ============================================
// Helper Functions
// ============================================

function getInitialTheme(): Theme {
  // Check localStorage first
  if (typeof window !== 'undefined') {
    const stored = localStorage.getItem(STORAGE_KEY) as Theme | null;
    if (stored && AVAILABLE_THEMES.includes(stored)) {
      return stored;
    }

    // Check system preference for dark mode
    if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
      return 'dark';
    }
  }

  return 'light';
}

function isDarkTheme(theme: Theme): boolean {
  return ['dark', 'futuristic', 'high-contrast', 'dynamic'].includes(theme);
}

// ============================================
// Theme Provider Component
// ============================================

export const ThemeProvider: ParentComponent = props => {
  const [theme, setThemeSignal] = createSignal<Theme>(getInitialTheme());

  // Apply theme to document
  createEffect(() => {
    const currentTheme = theme();

    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', currentTheme);

      // Store preference
      localStorage.setItem(STORAGE_KEY, currentTheme);
    }
  });

  // Listen for system theme changes
  createEffect(() => {
    if (typeof window === 'undefined') return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');

    const handleChange = (e: MediaQueryListEvent) => {
      // Only auto-switch if user hasn't manually set a theme this session
      const hasManualPreference = localStorage.getItem(STORAGE_KEY) !== null;
      if (!hasManualPreference) {
        setThemeSignal(e.matches ? 'dark' : 'light');
      }
    };

    mediaQuery.addEventListener('change', handleChange);

    // Cleanup
    return () => {
      mediaQuery.removeEventListener('change', handleChange);
    };
  });

  const setTheme = (newTheme: Theme) => {
    setThemeSignal(newTheme);
  };

  const toggleTheme = () => {
    const current = theme();
    const newTheme = isDarkTheme(current) ? 'light' : 'dark';
    setThemeSignal(newTheme);
  };

  const isDark = () => isDarkTheme(theme());

  const value: ThemeContextValue = {
    theme,
    setTheme,
    toggleTheme,
    availableThemes: AVAILABLE_THEMES,
    isDark,
  };

  return <ThemeContext.Provider value={value}>{props.children}</ThemeContext.Provider>;
};

// ============================================
// Theme Hook
// ============================================

export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);

  if (!context) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }

  return context;
}

export default useTheme;
