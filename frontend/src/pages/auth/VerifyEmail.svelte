<script lang="ts">
  import { useForm } from '@inertiajs/svelte'

  // Inline-typed props (kept self-contained so this page doesn't depend on the
  // generated `inertia-props.ts`). `status` is `"invalid-or-expired"` when a
  // bad token landed on the verify route, otherwise null.
  let { status }: { status: string | null } = $props()

  const form = useForm({})

  function resend(e: SubmitEvent) {
    e.preventDefault()
    form.post('/email/verification-notification')
  }

  function logout(e: SubmitEvent) {
    e.preventDefault()
    form.post('/logout')
  }
</script>

<div
  class="min-h-screen flex items-center justify-center bg-gray-50 py-12 px-4 sm:px-6 lg:px-8"
>
  <div class="max-w-md w-full space-y-8">
    <div>
      <h2 class="mt-6 text-center text-3xl font-extrabold text-gray-900">
        Verify your email address
      </h2>
      <p class="mt-4 text-center text-sm text-gray-600">
        Before continuing, please check your inbox for a verification link. If
        you didn't receive the email, we'll gladly send you another.
      </p>
    </div>

    {#if status === 'invalid-or-expired'}
      <div class="rounded-md bg-red-50 p-4 text-sm text-red-700">
        That verification link is invalid or has expired. Request a fresh one
        below.
      </div>
    {/if}

    <div class="flex items-center justify-between gap-4">
      <form onsubmit={resend}>
        <button
          type="submit"
          disabled={form.processing}
          class="group relative flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
        >
          Resend verification email
        </button>
      </form>

      <form onsubmit={logout}>
        <button
          type="submit"
          class="text-sm font-medium text-gray-600 hover:text-gray-900 underline"
        >
          Log out
        </button>
      </form>
    </div>
  </div>
</div>
