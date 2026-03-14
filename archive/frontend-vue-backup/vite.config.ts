import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vite.dev/config/
export default defineConfig({
  plugins: [vue()],
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (!id) {
            return
          }

          if (id.includes('node_modules')) {
            if (
              id.includes('/vue/') ||
              id.includes('/pinia/') ||
              id.includes('/vue-router/') ||
              id.includes('/@vue/')
            ) {
              return 'vendor-vue'
            }

            if (
              id.includes('/highlight.js/') ||
              id.includes('/marked/') ||
              id.includes('/dompurify/')
            ) {
              return 'vendor-markdown'
            }

            if (id.includes('/@vueuse/')) {
              return 'vendor-vueuse'
            }

            return 'vendor'
          }

        },
      },
    },
  },
})
