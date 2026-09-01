/// <reference types="vitest/config" />
import path from 'node:path';
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { VitePWA } from 'vite-plugin-pwa';
import { fileURLToPath } from 'node:url';
import { storybookTest } from '@storybook/addon-vitest/vitest-plugin';
import { playwright } from '@vitest/browser-playwright';
const dirname = typeof __dirname !== 'undefined' ? __dirname : path.dirname(fileURLToPath(import.meta.url));

// More info at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon
export default defineConfig({
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'stellar': ['@stellar/stellar-sdk', '@stellar/freighter-api'],
          'ui': ['@radix-ui/react-dialog', '@radix-ui/react-select'],
          'query': ['@tanstack/react-query', '@tanstack/react-query-devtools'],
          'map': ['leaflet', 'react-leaflet', 'leaflet.markercluster'],
        },
      },
    },
    chunkSizeWarningLimit: 1000,
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
      },
    },
  },
  plugins: [react(), VitePWA({
    registerType: 'autoUpdate',
    workbox: {
      globPatterns: ['**/*.{js,css,html,ico,png,svg}'],
      runtimeCaching: [{
        urlPattern: /^https:\/\/.*\.stellar\.org\/.*/i,
        handler: 'NetworkFirst',
        options: {
          cacheName: 'stellar-api-cache',
          expiration: {
            maxEntries: 100,
            maxAgeSeconds: 60 * 60 * 24 * 7
          }
        }
      }, {
        urlPattern: /^https:\/\/.*\.ipfs\.io\/.*/i,
        handler: 'CacheFirst',
        options: {
          cacheName: 'ipfs-cache',
          expiration: {
            maxEntries: 50,
            maxAgeSeconds: 60 * 60 * 24 * 30
          }
        }
      }, {
        urlPattern: /\/api\/.*/i,
        handler: 'NetworkFirst',
        options: {
          cacheName: 'backend-api-cache',
          expiration: {
            maxEntries: 50,
            maxAgeSeconds: 60
          }
        }
      }]
    },
    includeAssets: ['favicon.ico', 'apple-touch-icon.png', 'masked-icon.svg'],
    manifest: {
      name: 'ProofFlow',
      short_name: 'ProofFlow',
      description: 'Decentralized verification and milestone settlement protocol',
      theme_color: '#000000',
      background_color: '#ffffff',
      display: 'standalone',
      icons: [{
        src: 'pwa-192x192.svg',
        sizes: '192x192',
        type: 'image/svg+xml'
      }, {
        src: 'pwa-512x512.svg',
        sizes: '512x512',
        type: 'image/svg+xml'
      }]
    }
  })],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
      '@/components': path.resolve(__dirname, './src/components'),
      '@/pages': path.resolve(__dirname, './src/pages'),
      '@/hooks': path.resolve(__dirname, './src/hooks'),
      '@/lib': path.resolve(__dirname, './src/lib'),
      '@/context': path.resolve(__dirname, './src/context'),
      '@/types': path.resolve(__dirname, './src/types'),
      '@/config': path.resolve(__dirname, './src/config'),
      '@/stories': path.resolve(__dirname, './src/stories'),
      '@/api': path.resolve(__dirname, './src/api'),
      '@/assets': path.resolve(__dirname, './src/assets'),
      '@/i18n': path.resolve(__dirname, './src/i18n'),
      '@/test': path.resolve(__dirname, './src/test')
    }
  },
  test: {
    projects: [{
      extends: true,
      test: {
        globals: true,
        environment: 'jsdom',
        setupFiles: ['./src/test/setup.tsx'],
        css: false,
        coverage: {
          provider: 'v8',
          reporter: ['text', 'json', 'html', 'lcov'],
          include: ['src/**/*.{ts,tsx}'],
          exclude: [
            'src/**/*.d.ts',
            'src/**/*.test.{ts,tsx}',
            'src/**/*.spec.{ts,tsx}',
            'src/test/**',
            'src/**/*.stories.tsx',
          ],
          lines: 85,
          functions: 85,
          branches: 85,
          statements: 85,
          perFile: true,
        },
      }
    }, {
      extends: true,
      plugins: [
      // The plugin will run tests for the stories defined in your Storybook config
      // See options at: https://storybook.js.org/docs/next/writing-tests/integrations/vitest-addon#storybooktest
      storybookTest({
        configDir: path.join(dirname, '.storybook')
      })],
      test: {
        name: 'storybook',
        browser: {
          enabled: true,
          headless: true,
          provider: playwright({}),
          instances: [{
            browser: 'chromium'
          }]
        }
      }
    }]
  }
});