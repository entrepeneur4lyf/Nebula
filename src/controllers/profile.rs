//! Profile-management controller.
//!
//! Four authenticated handlers for the signed-in user to manage their own
//! account. They sit in the auth-but-not-verified route group, so an
//! unverified-yet-logged-in user can still update their profile (and, by
//! changing their email, re-trigger verification):
//!
//! - `GET    /profile`          — render the profile page seeded from the
//!   current user.
//! - `PATCH  /profile`          — update name/email. Changing the email nulls
//!   the verification timestamp and re-sends the verification link
//!   (best-effort — a mail failure must not 500 the update).
//! - `PUT    /profile/password` — rotate the password, gated on the current
//!   password (a wrong current password is a 422 on `current_password`).
//! - `DELETE /profile`          — password-gated account deletion: verify the
//!   password, log out, then delete the row.
//!
//! Validation rides the standard `FormRequest` path (per-field rules + a
//! cross-field `after_validation`), so failures surface as a Laravel-style 422
//! `{ message, errors }` envelope the Inertia client merges onto the page.

use serde::Deserialize;
use suprnova::auth_flows::EmailVerification;
use suprnova::{
    handler, hashing, inertia_response, redirect, Auth, CanResetPassword, FormRequest,
    FrameworkError, InertiaProps, Model, MustVerifyEmail, Request, Response, Validate,
    ValidationErrors,
};

use crate::models::user::User;

// ============================================================================
// Props
// ============================================================================

#[derive(InertiaProps)]
pub struct ProfileProps {
    pub name: String,
    pub email: String,
    pub email_verified: bool,
}

// ============================================================================
// Requests
// ============================================================================

#[derive(Deserialize, Validate)]
pub struct UpdateProfileRequest {
    #[validate(length(min = 1, message = "Name is required."))]
    pub name: String,
    #[validate(email(message = "Enter a valid email address."))]
    pub email: String,
}

impl FormRequest for UpdateProfileRequest {}

#[derive(Deserialize, Validate)]
pub struct UpdatePasswordRequest {
    pub current_password: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters."))]
    pub password: String,
    pub password_confirmation: String,
}

impl FormRequest for UpdatePasswordRequest {
    /// Cross-field check: the new password and its confirmation must match.
    /// Runs after the per-field rules, so `password` is already known to be
    /// well-formed (>= 8 chars) before we compare them.
    fn after_validation(&self) -> Result<(), ValidationErrors> {
        if self.password != self.password_confirmation {
            let mut errs = ValidationErrors::new();
            errs.add("password_confirmation", "Passwords do not match.");
            return Err(errs);
        }
        Ok(())
    }
}

/// The delete body carries only the confirming password. It is read straight
/// off the request (not a `FormRequest`) because there is nothing to validate
/// up front — the password is checked against the stored hash below, and a
/// mismatch is a 422 rather than a field-shape error.
#[derive(Deserialize)]
pub struct DeleteAccountRequest {
    pub password: String,
}

// ============================================================================
// Helpers
// ============================================================================

/// Resolve the currently authenticated user as the concrete `User` model.
/// These routes are behind `AuthMiddleware`, so a missing user means the
/// session expired between the gate and the handler — surface it as a 401.
async fn current_user() -> Result<User, FrameworkError> {
    Auth::user_as::<User>()
        .await?
        .ok_or(FrameworkError::Unauthorized)
}

// ============================================================================
// Handlers
// ============================================================================

/// `GET /profile` — render the profile page seeded from the current user.
#[handler]
pub async fn show(req: Request) -> Response {
    let user = current_user().await?;
    inertia_response!(
        &req,
        "Profile",
        ProfileProps {
            name: user.name.clone(),
            email: user.email.clone(),
            email_verified: user.is_email_verified(),
        }
    )
}

/// `PATCH /profile` — update name/email.
///
/// Changing the email address invalidates the existing verification: we null
/// the `email_verified_at` timestamp and re-send the verification link. That
/// send is best-effort — the row update has already succeeded, so a mail
/// failure (e.g. misconfigured `MAIL_FROM`) is logged and swallowed rather
/// than 500-ing the update. The user lands back on `/profile` showing
/// "not verified" and can resend from the verification notice.
#[handler]
pub async fn update(form: UpdateProfileRequest) -> Response {
    let mut user = current_user().await?;
    let email_changed = user.email != form.email;

    // Guard the `users.email` unique constraint: if the new address belongs to
    // a *different* account, surface a 422 on `email` rather than letting the
    // insert/update hit the DB constraint and 500. Re-submitting the same
    // address (email unchanged) is fine and skips the lookup.
    if email_changed
        && let Some(existing) = User::find_by_email(&form.email).await?
        && existing.id != user.id
    {
        let mut errs = ValidationErrors::new();
        errs.add("email", "This email is already registered.");
        return Err(FrameworkError::Validation(errs).into());
    }

    user.name = form.name;
    user.email = form.email;
    if email_changed {
        user.set_email_verified_at(None);
    }
    Model::save(&user).await?;

    if email_changed {
        let base = format!("{}/verify-email/verify", crate::controllers::app_url());
        if let Err(err) = EmailVerification::send_link(&user, &base).await {
            tracing::warn!(error = %err, "failed to send verification email after email change");
        }
    }

    redirect!("/profile").into()
}

/// `PUT /profile/password` — rotate the password.
///
/// Gated on the current password: a wrong `current_password` is a 422 pinned
/// to that field rather than a silent failure. On success the new (hashed)
/// password is persisted through `set_password_hash` + `save`.
#[handler]
pub async fn update_password(form: UpdatePasswordRequest) -> Response {
    let mut user = current_user().await?;

    if !user.verify_password(&form.current_password)? {
        let mut errs = ValidationErrors::new();
        errs.add("current_password", "The current password is incorrect.");
        return Err(FrameworkError::Validation(errs).into());
    }

    user.set_password_hash(&hashing::hash(&form.password)?);
    Model::save(&user).await?;

    redirect!("/profile").into()
}

/// `DELETE /profile` — password-gated account deletion.
///
/// Verify the confirming password (wrong → 422 on `password`), then log the
/// session out and delete the user row. Deletion happens last so an
/// already-logged-out-then-failed-delete can't leave a ghost session pointing
/// at a live account; if the delete fails the user is logged out and re-auth
/// is required, which is the safe direction.
#[handler]
pub async fn destroy(req: Request) -> Response {
    // `input()` dispatches on Content-Type (JSON or form-urlencoded), matching
    // the body shape the Inertia client sends — `delete('/profile', { password })`
    // posts `application/json` by default, which `form()` alone would reject.
    let form: DeleteAccountRequest = req.input().await?;
    let user = current_user().await?;

    if !user.verify_password(&form.password)? {
        let mut errs = ValidationErrors::new();
        errs.add("password", "The password is incorrect.");
        return Err(FrameworkError::Validation(errs).into());
    }

    Auth::logout().await?;
    Model::delete(user).await?;

    redirect!("/").into()
}
