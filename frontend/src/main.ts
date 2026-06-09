import './app.css'
import { createInertiaApp, router, type ResolvedComponent } from '@inertiajs/svelte'
import { hydrate, mount } from 'svelte'
import Layout from './lib/Layout.svelte'

// Forward the session's CSRF token on every mutating Inertia visit.
// Suprnova rotates the token on login/logout, so read it fresh per request:
// prefer the live XSRF-TOKEN cookie (set by the CSRF middleware and accepted
// back as X-XSRF-TOKEN), falling back to the <meta name="csrf-token"> tag the
// server template rendered with the page.
function csrfToken(): string | undefined {
  const cookie = document.cookie
    .split('; ')
    .find((entry) => entry.startsWith('XSRF-TOKEN='))
  if (cookie) {
    return decodeURIComponent(cookie.slice('XSRF-TOKEN='.length))
  }
  const meta = document.querySelector('meta[name="csrf-token"]')
  return meta?.getAttribute('content') ?? undefined
}

router.on('before', (event) => {
  const { method, headers } = event.detail.visit
  // Inertia's Method type is get/post/put/patch/delete; only the
  // mutating methods need the token.
  if (method === 'get') return
  const token = csrfToken()
  if (token) headers['X-XSRF-TOKEN'] = token
})

createInertiaApp({
  resolve: (name) => {
    const pages = import.meta.glob<ResolvedComponent>('./pages/**/*.svelte', {
      eager: true,
    })
    return pages[`./pages/${name}.svelte`]
  },
  // Persistent default layout. Inertia 3's `layout` option is a callback
  // `(name, page) => Component` consumed as App's `defaultLayout`; a page can
  // opt out or override by exporting its own `layout` from <script module>.
  // (Mutating `page.default.layout` in `resolve` is not an option here:
  // eager `import.meta.glob` returns frozen module namespace objects.)
  layout: () => Layout,
  setup({ el, App, props }) {
    if (el?.hasAttribute('data-server-rendered')) {
      hydrate(App, { target: el, props })
    } else {
      mount(App, { target: el!, props })
    }
  },
})
