<script lang="ts">
  import { untrack } from 'svelte'
  import { useForm } from '@inertiajs/svelte'

  let { name, email, email_verified, errors }: {
    name: string
    email: string
    email_verified: boolean
    errors?: Record<string, string[]> | null
  } = $props()

  // Seed the profile form from the server-provided values. They're the
  // initial state of this page; `untrack` makes that intent explicit and
  // silences the state-referenced-locally hint (matching the auth pages).
  const profileForm = useForm({
    name: untrack(() => name),
    email: untrack(() => email),
  })

  const passwordForm = useForm({
    current_password: '',
    password: '',
    password_confirmation: '',
  })

  const deleteForm = useForm({
    password: '',
  })

  let showConfirm = $state(false)

  function submitProfile(e: SubmitEvent) {
    e.preventDefault()
    profileForm.patch('/profile')
  }

  function submitPassword(e: SubmitEvent) {
    e.preventDefault()
    passwordForm.put('/profile/password', {
      onSuccess: () => passwordForm.reset(),
    })
  }

  function submitDelete(e: SubmitEvent) {
    e.preventDefault()
    deleteForm.delete('/profile')
  }
</script>

<div class="min-h-screen bg-gray-50 py-12 px-4 sm:px-6 lg:px-8">
  <div class="max-w-2xl w-full mx-auto space-y-8">
    <h1 class="text-3xl font-extrabold text-gray-900">Profile</h1>

    <!-- Profile information -->
    <section class="bg-white shadow rounded-lg p-6 space-y-6">
      <div>
        <h2 class="text-lg font-medium text-gray-900">Profile information</h2>
        <p class="mt-1 text-sm text-gray-500">
          Update your account's name and email address.
        </p>
      </div>

      <form class="space-y-4" onsubmit={submitProfile}>
        <div>
          <label for="name" class="block text-sm font-medium text-gray-700">
            Name
          </label>
          <input
            id="name"
            name="name"
            type="text"
            autocomplete="name"
            required
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            bind:value={profileForm.name}
          />
          {#if errors?.name}
            <div class="mt-1 text-red-600 text-sm">{errors.name[0]}</div>
          {/if}
        </div>

        <div>
          <label for="email" class="block text-sm font-medium text-gray-700">
            Email address
          </label>
          <input
            id="email"
            name="email"
            type="email"
            autocomplete="email"
            required
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            bind:value={profileForm.email}
          />
          {#if errors?.email}
            <div class="mt-1 text-red-600 text-sm">{errors.email[0]}</div>
          {/if}
          <div class="mt-2 text-sm">
            {#if email_verified}
              <span class="text-green-600">Email verified ✓</span>
            {:else}
              <span class="text-amber-600">Email not verified</span>
            {/if}
          </div>
        </div>

        <button
          type="submit"
          disabled={profileForm.processing}
          class="inline-flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
        >
          {profileForm.processing ? 'Saving...' : 'Save'}
        </button>
      </form>
    </section>

    <!-- Update password -->
    <section class="bg-white shadow rounded-lg p-6 space-y-6">
      <div>
        <h2 class="text-lg font-medium text-gray-900">Update password</h2>
        <p class="mt-1 text-sm text-gray-500">
          Use a long, random password to keep your account secure.
        </p>
      </div>

      <form class="space-y-4" onsubmit={submitPassword}>
        <div>
          <label
            for="current_password"
            class="block text-sm font-medium text-gray-700"
          >
            Current password
          </label>
          <input
            id="current_password"
            name="current_password"
            type="password"
            autocomplete="current-password"
            required
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            bind:value={passwordForm.current_password}
          />
          {#if errors?.current_password}
            <div class="mt-1 text-red-600 text-sm">
              {errors.current_password[0]}
            </div>
          {/if}
        </div>

        <div>
          <label for="password" class="block text-sm font-medium text-gray-700">
            New password
          </label>
          <input
            id="password"
            name="password"
            type="password"
            autocomplete="new-password"
            required
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            bind:value={passwordForm.password}
          />
          {#if errors?.password}
            <div class="mt-1 text-red-600 text-sm">{errors.password[0]}</div>
          {/if}
        </div>

        <div>
          <label
            for="password_confirmation"
            class="block text-sm font-medium text-gray-700"
          >
            Confirm new password
          </label>
          <input
            id="password_confirmation"
            name="password_confirmation"
            type="password"
            autocomplete="new-password"
            required
            class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 focus:outline-none focus:ring-indigo-500 focus:border-indigo-500 sm:text-sm"
            bind:value={passwordForm.password_confirmation}
          />
          {#if errors?.password_confirmation}
            <div class="mt-1 text-red-600 text-sm">
              {errors.password_confirmation[0]}
            </div>
          {/if}
        </div>

        <button
          type="submit"
          disabled={passwordForm.processing}
          class="inline-flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-indigo-600 hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:opacity-50"
        >
          {passwordForm.processing ? 'Saving...' : 'Update password'}
        </button>
      </form>
    </section>

    <!-- Delete account -->
    <section class="bg-white shadow rounded-lg p-6 space-y-6">
      <div>
        <h2 class="text-lg font-medium text-red-700">Delete account</h2>
        <p class="mt-1 text-sm text-gray-500">
          Once your account is deleted, all of its data is permanently removed.
          Enter your password to confirm.
        </p>
      </div>

      {#if !showConfirm}
        <button
          type="button"
          onclick={() => (showConfirm = true)}
          class="inline-flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500"
        >
          Delete account
        </button>
      {:else}
        <form class="space-y-4" onsubmit={submitDelete}>
          <div>
            <label
              for="delete_password"
              class="block text-sm font-medium text-gray-700"
            >
              Password
            </label>
            <input
              id="delete_password"
              name="password"
              type="password"
              autocomplete="current-password"
              required
              class="mt-1 block w-full px-3 py-2 border border-gray-300 rounded-md text-gray-900 focus:outline-none focus:ring-red-500 focus:border-red-500 sm:text-sm"
              bind:value={deleteForm.password}
            />
            {#if errors?.password}
              <div class="mt-1 text-red-600 text-sm">{errors.password[0]}</div>
            {/if}
          </div>

          <div class="flex items-center gap-3">
            <button
              type="submit"
              disabled={deleteForm.processing}
              class="inline-flex justify-center py-2 px-4 border border-transparent text-sm font-medium rounded-md text-white bg-red-600 hover:bg-red-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-red-500 disabled:opacity-50"
            >
              {deleteForm.processing ? 'Deleting...' : 'Permanently delete'}
            </button>
            <button
              type="button"
              onclick={() => {
                showConfirm = false
                deleteForm.reset()
              }}
              class="inline-flex justify-center py-2 px-4 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500"
            >
              Cancel
            </button>
          </div>
        </form>
      {/if}
    </section>
  </div>
</div>
