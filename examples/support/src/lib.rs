//! Application-side helpers for the OptionStratLib example binaries.
//!
//! Installing a global `tracing` subscriber is an application decision, not
//! a library one (`rules/global_rules.md`, "Logging & Observability"), so
//! the example crates take their logger from here instead of from
//! `optionstratlib::utils::logger`, which is deprecated and leaves the
//! library in 0.22. This package is a workspace member with
//! `publish = false`; nothing outside `examples/` depends on it.

use std::env;
use std::sync::Once;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

static INIT: Once = Once::new();

/// Maps a level name to a `tracing` level; anything unknown is `INFO`.
fn parse_level(name: &str) -> Level {
    match name.to_uppercase().as_str() {
        "DEBUG" => Level::DEBUG,
        "ERROR" => Level::ERROR,
        "WARN" => Level::WARN,
        "TRACE" => Level::TRACE,
        _ => Level::INFO,
    }
}

fn install(level: Level) {
    INIT.call_once(|| {
        let subscriber = FmtSubscriber::builder().with_max_level(level).finish();
        if tracing::subscriber::set_global_default(subscriber).is_ok() {
            tracing::debug!("Log level set to: {}", level);
        }
    });
}

/// Installs a global `tracing` subscriber at the level named by the
/// `LOGLEVEL` environment variable (`DEBUG`, `INFO`, `WARN`, `ERROR`,
/// `TRACE`; default `INFO`). Idempotent: later calls are no-ops.
pub fn setup_logger() {
    let level = env::var("LOGLEVEL").unwrap_or_else(|_| "INFO".to_string());
    install(parse_level(&level));
}

/// Installs a global `tracing` subscriber at the given level name
/// (`DEBUG`, `INFO`, `WARN`, `ERROR`, `TRACE`; anything else is `INFO`).
/// Idempotent: later calls are no-ops.
pub fn setup_logger_with_level(log_level: &str) {
    install(parse_level(log_level));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_level_maps_names_and_defaults_to_info() {
        assert_eq!(parse_level("debug"), Level::DEBUG);
        assert_eq!(parse_level("ERROR"), Level::ERROR);
        assert_eq!(parse_level("Warn"), Level::WARN);
        assert_eq!(parse_level("trace"), Level::TRACE);
        assert_eq!(parse_level("nonsense"), Level::INFO);
    }

    #[test]
    fn test_setup_is_idempotent() {
        setup_logger_with_level("WARN");
        assert!(tracing::dispatcher::has_been_set());
        // Later calls are no-ops: they neither panic nor replace the
        // subscriber installed by the first call.
        setup_logger();
        setup_logger_with_level("TRACE");
        assert!(tracing::dispatcher::has_been_set());
    }
}
