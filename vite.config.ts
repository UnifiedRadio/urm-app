import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// Tauri expects a fixed port; don't open browser automatically
const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: { '@': resolve(__dirname, 'src') },
  },
  clearScreen: false,
  server: {
    host: host ?? false,
    port: 1420,
    strictPort: true,
    fs: {
      // Allow importing urc-schema/data/devices.json as the shared device catalog source.
      allow: ['..'],
    },
    watch: {
      // Tell vite to ignore watching src-tauri
      ignored: ['**/src-tauri/**'],
    },
  },
  build: {
    // Tauri uses Chromium — target modern browsers only
    target: ['es2021', 'chrome105', 'safari15'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
  },
})
