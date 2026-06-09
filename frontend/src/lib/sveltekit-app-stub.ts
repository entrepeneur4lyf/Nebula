// sv5ui's barrel index re-exports its SvelteKit-coupled `Link`, whose
// `import { page } from '$app/state'` must still resolve at bundle time even
// though the component is tree-shaken out of this plain-Vite build (Nebula
// navigates with Inertia's <Link>, never sv5ui's). The `$app/state` alias in
// vite.config.ts points here so the resolver is satisfied; both this stub and
// sv5ui's Link are dropped from the output by tree-shaking. The shape mirrors
// the `page.url` access sv5ui's Link performs for active-link detection, so
// nothing crashes even if the module were ever retained.
export const page = {
  url: new URL(
    typeof window === 'undefined' ? 'http://localhost/' : window.location.href,
  ),
}
