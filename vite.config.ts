import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [
    svelte(),
    {
      name: 'offline-error-messages',
      renderChunk(code) {
        // Svelte's error links are documentation strings, never runtime requests.
        return code.replace(/https:\/\/svelte\.dev\/e\//g, 'Svelte error: ');
      },
    },
  ],
  clearScreen: false,
  build: { target: 'chrome105', sourcemap: false, modulePreload: false },
});
