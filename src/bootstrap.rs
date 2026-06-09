//! Application Bootstrap
//!
//! This is where you register global middleware and services that need runtime configuration.
//! Services that don't need runtime config can use `#[service(ConcreteType)]` instead.
//!
//! # Example
//!
//! ```rust,ignore
//! // For services with no runtime config, use the macro:
//! #[service(RedisCache)]
//! pub trait CacheStore { ... }
//!
//! // For services needing runtime config, register here:
//! pub async fn register() {
//!     // Initialize database
//!     DB::init().await.expect("Failed to connect to database");
//!
//!     // Global middleware
//!     global_middleware!(middleware::LoggingMiddleware);
//!
//!     // Services
//!     bind!(dyn Database, PostgresDB::new());
//! }
//! ```

use std::sync::Arc;

#[allow(unused_imports)]
use suprnova::{
    bind, global_middleware, indexmap::IndexMap, serde_json, singleton, App, Auth, AuthConfig,
    AuthManager, CsrfMiddleware, EloquentUserProvider, FrameworkError, IncludeMiddleware, Inertia,
    InertiaConfig, InertiaRequestExt, InertiaSharedData, Prop, SessionConfig, SessionMiddleware,
    DB,
};

use crate::middleware;
use crate::models::user::User;

/// Shares the authenticated user on every Inertia response as `auth.user`
/// — the shape `frontend/src/types/auth.ts` declares and `Layout.svelte`
/// renders the user menu / Dashboard nav link from. Guests share
/// `auth.user: null` so pages can branch on it without optional-chaining
/// surprises.
///
/// Public so the integration-test harness (`tests/http_flows.rs`) can mirror
/// this registration against its own server stack.
pub struct AuthShare;

#[suprnova::__async_trait]
impl InertiaSharedData for AuthShare {
    async fn share(
        &self,
        _req: &dyn InertiaRequestExt,
    ) -> Result<IndexMap<String, Prop>, FrameworkError> {
        let user = Auth::user_as::<User>().await?;
        let mut out = IndexMap::new();
        out.insert(
            "auth".to_string(),
            Prop::Eager(serde_json::json!({
                "user": user.map(|u| serde_json::json!({
                    "id": u.id,
                    "name": u.name,
                    "email": u.email,
                })),
            })),
        );
        Ok(out)
    }
}

/// Register global middleware and services
///
/// Called from cmd/main.rs before `Server::from_config()`.
/// Middleware and services registered here can use environment variables, config files, etc.
pub async fn register() {
    // Initialize database connection
    DB::init().await.expect("Failed to connect to database");

    // Global middleware (runs on every request in registration order)
    global_middleware!(middleware::LoggingMiddleware);

    // Session middleware (required for authentication)
    let session_config = SessionConfig::from_env();
    global_middleware!(SessionMiddleware::new(session_config));

    // CSRF protection (validates tokens on POST/PUT/PATCH/DELETE)
    global_middleware!(CsrfMiddleware::new());

    // Parse `?include=`/`?exclude=`/`?only=`/`?except=` and `?fields[...]=`
    // into the per-request task-local so `#[derive(Data)]` responses,
    // `Resource::single`, and `Prop::Lazy` resolution honour the client's
    // requested shape out of the box. Without this, Data DTOs silently
    // ignore include/fieldset query parameters.
    global_middleware!(IncludeMiddleware);

    // Authentication: register the AuthManager (the config/auth.php analogue)
    // and a user provider so `Auth::attempt` and `Auth::user_as::<User>()`
    // resolve users. `EloquentUserProvider<User>` queries the typed model; the
    // SessionMiddleware above persists the authenticated id across requests.
    App::singleton(AuthManager::new(AuthConfig::from_env()));
    Auth::register_provider("users", Arc::new(EloquentUserProvider::<User>::new()))
        .expect("register users provider");

    // Inertia protocol middlewares: the 409 version-mismatch handshake
    // (stale clients hard-reload to pick up new assets) and the 302→303
    // upgrade on non-GET redirects. Without the 303 upgrade a browser
    // replays a PATCH/PUT/DELETE verbatim against the redirect target,
    // looping until its 20-redirect cap kills the visit.
    //
    // The version given here MUST match the one the page responses embed,
    // or every Inertia GET answers 409 and the client hard-reloads on each
    // navigation. The kit's `inertia_response!` calls use the default
    // `InertiaConfig` (static version "1.0"), so install with that same
    // default. If you wire a real asset version (e.g. a build hash), set
    // it both here and on every response's config.
    Inertia::install(&InertiaConfig::new());

    // Share the authenticated user (`auth.user`) on every Inertia
    // response so the layout can render the user menu without each
    // handler threading the user through its own props.
    App::register_inertia_shared(Arc::new(AuthShare));

    // Example: Register a trait binding with runtime config
    // bind!(dyn Database, PostgresDB::new());

    // Example: Register a concrete singleton
    // singleton!(CacheService::new());

    // Add your middleware and service registrations here
}
