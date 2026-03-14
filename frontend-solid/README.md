# RustChat Frontend (Solid.js)

This is the Solid.js version of the RustChat frontend, built as part of the migration from Vue 3.

## Technology Stack

- **Framework**: Solid.js 1.9+
- **Build Tool**: Vite 7+
- **Language**: TypeScript 5.9+
- **Styling**: Tailwind CSS 4+
- **UI Components**: Kobalte (headless UI for Solid.js)
- **Routing**: @solidjs/router
- **Testing**: Vitest (unit), Playwright (E2E)
- **Linting**: ESLint with accessibility plugins

## Getting Started

### Prerequisites

- Node.js >= 20.0.0
- npm or pnpm

### Installation

```bash
cd frontend-solid
npm install
```

### Development

```bash
# Start development server
npm run dev

# Type check
npm run typecheck

# Run linter
npm run lint

# Format code
npm run format
```

### Testing

```bash
# Run unit tests
npm run test

# Run unit tests in watch mode
npm run test:watch

# Run E2E tests
npm run test:e2e

# Run E2E tests with UI
npm run test:e2e -- --ui
```

### Building

```bash
# Production build
npm run build

# Preview production build
npm run preview
```

## Verification Checklist

After setup, verify the following commands work:

- ✅ `npm install` - Installs all dependencies
- ✅ `npm run dev` - Starts development server at http://127.0.0.1:5173/
- ✅ `npm run build` - Creates production build in `dist/`
- ✅ `npm run test` - Runs Vitest unit tests
- ✅ `npm run lint` - ESLint with no errors
- ✅ `npm run typecheck` - TypeScript type checking passes

## Project Structure

```
frontend-solid/
├── src/
│   ├── components/
│   │   └── ui/           # Base UI components (Button, Input, Modal)
│   ├── hooks/            # Custom Solid.js hooks (placeholder)
│   ├── routes/           # Page components
│   │   ├── Home.tsx      # Demo page with components
│   │   ├── About.tsx     # About page
│   │   └── NotFound.tsx  # 404 page
│   ├── stores/           # Global state (Solid Stores)
│   │   └── theme.tsx     # Theme provider and hook
│   ├── styles/           # CSS and design tokens
│   │   ├── index.css     # Main CSS with Tailwind
│   │   └── tokens.css    # CSS variables and themes
│   ├── types/            # TypeScript types
│   ├── utils/            # Utility functions
│   │   └── cn.ts         # Tailwind class merging
│   ├── App.tsx           # Root component
│   └── index.tsx         # Entry point
├── e2e/                  # Playwright E2E tests
│   └── home.spec.ts      # Home page tests
├── tests/                # Vitest unit tests
│   ├── setup.ts          # Test setup
│   └── cn.test.ts        # cn utility tests
├── public/               # Static assets
│   └── vite.svg
└── ...config files
```

## Features

### Theming

The application supports multiple themes with CSS custom properties:
- Light (default)
- Dark
- Modern (green-tinted)
- Metallic (industrial)
- Futuristic (cyber)
- High Contrast (accessibility)
- Simple (minimal)
- Dynamic (vibrant)

Themes are stored in `localStorage` and persist across sessions.

### Accessibility

- Full keyboard navigation support
- ARIA attributes on all interactive elements
- Focus management and trapping in modals
- Screen reader announcements
- Reduced motion support
- High contrast theme for accessibility

### Path Aliases

The following path aliases are configured:

- `@/` → `src/`
- `@components/` → `src/components/`
- `@ui/` → `src/components/ui/`
- `@hooks/` → `src/hooks/`
- `@stores/` → `src/stores/`
- `@utils/` → `src/utils/`
- `@types/` → `src/types/`
- `@styles/` → `src/styles/`

## Migration Status

This is Phase F0 (Foundation & Setup) of the Vue 3 → Solid.js migration.

Completed:
- ✅ Project structure in `frontend-solid/`
- ✅ Solid.js 1.9 + Vite 7 + TypeScript 5.9
- ✅ Tailwind CSS 4 configuration with design tokens
- ✅ ESLint + Prettier with accessibility plugins
- ✅ Path aliases configured
- ✅ Vitest for unit testing
- ✅ Playwright for E2E testing
- ✅ Base UI components: Button, Input, Modal
- ✅ Dark mode and 8 theme variants
- ✅ Accessibility foundations (keyboard nav, ARIA, focus trapping)
- ✅ Theme store with localStorage persistence
- ✅ Demo page showcasing all components

Next: Phase F1 (Core Infrastructure)

## Browser Support

- Chrome/Edge 90+
- Firefox 90+
- Safari 15+
- Mobile Safari (iOS 15+)
- Chrome for Android

## License

Same as the main RustChat project.
