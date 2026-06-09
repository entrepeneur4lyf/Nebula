<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import { Alert, Button, Card, FormField, Input } from 'sv5ui'
  import AuthEmblem from '../../lib/AuthEmblem.svelte'

  const form = useForm({
    email: '',
  })

  // Tracks "a reset link was sent" across submits. `form.wasSuccessful` sticks
  // once set, so a later failed submit would show the success banner alongside
  // a field error; this flag clears at submit start and persists otherwise.
  let linkSent = $state(false)

  function submit(e: SubmitEvent) {
    e.preventDefault()
    linkSent = false
    form.post('/forgot-password', {
      preserveState: true,
      onSuccess: () => {
        linkSent = true
      },
    })
  }
</script>

<div class="flex justify-center px-4 py-12 sm:py-16">
  <Card class="w-full max-w-md">
    {#snippet header()}
      <AuthEmblem />
      <h1 class="mt-3 text-center text-xl font-semibold text-on-surface">
        Forgot your password?
      </h1>
      <p class="mt-1 text-center text-sm text-on-surface-variant">
        Enter your email address and we'll send you a link to reset it.
      </p>
    {/snippet}

    <div class="space-y-5">
      {#if linkSent}
        <Alert
          color="success"
          variant="soft"
          icon="lucide:mail"
          title="Check your inbox"
          description="If that email address is in our system, a password reset link is on its way."
        />
      {/if}

      <form class="space-y-5" onsubmit={submit}>
        <FormField
          name="email"
          label="Email address"
          required
          error={form.errors.email?.[0]}
        >
          <Input
            type="email"
            autocomplete="email"
            placeholder="you@example.com"
            required
            bind:value={form.email}
          />
        </FormField>

        <Button
          type="submit"
          block
          label="Email password reset link"
          loading={form.processing}
        />
      </form>
    </div>

    {#snippet footer()}
      <p class="text-center text-sm text-on-surface-variant">
        Remembered it after all?
        <Link href="/login" class="font-medium text-primary hover:underline">
          Back to sign in
        </Link>
      </p>
    {/snippet}
  </Card>
</div>
