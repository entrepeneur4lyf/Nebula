<script lang="ts">
  import { untrack } from 'svelte'
  import { useForm } from '@inertiajs/svelte'
  import { Badge, Button, Card, FormField, Input, Modal } from 'sv5ui'

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

  let confirmingDeletion = $state(false)

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

  function closeConfirm() {
    confirmingDeletion = false
    deleteForm.reset()
  }
</script>

<div class="mx-auto w-full max-w-2xl space-y-6 px-4 py-10">
  <h1 class="text-2xl font-semibold text-on-surface">Profile</h1>

  <Card>
    {#snippet header()}
      <div class="flex items-center justify-between gap-3">
        <h2 class="text-lg font-medium text-on-surface">Profile information</h2>
        {#if email_verified}
          <Badge color="success" variant="soft" label="Email verified" />
        {:else}
          <Badge color="warning" variant="soft" label="Email not verified" />
        {/if}
      </div>
      <p class="mt-1 text-sm text-on-surface-variant">
        Update your account's name and email address.
      </p>
    {/snippet}

    <form class="space-y-5" onsubmit={submitProfile}>
      <FormField name="name" label="Name" required error={errors?.name?.[0]}>
        <Input
          type="text"
          autocomplete="name"
          required
          bind:value={profileForm.name}
        />
      </FormField>

      <FormField
        name="email"
        label="Email address"
        required
        help="Changing your email requires verifying the new address."
        error={errors?.email?.[0]}
      >
        <Input
          type="email"
          autocomplete="email"
          required
          bind:value={profileForm.email}
        />
      </FormField>

      <Button type="submit" label="Save" loading={profileForm.processing} />
    </form>
  </Card>

  <Card>
    {#snippet header()}
      <h2 class="text-lg font-medium text-on-surface">Update password</h2>
      <p class="mt-1 text-sm text-on-surface-variant">
        Use a long, random password to keep your account secure.
      </p>
    {/snippet}

    <form class="space-y-5" onsubmit={submitPassword}>
      <FormField
        name="current_password"
        label="Current password"
        required
        error={errors?.current_password?.[0]}
      >
        <Input
          type="password"
          autocomplete="current-password"
          required
          bind:value={passwordForm.current_password}
        />
      </FormField>

      <FormField
        name="password"
        label="New password"
        required
        help="At least 8 characters."
        error={errors?.password?.[0]}
      >
        <Input
          type="password"
          autocomplete="new-password"
          required
          bind:value={passwordForm.password}
        />
      </FormField>

      <FormField
        name="password_confirmation"
        label="Confirm new password"
        required
        error={errors?.password_confirmation?.[0]}
      >
        <Input
          type="password"
          autocomplete="new-password"
          required
          bind:value={passwordForm.password_confirmation}
        />
      </FormField>

      <Button
        type="submit"
        label="Update password"
        loading={passwordForm.processing}
      />
    </form>
  </Card>

  <Card>
    {#snippet header()}
      <h2 class="text-lg font-medium text-error">Delete account</h2>
      <p class="mt-1 text-sm text-on-surface-variant">
        Once your account is deleted, all of its data is permanently removed.
      </p>
    {/snippet}

    <Button
      color="error"
      label="Delete account"
      onclick={() => (confirmingDeletion = true)}
    />
  </Card>
</div>

<Modal
  bind:open={confirmingDeletion}
  title="Delete account"
  description="This action cannot be undone. Enter your password to confirm you want to permanently delete your account."
  onOpenChange={(open) => {
    if (!open) deleteForm.reset()
  }}
>
  {#snippet body()}
    <form id="delete-account-form" onsubmit={submitDelete}>
      <FormField
        name="password"
        label="Password"
        required
        error={errors?.password?.[0]}
      >
        <Input
          type="password"
          autocomplete="current-password"
          placeholder="Your current password"
          required
          bind:value={deleteForm.password}
        />
      </FormField>
    </form>
  {/snippet}

  {#snippet footer()}
    <div class="flex w-full justify-end gap-3">
      <Button
        variant="ghost"
        color="surface"
        label="Cancel"
        onclick={closeConfirm}
      />
      <Button
        type="submit"
        form="delete-account-form"
        color="error"
        label="Permanently delete"
        loading={deleteForm.processing}
      />
    </div>
  {/snippet}
</Modal>
