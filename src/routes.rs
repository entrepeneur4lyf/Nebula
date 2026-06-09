use suprnova::{get, group, post, routes};

use crate::controllers;
use crate::middleware;

routes! {
    // Public routes
    get!("/", controllers::home::index),

    // Guest-only routes (redirect to dashboard if logged in)
    group!("/", {
        get!("/login", controllers::auth::show_login),
        post!("/login", controllers::auth::login),
        get!("/register", controllers::auth::show_register),
        post!("/register", controllers::auth::register),
    }).middleware(middleware::authenticate::guest()),

    // Authenticated, verification NOT required. An unverified-but-logged-in
    // user must be able to view the notice, resend the link, hit the verify
    // link, and log out — so these stay off the `verified` gate.
    group!("/", {
        get!("/verify-email", controllers::verify_email::show_notice),
        post!("/email/verification-notification", controllers::verify_email::resend),
        get!("/verify-email/verify", controllers::verify_email::verify),
        post!("/logout", controllers::auth::logout),
    }).middleware(middleware::authenticate::auth()),

    // Authenticated AND email-verified. `verified()` composes after `auth()`,
    // so an unverified user is redirected to `/verify-email` and an
    // unauthenticated one to `/login`.
    group!("/", {
        get!("/dashboard", controllers::dashboard::index),
    })
        .middleware(middleware::authenticate::auth())
        .middleware(middleware::authenticate::verified()),
}
