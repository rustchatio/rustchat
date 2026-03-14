import { defineConfig } from 'vite';
import solid from 'vite-plugin-solid';
import path from 'path';

function resolveApiProxyTarget(): string {
  const apiUrl = process.env.VITE_API_URL;
  if (apiUrl) {
    try {
      return new URL(apiUrl).origin;
    } catch {
      // Ignore relative/non-URL values and fall through to the default target.
    }
  }

  return process.env.VITE_API_PROXY_TARGET || 'http://127.0.0.1:3000';
}

const apiProxyTarget = resolveApiProxyTarget();

export default defineConfig({
  plugins: [solid()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@components': path.resolve(__dirname, './src/components'),
      '@ui': path.resolve(__dirname, './src/components/ui'),
      '@hooks': path.resolve(__dirname, './src/hooks'),
      '@stores': path.resolve(__dirname, './src/stores'),
      '@utils': path.resolve(__dirname, './src/utils'),
      '@types': path.resolve(__dirname, './src/types'),
      '@styles': path.resolve(__dirname, './src/styles'),
    },
  },
  build: {
    target: 'esnext',
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id) return;

          if (id.includes('node_modules')) {
            if (id.includes('/solid-js/') || id.includes('/@solidjs/')) {
              return 'vendor-solid';
            }

            if (
              id.includes('/highlight.js/') ||
              id.includes('/marked/') ||
              id.includes('/dompurify/')
            ) {
              return 'vendor-markdown';
            }

            if (id.includes('/@kobalte/')) {
              return 'vendor-ui';
            }

            return 'vendor';
          }
        },
      },
    },
  },
  server: {
    port: 5173,
    host: '127.0.0.1',
    proxy: {
      '/api': {
        target: apiProxyTarget,
        changeOrigin: true,
        ws: true,
      },
    },
  },
});
