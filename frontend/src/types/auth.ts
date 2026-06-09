// Hand-maintained auth types. The generated `inertia-props.ts` is owned by
// `suprnova generate-types` and must not be edited, so the shared auth-prop
// shape lives here instead.

/** The authenticated user as shared on `page.props.auth.user`. */
export interface AuthUser {
  id: number
  name: string
  email: string
}
