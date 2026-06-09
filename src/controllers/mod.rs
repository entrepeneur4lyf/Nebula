pub mod auth;
pub mod dashboard;
pub mod home;
pub mod password_reset;
pub mod profile;
pub mod static_files;
pub mod verify_email;

/// The app's external base URL (for building emailed links). `APP_URL` or the dev default.
pub(crate) fn app_url() -> String {
    std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:8765".into())
}
