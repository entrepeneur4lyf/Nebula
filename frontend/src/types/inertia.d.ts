// Suprnova flashes validation errors as a field -> messages map
// (`HashMap<String, Vec<String>>` on the wire), so every error value the
// Inertia client sees — `page.props.errors` and each `form.errors` — is a
// `string[]`, not Inertia's default `string`. This declaration-merge teaches
// the client types the real shape; pages render the first message via
// `form.errors.field?.[0]`.
import '@inertiajs/core'

declare module '@inertiajs/core' {
  export interface InertiaConfig {
    errorValueType: string[]
  }
}
