//! Password-reset controller.
//!
//! The full forgot-password / reset-password flow, all on guest-only routes:
//!
//! - `GET  /forgot-password` — render the "request a reset link" form.
//! - `POST /forgot-password` — mail a reset link. Anti-enumeration: the
//!   response is identical whether or not the address exists, so a caller can't
//!   probe which emails are registered. The `PasswordReset::send_link` facade
//!   only mails when the address resolves, but always returns `Ok`.
//! - `GET  /reset-password?token=…` — render the new-password form, carrying the
//!   token through as a hidden field.
//! - `POST /reset-password` — consume the token and set the new password via
//!   `PasswordReset::complete`. An invalid or expired token re-renders the form
//!   with a `token` validation error rather than dumping a raw failure.
//!
//! Tokens are minted/consumed through the provider-agnostic `PasswordReset`
//! facade, which drives the address lookup + password write through the
//! `EloquentUserProvider<User>` registered in `bootstrap::register()`.

use serde::Deserialize;
use suprnova::auth_flows::PasswordReset;
use suprnova::{
    handler, inertia_response, redirect, FormRequest, FrameworkError, InertiaProps, Request,
    Response, Validate, ValidationErrors,
};

/// Base URL the reset token is appended to. `PasswordReset` builds
/// `{base}?token=…`, which must resolve to the `show_reset` handler below.
fn reset_base() -> String {
    format!("{}/reset-password", crate::controllers::app_url())
}

#[derive(InertiaProps)]
pub struct ForgotPasswordProps {}

#[derive(InertiaProps)]
pub struct ResetPasswordProps {
    /// The reset token from the emailed link, threaded into the form so the
    /// POST can present it back for consumption.
    pub token: String,
}

#[derive(Deserialize, Validate)]
pub struct SendLinkRequest {
    #[validate(email(message = "Enter a valid email address."))]
    pub email: String,
}

impl FormRequest for SendLinkRequest {}

#[derive(Deserialize, Validate)]
pub struct ResetRequest {
    pub token: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters."))]
    pub password: String,
    pub password_confirmation: String,
}

impl FormRequest for ResetRequest {
    /// Cross-field check: the password and its confirmation must match. Runs
    /// after the per-field rules pass, so the length rule is already satisfied
    /// before we compare the two values.
    fn after_validation(&self) -> Result<(), ValidationErrors> {
        if self.password != self.password_confirmation {
            let mut errs = ValidationErrors::new();
            errs.add("password", "Passwords do not match.");
            return Err(errs);
        }
        Ok(())
    }
}

/// `GET /forgot-password` — render the "request a reset link" form.
#[handler]
pub async fn show_request(req: Request) -> Response {
    inertia_response!(&req, "auth/ForgotPassword", ForgotPasswordProps {})
}

/// `POST /forgot-password` — mail a reset link.
///
/// Anti-enumeration: this always succeeds the same way regardless of whether
/// the address is registered. `send_link` mails only when the address resolves
/// but returns `Ok` either way, so the 302-back response is indistinguishable.
#[handler]
pub async fn send_link(form: SendLinkRequest) -> Response {
    PasswordReset::send_link(&form.email, &reset_base()).await?;
    redirect!("/forgot-password").into()
}

/// `GET /reset-password?token=…` — render the new-password form.
///
/// The token is carried through to the page so the subsequent POST can present
/// it back. A missing token renders an empty field; the POST then fails the
/// token check and surfaces the standard "invalid or expired" message.
#[handler]
pub async fn show_reset(req: Request) -> Response {
    let token = req.query_param("token").unwrap_or_default();
    inertia_response!(&req, "auth/ResetPassword", ResetPasswordProps { token })
}

/// `POST /reset-password` — consume the token and set the new password.
///
/// On success, 302 to `/login` so the user signs in with the new credentials.
/// An invalid/expired/consumed token re-renders the form with a `token`
/// validation error (a standard 422) instead of a raw failure.
#[handler]
pub async fn reset(form: ResetRequest) -> Response {
    match PasswordReset::complete(&form.token, &form.password).await {
        Ok(_user_id) => redirect!("/login").into(),
        Err(_) => {
            let mut errs = ValidationErrors::new();
            errs.add("token", "This password reset link is invalid or has expired.");
            Err(FrameworkError::Validation(errs).into())
        }
    }
}
