<script lang="ts">
  import { Card } from 'sv5ui'
  import type { DashboardProps } from '../types/inertia-props'

  let { user }: DashboardProps = $props()
</script>

<div class="mx-auto w-full max-w-5xl space-y-6 px-4 py-10">
  <h1 class="text-2xl font-semibold text-on-surface">Dashboard</h1>

  <Card>
    <div class="space-y-2">
      <h2 class="text-lg font-medium text-on-surface">
        Welcome, {user.name}!
      </h2>
      <p class="text-sm text-on-surface-variant">You are logged in.</p>
      <p class="text-sm text-on-surface-variant">Email: {user.email}</p>
    </div>
  </Card>
</div>
