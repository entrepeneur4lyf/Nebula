//! Account-management flow tests for the Nebula starter kit — facade level.
//!
//! These exercise the three account flows the kit ships — email verification,
//! password reset (with anti-enumeration), and the profile update / password /
//! delete surface — end-to-end against a **real** in-memory database (Nebula's
//! own `Migrator`) with the mail transport **faked** (`Mail::fake()`). No mocks:
//! the assertions read the persisted `users` row back through the same
//! `EloquentUserProvider<User>` the kit registers, and the tokens are extracted
//! from the captured mail bodies.
//!
//! ## Why facade level and not full-HTTP — two kit defects surfaced
//!
//! Task 6's preferred shape is full-HTTP through the kit's real router
//! (`nebula::routes::register()`). That is currently **blocked** by two defects
//! these tests uncovered — exactly the kind of pre-v0.1.0 wiring breakage this
//! task exists to flush out. As shipped, the starter kit **cannot serve a single
//! authenticated HTTP request**: every grouped route 404s and every controller
//! redirect 500s. Both defects are fixable **entirely within the nebula repo**
//! (no cross-repo coordination needed), but each reaches beyond
//! `tests/auth_flows.rs`, so per the task's escalation clause we test at the
//! facade level here and report the defects + fixes for a follow-up task.
//!
//! 1. **`group!("/")` registers unreachable double-slash paths.** `src/routes.rs`
//!    wraps every authenticated route in `group!("/", { get!("/login", …), … })`.
//!    The group builder concatenates `prefix + inner`
//!    (`framework/src/routing/macros.rs::register_with_inherited`), so a `"/"`
//!    prefix with inner `/login` registers the matchit pattern `//login`. A
//!    browser request to `/login` then 404s. Empirically: driving
//!    `nebula::routes::register()` over a real hyper server returns **404** for
//!    every grouped route (`POST /register`, `POST /forgot-password`, `PATCH
//!    /profile`, …); rebuilding the identical groups with an **empty** prefix
//!    makes them route. Two fixes, not mutually exclusive: the *correct*
//!    long-term fix is to collapse a leading `//` in `register_with_inherited`
//!    (a framework change, in nation-x-com, so no other consumer hits this); the
//!    *nebula-local* fix that unblocks full-HTTP coverage now is to change the
//!    kit's idiom to `group!("", { … })` (or drop the wrapping group for
//!    absolute-path routes). The top-level ungrouped routes (`GET /`, `GET
//!    /verify-email/verify`) are unaffected.
//!
//! 2. **Controllers use the named-route redirect helper with no named routes.**
//!    Every controller success path calls `redirect!("/dashboard")` /
//!    `redirect!("/login")` / `redirect!("/profile")`. `redirect!` expands to
//!    `Redirect::route(name)`, which resolves its argument as a **route name**
//!    via `try_route_with_params` — and the kit names no routes, so the lookup
//!    returns `NameNotFound` and the request 500s. (Proven directly: with the
//!    routing in #1 worked around, the grouped POSTs reach their handlers and
//!    then 500 with `Route '/dashboard' not found`.) Fix (nebula-local): the
//!    controllers should use the literal-path redirect (`Redirect::to(
//!    "/dashboard")`) — or the routes should be `.name(...)`d to match the
//!    `redirect!` arguments. The gate middleware is read-of-source **inferred**
//!    to be unaffected (`AuthMiddleware`/`EnsureEmailVerifiedMiddleware` set the
//!    `Location` header from their literal `redirect_to(...)` string, not via
//!    `redirect!`), so the guest / auth / verified gates should work — but the
//!    HTTP path 404s/500s before any gate completes a redirect, so this is not
//!    yet empirically exercised. The full-HTTP tests will be the first to prove
//!    it once #1 and #2 are fixed.
//!
//! 3. **The compile-time redirect guard is silently disabled for this kit.**
//!    `redirect!`'s `validate_route_exists` greps `routes.rs` for `.name("…")`
//!    and, finding *zero* named routes, treats the route table as "unknown" and
//!    **skips validation entirely** (`available_routes.is_empty()` → `Ok`). So a
//!    kit that names no routes gets no compile-time protection and 100% runtime
//!    500s on every redirect — the safety net is off precisely in the case that
//!    needs it most. Worth a framework hardening pass (e.g. treat an empty name
//!    table as "validate against the path table" rather than "skip").
//!
//! These tests pass cleanly because the **flow logic** (token mint/consume,
//! password rotation, verification-stamp write, anti-enumeration) is correct —
//! the breakage is purely in the HTTP wiring above it. That isolation is itself
//! the signal: fix the two wiring defects and the same flows light up over HTTP.
//!
//! ## Coverage note
//!
//! Because these run below the router, they do **not** exercise the dashboard
//! gate's 302 nor the profile *controllers* (which are unreachable until the
//! two defects are fixed). They DO prove: verification send + single-use
//! consume + stamp persistence; resend; password reset rotation + revocation +
//! single-use; anti-enumeration; and the profile email-change re-verification
//! and password-rotation *logic* against the real `User` model.
//!
//! ## Serial execution
//!
//! `Mail::fake()` swaps the process-global mail transport and the active user
//! provider is process-global, so the file is serialized behind `TEST_LOCK`,
//! mirroring `framework/tests/email_verify.rs`.

use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use sea_orm_migration::MigratorTrait;
use tokio::sync::Mutex;

use suprnova::auth::AuthConfig;
use suprnova::auth_flows::{EmailVerification, PasswordReset};
use suprnova::mail::{Mail, MailFake};
use suprnova::session::driver::database::DatabaseSessionDriver;
use suprnova::session::{SessionData, SessionStore};
use suprnova::{
    App, Auth, AuthManager, Authenticatable, EloquentUserProvider, MustVerifyEmail,
};

use nebula::migrations::Migrator;
use nebula::models::user::User;

/// Serializes every test in this file (process-global mail fake + provider).
static TEST_LOCK: Mutex<()> = Mutex::const_new(());

/// Held-for-the-test guard: keeps the SeaORM connection registered for the
/// duration of the test so the provider + facades resolve `DB::connection()`.
struct Harness {
    _lock: tokio::sync::MutexGuard<'static, ()>,
}

/// Fresh in-memory DB with Nebula's full migration set, the
/// `EloquentUserProvider::<User>` registered as the active "users" provider
/// (mirroring `bootstrap::register()`), and the load-bearing `MAIL_FROM` /
/// `APP_URL` env set (the verify/reset send paths fail closed without
/// `MAIL_FROM`; `APP_URL` pins the emitted link base).
async fn setup() -> Harness {
    let lock = TEST_LOCK.lock().await;

    // SAFETY: every test in this file is serialized behind `TEST_LOCK`.
    unsafe {
        std::env::set_var("MAIL_FROM", "test@nebula.test");
        std::env::set_var("APP_URL", "http://nebula.test");
    }

    let conn = sea_orm::Database::connect("sqlite::memory:")
        .await
        .expect("connect sqlite::memory:");
    Migrator::up(&conn, None)
        .await
        .expect("run Nebula migrations against sqlite::memory:");
    App::singleton(suprnova::DbConnection::from_raw(conn));

    // Auth wiring — mirror `bootstrap::register()` exactly. `AuthConfig::default()`'s
    // "web" guard points at the "users" provider.
    App::singleton(AuthManager::new(AuthConfig::default()));
    Auth::register_provider("users", Arc::new(EloquentUserProvider::<User>::new()))
        .expect("register users provider");

    Harness { _lock: lock }
}

/// Reload a user from the DB by email via the same model surface the kit uses.
async fn reload(email: &str) -> User {
    User::find_by_email(email)
        .await
        .expect("lookup")
        .unwrap_or_else(|| panic!("user {email} exists"))
}

/// Stamp a user verified (used to seed the password-reset / profile fixtures in
/// the same already-verified state the kit produces after a click-through).
async fn mark_verified(user: &mut User) {
    user.email_verified_at = Some(Utc::now());
    suprnova::eloquent::Model::save(user)
        .await
        .expect("stamp email_verified_at");
}

/// Pull the plaintext token out of the first captured mail whose text body
/// carries a `token=` link (the text body renders the URL verbatim; the HTML
/// body HTML-escapes slashes) — the same extraction the framework facade tests
/// use.
fn token_from_fake(fake: &MailFake) -> String {
    let captured = fake.captured();
    let msg = captured
        .iter()
        .find(|m| {
            m.text
                .as_deref()
                .is_some_and(|t| t.lines().any(|l| l.contains("token=")))
        })
        .expect("a captured mail with a token link");
    let text = msg.text.as_deref().expect("token mail has a text body");
    let link = text
        .lines()
        .find(|l| l.contains("token="))
        .expect("a line with the token link");
    link.rsplit("token=")
        .next()
        .expect("token after marker")
        .trim()
        .to_string()
}

// ============================================================================
// 1. Verification: send link → single-use consume → stamp persists
// ============================================================================
//
// The kit's `register` controller sends this link; the public
// `/verify-email/verify` handler consumes it through `EmailVerification::verify`
// and the `verified` gate keys off `is_email_verified()`. This proves the mint
// → consume → persistence chain the gate depends on.

#[tokio::test]
async fn verification_sends_link_consumes_once_and_persists_stamp() {
    let _h = setup().await;

    // A freshly-registered, unverified user (created via the kit's own helper).
    let user = User::create("Grace Hopper", "grace@nebula.test", "supersecret")
        .await
        .expect("create user");
    assert!(
        !user.is_email_verified(),
        "a freshly created user is unverified"
    );

    // Send the verification link (the base the kit appends `?token=` to).
    let fake = Mail::fake();
    EmailVerification::send_link(&user, "http://nebula.test/verify-email/verify")
        .await
        .expect("send verification link");
    fake.assert_sent_to("grace@nebula.test");
    assert_eq!(fake.count(), 1, "exactly one verification mail");
    let token = token_from_fake(&fake);

    // Not yet verified in the DB.
    assert!(!reload("grace@nebula.test").await.is_email_verified());

    // Consume the token — marks the user verified, returns the id.
    let id = EmailVerification::verify(&token).await.expect("verify");
    assert_eq!(id, user.get_auth_identifier());
    assert!(
        reload("grace@nebula.test").await.is_email_verified(),
        "verify must persist email_verified_at through the provider"
    );

    // Single-use: a second consume of the same token fails.
    assert!(
        EmailVerification::verify(&token).await.is_err(),
        "a consumed verification token must not verify again"
    );
}

// ============================================================================
// 2. Resend: a fresh link for an unverified user; silent for unknown
// ============================================================================
//
// Mirrors `POST /email/verification-notification` (resend) + the kit's
// anti-enumeration posture.

#[tokio::test]
async fn resend_sends_a_fresh_link_and_is_silent_for_unknown() {
    let _h = setup().await;
    User::create("Ada Lovelace", "ada@nebula.test", "oldpass1!")
        .await
        .expect("create user");

    // Known, unverified email → a fresh link is mailed.
    {
        let fake = Mail::fake();
        EmailVerification::resend("ada@nebula.test", "http://nebula.test/verify-email/verify")
            .await
            .expect("resend known");
        assert_eq!(fake.count(), 1, "known email must trigger a fresh link");
        fake.assert_sent_to("ada@nebula.test");
        assert!(
            !token_from_fake(&fake).is_empty(),
            "the resent link carries a token"
        );
    }

    // Unknown email → anti-enumeration: nothing sent, still Ok.
    {
        let fake = Mail::fake();
        EmailVerification::resend("nobody@nebula.test", "http://nebula.test/verify-email/verify")
            .await
            .expect("resend unknown returns Ok (no leak)");
        assert_eq!(
            fake.count(),
            0,
            "unknown email must send nothing (anti-enumeration)"
        );
    }
}

// ============================================================================
// 3. Password reset: rotate + revoke + single-use, and anti-enumeration
// ============================================================================
//
// Mirrors `POST /forgot-password` (send_link, anti-enumeration) and `POST
// /reset-password` (complete). Asserts the new password verifies, the old one
// does not, a live session is revoked, the token is single-use, and an unknown
// email leaks nothing.

#[tokio::test]
async fn password_reset_rotates_password_revokes_sessions_and_is_anti_enumerating() {
    let _h = setup().await;

    let mut ada = User::create("Ada Reset", "ada@x.com", "oldpass1!")
        .await
        .expect("create user");
    mark_verified(&mut ada).await;
    let id = ada.get_auth_identifier();

    // Seed a live session row for Ada so the reset's revocation has a real row
    // to delete (the kit's `PasswordReset::complete` revokes sessions on reset).
    let session_driver = DatabaseSessionDriver::new(Duration::from_secs(3600));
    let mut sess = SessionData::new("ada-sess-1".into(), "ada-csrf".into());
    sess.user_id = Some(id.clone());
    session_driver.write(&sess).await.expect("seed session");
    assert!(
        session_driver
            .read("ada-sess-1")
            .await
            .expect("read seeded session")
            .is_some(),
        "the seeded session must exist before the reset"
    );

    // Known email → a reset mail is sent.
    let fake = Mail::fake();
    PasswordReset::send_link("ada@x.com", "http://nebula.test/reset-password")
        .await
        .expect("send_link");
    fake.assert_sent_to("ada@x.com");
    let token = token_from_fake(&fake);

    // Unknown email → anti-enumeration: nothing sent, still Ok.
    {
        let fake2 = Mail::fake();
        PasswordReset::send_link("nobody@x.com", "http://nebula.test/reset-password")
            .await
            .expect("send_link unknown returns Ok (no leak)");
        assert_eq!(
            fake2.count(),
            0,
            "unknown email must send nothing (anti-enumeration)"
        );
    }

    // Complete the reset → rotates the password, returns the id.
    let returned = PasswordReset::complete(&token, "newpass1!")
        .await
        .expect("complete");
    assert_eq!(returned, id);

    // New password verifies; old one no longer does.
    let ada = reload("ada@x.com").await;
    assert!(
        ada.verify_password("newpass1!").expect("verify new"),
        "the new password must verify after reset"
    );
    assert!(
        !ada.verify_password("oldpass1!").expect("verify old"),
        "the old password must no longer verify"
    );

    // The live session was revoked.
    assert!(
        session_driver
            .read("ada-sess-1")
            .await
            .expect("read session post-reset")
            .is_none(),
        "the user's session must be revoked after a completed reset"
    );

    // Single-use: a second complete on the same token fails.
    assert!(
        PasswordReset::complete(&token, "again1!").await.is_err(),
        "a consumed reset token must not complete again"
    );
}

// ============================================================================
// 4. Profile: email change re-verifies; password rotation is gated
// ============================================================================
//
// Mirrors the kit's `PATCH /profile` (email change → null the stamp + re-send
// verification) and `PUT /profile/password` (gate on the current password) +
// `DELETE /profile` (delete removes the row) logic, exercised against the real
// `User` model exactly as the controllers do.

#[tokio::test]
async fn profile_email_change_reverifies_and_password_and_delete_logic() {
    let _h = setup().await;

    let mut user = User::create("Edsger", "edsger@x.com", "oldpass1!")
        .await
        .expect("create user");
    mark_verified(&mut user).await;
    assert!(
        reload("edsger@x.com").await.is_email_verified(),
        "seed user starts verified"
    );

    // --- Email change: null the verification stamp, save, re-send the link.
    //     This mirrors `profile::update` exactly.
    let fake = Mail::fake();
    let mut user = reload("edsger@x.com").await;
    user.name = "Edsger Dijkstra".into();
    user.email = "edsger.new@x.com".into();
    user.set_email_verified_at(None);
    suprnova::eloquent::Model::save(&user)
        .await
        .expect("save profile update");
    EmailVerification::send_link(&user, "http://nebula.test/verify-email/verify")
        .await
        .expect("re-send verification after email change");

    let updated = reload("edsger.new@x.com").await;
    assert_eq!(updated.name, "Edsger Dijkstra", "name saved");
    assert_eq!(updated.email, "edsger.new@x.com", "email saved");
    assert!(
        !updated.is_email_verified(),
        "changing the email nulls email_verified_at"
    );
    assert_eq!(fake.count(), 1, "email change re-sends a verification link");
    fake.assert_sent_to("edsger.new@x.com");

    // --- Password rotation is gated on the current password. The controller
    //     rejects a wrong current password with a 422; the gate is
    //     `verify_password(current)`. Prove both arms of that gate.
    let user = reload("edsger.new@x.com").await;
    assert!(
        !user
            .verify_password("not-the-password")
            .expect("verify wrong current"),
        "a wrong current password must NOT verify (controller → 422)"
    );
    assert!(
        user.verify_password("oldpass1!").expect("verify right current"),
        "the correct current password verifies (controller → rotate)"
    );

    // Rotate: hash + save the new password, exactly as `profile::update_password`.
    let mut user = user;
    user.password = suprnova::hashing::hash("brandnew1!").expect("hash");
    suprnova::eloquent::Model::save(&user)
        .await
        .expect("save rotated password");
    let after = reload("edsger.new@x.com").await;
    assert!(
        after.verify_password("brandnew1!").expect("verify rotated"),
        "the rotated password verifies"
    );
    assert!(
        !after.verify_password("oldpass1!").expect("verify old"),
        "the old password no longer verifies"
    );

    // --- Delete removes the row (mirrors `profile::destroy` after the password
    //     gate passes). Wrong-password gate proven above via verify_password.
    suprnova::eloquent::Model::delete(after)
        .await
        .expect("delete user");
    assert!(
        User::find_by_email("edsger.new@x.com")
            .await
            .expect("lookup")
            .is_none(),
        "a confirmed delete removes the user row"
    );
}
