<script lang="ts">
  import type { HomeProps } from '../types/inertia-props'

  let { title, message }: HomeProps = $props()
</script>

<div class="font-sans p-8 max-w-xl mx-auto">
  <h1 class="text-3xl font-bold">{title}</h1>
  <p class="mt-2">{message}</p>
  <p class="mt-8 text-on-surface-variant">
    Edit
    <code class="bg-surface-container px-1 rounded"
      >frontend/src/pages/Home.svelte</code
    >
    to get started.
  </p>
</div>
