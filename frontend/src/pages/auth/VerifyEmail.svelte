<script lang="ts">
  import { useForm } from '@inertiajs/svelte'
  import { Alert, Button, Card } from 'sv5ui'
  import AuthEmblem from '../../lib/AuthEmblem.svelte'

  // `status` is set by the server: `"invalid-or-expired"` when a bad token
  // landed on the verify route, `null` otherwise. A successful verify
  // redirects to the dashboard and a successful resend redirects back here
  // without a status key, so the "link sent" confirmation keys off a local
  // flag instead.
  let { status }: { status: string | null } = $props()

  const statusAlerts: Record<
    string,
    { color: 'success' | 'error'; icon: string; title: string }
  > = {
    'invalid-or-expired': {
      color: 'error',
      icon: 'lucide:link-2-off',
      title:
        'That verification link is invalid or has expired. Request a fresh one below.',
    },
  }

  const statusAlert = $derived(status ? (statusAlerts[status] ?? null) : null)

  const resendForm = useForm({})
  const logoutForm = useForm({})

  // Tracks "a fresh link was sent" across submits. `resendForm.wasSuccessful`
  // sticks once set, so a later failed resend would keep showing the success
  // banner; this flag clears at submit start and persists otherwise.
  let linkSent = $state(false)

  function resend(e: SubmitEvent) {
    e.preventDefault()
    linkSent = false
    resendForm.post('/email/verification-notification', {
      preserveState: true,
      onSuccess: () => {
        linkSent = true
      },
    })
  }

  function logout(e: SubmitEvent) {
    e.preventDefault()
    logoutForm.post('/logout')
  }
</script>

<div class="flex justify-center px-4 py-12 sm:py-16">
  <Card class="w-full max-w-md">
    <div class="space-y-5 text-center">
      <AuthEmblem />

      <div class="space-y-1">
        <h1 class="text-xl font-semibold text-on-surface">
          Verify your email address
        </h1>
        <p class="text-sm text-on-surface-variant">
          Before continuing, please check your inbox for a verification link.
          If you didn't receive the email, we'll gladly send you another.
        </p>
      </div>

      {#if statusAlert}
        <Alert
          color={statusAlert.color}
          variant="soft"
          icon={statusAlert.icon}
          title={statusAlert.title}
          class="text-left"
        />
      {:else if linkSent}
        <Alert
          color="success"
          variant="soft"
          icon="lucide:mail-check"
          title="A fresh verification link has been sent to your email address."
          class="text-left"
        />
      {/if}

      <form onsubmit={resend}>
        <Button
          type="submit"
          block
          label="Resend verification email"
          loading={resendForm.processing}
        />
      </form>
    </div>

    {#snippet footer()}
      <form class="text-center" onsubmit={logout}>
        <Button
          type="submit"
          variant="link"
          color="surface"
          label="Log out"
          loading={logoutForm.processing}
        />
      </form>
    {/snippet}
  </Card>
</div>
