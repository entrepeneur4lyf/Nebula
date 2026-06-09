<script lang="ts">
  import { Link, useForm } from '@inertiajs/svelte'
  import { Button, Card, Checkbox, FormField, Input } from 'sv5ui'
  import AuthEmblem from '../../lib/AuthEmblem.svelte'

  const form = useForm({
    email: '',
    password: '',
    remember: false,
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/login')
  }
</script>

<div class="flex justify-center px-4 py-12 sm:py-16">
  <Card class="w-full max-w-md">
    {#snippet header()}
      <AuthEmblem />
      <h1 class="mt-3 text-center text-xl font-semibold text-on-surface">
        Sign in to Nebula
      </h1>
      <p class="mt-1 text-center text-sm text-on-surface-variant">
        Welcome back. Enter your credentials to continue.
      </p>
    {/snippet}

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

      <FormField
        name="password"
        label="Password"
        required
        error={form.errors.password?.[0]}
      >
        <Input
          type="password"
          autocomplete="current-password"
          required
          bind:value={form.password}
        />
      </FormField>

      <div class="flex items-center justify-between">
        <Checkbox label="Remember me" bind:checked={form.remember} />
        <Link
          href="/forgot-password"
          class="text-sm font-medium text-primary hover:underline"
        >
          Forgot your password?
        </Link>
      </div>

      <Button type="submit" block label="Sign in" loading={form.processing} />
    </form>

    {#snippet footer()}
      <p class="text-center text-sm text-on-surface-variant">
        Don't have an account?
        <Link href="/register" class="font-medium text-primary hover:underline">
          Register
        </Link>
      </p>
    {/snippet}
  </Card>
</div>
