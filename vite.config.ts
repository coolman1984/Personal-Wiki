import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { fileURLToPath } from 'node:url';

// Tauri serves the built SPA from disk, so relative asset paths and a fixed dev
// port are what the desktop shell expects.
export default defineConfig({
  plugins: [svelte()],
  base: './',
  resolve: {
    alias: {
      $lib: fileURLToPath(new URL('./src/lib', import.meta.url)),
      $components: fileURLToPath(new URL('./src/components', import.meta.url))
    }
  },
  clearScreen: false,
  server: { port: 5183, strictPort: true },
  build: { target: 'es2022', outDir: 'dist', emptyOutDir: true, sourcemap: true }
});
