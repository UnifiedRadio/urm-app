/// Structured logging setup for URM. Wrap tracing initialization here so
/// both the Tauri app and CLI tools share identical log configuration.

pub fn init(level: &str) {
    use tracing_subscriber::{fmt, EnvFilter};
    fmt()
        .with_env_filter(EnvFilter::new(level))
        .with_target(true)
        .with_thread_ids(false)
        .json()
        .init();
}
