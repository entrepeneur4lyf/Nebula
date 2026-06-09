<script lang="ts">
  import { untrack } from 'svelte'
  import { useForm } from '@inertiajs/svelte'

  let {
    token,
    errors,
  }: { token: string; errors?: Record<string, string[]> | null } = $props()

  // The reset token is fixed for the life of this page (it comes from the
  // one-shot emailed link), so seed the form with its initial value. `untrack`
  // makes that intent explicit and silences the state-referenced-locally hint.
  const form = useForm({
    token: untrack(() => token),
    password: '',
    password_confirmation: '',
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/reset-password')
  }
</script>

<div
  class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8"
>
  <div class="max-w-md w-full space-y-8">
    <div>
      <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
        Reset your password
      </h2>
      <p class="mt-2 text-center text-sm text-gray-600">
        Choose a new password for your account.
      </p>
    </div>

    {#if errors?.token}
      <div class="rounded-md bg-red-50 border border-red-200 p-4 text-sm text-red-800">
        {errors.token[0]}
      </div>
    {/if}

    <form class="mt-8 space-y-6" onsubmit={submit}>
      <input type="hidden" name="token" bind:value={form.token} />

      <div class="space-y-4">
        <div>
          <label for="password" class="block text-sm font-medium text-gray-700"
            >New password</label
          >
          <input
            id="password"
            name="password"
            type="password"
            autocomplete="new-password"
            required
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            bind:value={form.password}
          />
          {#if errors?.password}
            <p class="mt-1 text-sm text-red-600">{errors.password[0]}</p>
          {/if}
        </div>

        <div>
          <label
            for="password_confirmation"
            class="block text-sm font-medium text-gray-700">Confirm new password</label
          >
          <input
            id="password_confirmation"
            name="password_confirmation"
            type="password"
            autocomplete="new-password"
            required
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            bind:value={form.password_confirmation}
          />
        </div>
      </div>

      <div>
        <button
          type="submit"
          disabled={form.processing}
          class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
        >
          {form.processing ? 'Resetting...' : 'Reset password'}
        </button>
      </div>

      <div class="text-center">
        <a href="/login" class="text-indigo-600 hover:text-indigo-500">
          Back to sign in
        </a>
      </div>
    </form>
  </div>
</div>
