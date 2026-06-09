<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import { Alert, Button, Card, FormField, Input } from 'sv5ui'

  let { errors }: { errors?: Record<string, string[]> | null } = $props()

  const form = useForm({
    email: '',
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/forgot-password', { preserveState: true })
  }
</script>

<div class="flex justify-center px-4 py-12 sm:py-16">
  <Card class="w-full max-w-md">
    {#snippet header()}
      <h1 class="text-xl font-semibold text-on-surface">
        Forgot your password?
      </h1>
      <p class="mt-1 text-sm text-on-surface-variant">
        Enter your email address and we'll send you a link to reset it.
      </p>
    {/snippet}

    <div class="space-y-5">
      {#if form.recentlySuccessful}
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
          error={errors?.email?.[0]}
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
