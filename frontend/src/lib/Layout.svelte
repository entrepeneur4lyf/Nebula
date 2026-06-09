<script lang="ts">
  import type { Snippet } from 'svelte'
  import { Link, page, router } from '@inertiajs/svelte'
  import { ModeWatcher } from 'mode-watcher'
  import { Alert, Button, DropdownMenu, ThemeModeButton } from 'sv5ui'
  import type { DropdownMenuItem } from 'sv5ui'

  let { children }: { children: Snippet } = $props()

  interface AuthUser {
    id: number
    name: string
    email: string
  }

  // `page` in @inertiajs/svelte 3 is a $state-backed reactive object (not a
  // store), so plain property access stays reactive across visits.
  const user = $derived(
    (page.props.auth as { user?: AuthUser | null } | undefined)?.user ?? null,
  )

  // Inertia v3 carries one-shot flash data on the page object itself
  // (`page.flash`), not inside `props`.
  const flash = $derived(
    (page.flash as Record<string, unknown> | undefined) ?? {},
  )
  const flashSuccess = $derived(
    typeof flash.success === 'string' ? flash.success : null,
  )
  const flashError = $derived(
    typeof flash.error === 'string' ? flash.error : null,
  )

  const userMenuItems: DropdownMenuItem[] = [
    {
      label: 'Profile',
      icon: 'lucide:user',
      onSelect: () => router.visit('/profile'),
    },
    { type: 'separator' },
    {
      label: 'Log out',
      icon: 'lucide:log-out',
      color: 'error',
      onSelect: () => router.post('/logout'),
    },
  ]
</script>

<ModeWatcher defaultMode="dark" />

<div class="flex min-h-screen flex-col bg-surface text-on-surface">
  <header class="border-b border-outline-variant bg-surface-container-low">
    <nav class="mx-auto flex h-14 w-full max-w-5xl items-center gap-6 px-4">
      <Link href="/" class="flex items-center gap-2">
        <svg
          class="size-6 text-primary"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          aria-hidden="true"
        >
          <path
            d="M12 3c-1.2 3.6-2.4 4.8-6 6 3.6 1.2 4.8 2.4 6 6 1.2-3.6 2.4-4.8 6-6-3.6-1.2-4.8-2.4-6-6z"
          />
          <circle cx="19" cy="5" r="1" fill="currentColor" stroke="none" />
          <circle cx="5" cy="19" r="1" fill="currentColor" stroke="none" />
        </svg>
        <span class="text-lg font-semibold tracking-tight">Nebula</span>
      </Link>

      {#if user}
        <Link
          href="/dashboard"
          class="text-sm font-medium text-on-surface-variant transition-colors hover:text-on-surface"
        >
          Dashboard
        </Link>
      {/if}

      <div class="ml-auto flex items-center gap-2">
        <ThemeModeButton />

        {#if user}
          <DropdownMenu items={userMenuItems} align="end">
            {#snippet children({ props })}
              <Button
                {...props}
                variant="ghost"
                color="surface"
                trailingIcon="lucide:chevron-down"
                label={user?.name ?? 'Account'}
              />
            {/snippet}
          </DropdownMenu>
        {:else}
          <Link
            href="/login"
            class="text-sm font-medium text-on-surface-variant transition-colors hover:text-on-surface"
          >
            Log in
          </Link>
          <Link
            href="/register"
            class="rounded-lg bg-primary px-3 py-1.5 text-sm font-medium text-on-primary transition-opacity hover:opacity-90"
          >
            Register
          </Link>
        {/if}
      </div>
    </nav>
  </header>

  <!--
    Flash region renders human-ready copy only. Machine `status` keys (e.g.
    `invalid-or-expired`) are mapped to copy by the page that consumes them,
    never echoed raw here.
  -->
  {#if flashSuccess || flashError}
    <div class="mx-auto w-full max-w-5xl space-y-2 px-4 pt-4">
      {#if flashSuccess}
        <Alert color="success" variant="soft" title={flashSuccess} />
      {/if}
      {#if flashError}
        <Alert color="error" variant="soft" title={flashError} />
      {/if}
    </div>
  {/if}

  <main class="flex-1">
    {@render children()}
  </main>
</div>
