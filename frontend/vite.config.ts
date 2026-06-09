import { fileURLToPath } from 'node:url'
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [tailwindcss(), svelte()],
  resolve: {
    alias: {
      // sv5ui's barrel reaches its SvelteKit-only Link ($app/state) for import
      // resolution. The stub alias resolves the module; nothing imports Link so
      // it remains inert in the bundle (Inertia <Link> owns navigation).
      '$app/state': fileURLToPath(
        new URL('./src/lib/sveltekit-app-stub.ts', import.meta.url),
      ),
    },
  },
  server: {
    // `suprnova serve` sets VITE_PORT to the port it resolved (the
    // distinctive 5765 default, or a scanned free port). Falling back to
    // 5765 keeps a bare `npm run dev` off the squatted 5173.
    port: Number(process.env.VITE_PORT) || 5765,
    strictPort: true,
    cors: true,
  },
  build: {
    outDir: '../public/assets',
    manifest: true,
    rollupOptions: {
      input: 'src/main.ts',
    },
  },
})
