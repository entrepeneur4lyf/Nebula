<script lang="ts">
  import { useForm } from '@inertiajs/svelte'

  let { errors }: { errors?: Record<string, string[]> | null } = $props()

  const form = useForm({
    email: '',
  })

  function submit(e: SubmitEvent) {
    e.preventDefault()
    form.post('/forgot-password', { preserveState: true })
  }
</script>

<div
  class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8"
>
  <div class="max-w-md w-full space-y-8">
    <div>
      <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
        Forgot your password?
      </h2>
      <p class="mt-2 text-center text-sm text-gray-600">
        Enter your email address and we'll send you a link to reset it.
      </p>
    </div>

    {#if form.recentlySuccessful}
      <div class="rounded-md bg-green-50 border border-green-200 p-4 text-sm text-green-800">
        If that email address is in our system, a password reset link is on its
        way.
      </div>
    {/if}

    <form class="mt-8 space-y-6" onsubmit={submit}>
      <div>
        <label for="email" class="block text-sm font-medium text-gray-700"
          >Email address</label
        >
        <input
          id="email"
          name="email"
          type="email"
          autocomplete="email"
          required
          class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
          placeholder="Email address"
          bind:value={form.email}
        />
        {#if errors?.email}
          <p class="mt-1 text-sm text-red-600">{errors.email[0]}</p>
        {/if}
      </div>

      <div>
        <button
          type="submit"
          disabled={form.processing}
          class="w-full flex justify-center py-2 px-4 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
        >
          {form.processing ? 'Sending...' : 'Email password reset link'}
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
