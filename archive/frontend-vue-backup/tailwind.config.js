/** @type {import('tailwindcss').Config} */
export default {
  theme: {
    extend: {
      colors: {
        primary: 'var(--color-primary)',
        secondary: 'var(--color-secondary)',
        'bg-main': 'var(--color-bg-main)',
        'bg-sidebar': 'var(--color-bg-sidebar)',
        'text-base': 'var(--color-text-base)',
        accent: 'var(--color-accent)',
        surface: 'var(--theme-surface)',
        'surface-muted': 'var(--theme-surface-muted)',
        text: 'var(--theme-text)',
        background: 'var(--theme-bg)',
        border: 'var(--theme-border)',
      },
      fontFamily: {
        chat: ['var(--chat-font-family)'],
        inter: ['var(--font-inter)'],
        figtree: ['var(--font-figtree)'],
        'jetbrains-mono': ['var(--font-jetbrains-mono)'],
        quicksand: ['var(--font-quicksand)'],
        montserrat: ['var(--font-montserrat)'],
        'source-sans-3': ['var(--font-source-sans-3)'],
        nunito: ['var(--font-nunito)'],
        manrope: ['var(--font-manrope)'],
        'work-sans': ['var(--font-work-sans)'],
        'ibm-plex-sans': ['var(--font-ibm-plex-sans)'],
      },
      fontSize: {
        'chat-xs': ['var(--fs-xs)', { lineHeight: 'var(--lh-xs)' }],
        'chat-sm': ['var(--fs-sm)', { lineHeight: 'var(--lh-sm)' }],
        'chat-base': ['var(--fs-base)', { lineHeight: 'var(--lh-base)' }],
        'chat-md': ['var(--fs-md)', { lineHeight: 'var(--lh-md)' }],
        'chat-lg': ['var(--fs-lg)', { lineHeight: 'var(--lh-lg)' }],
      },
      boxShadow: {
        theme: 'var(--theme-shadow)',
      },
    },
  },
}
